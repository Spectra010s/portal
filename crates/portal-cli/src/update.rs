use crate::progress::stream_download_with_spinner;
use anyhow::{Context, Error, Result};
use flate2::read::GzDecoder;
use inquire::Confirm;
use {
    reqwest::blocking::Client,
    self_replace::self_replace,
    self_update::{
        backends::github::Update, cargo_crate_version, update::Release, version::bump_is_greater,
    },
    std::{fs::File, time::Duration},
    tar::Archive,
    tempfile::Builder,
    tokio::task::spawn_blocking,
    tracing::{debug, error, info, trace},
    xz2::read::XzDecoder,
};

// Windows-specific imports
#[cfg(target_os = "windows")]
use {
    anyhow::anyhow,
    std::{
        env::{current_exe, temp_dir, var_os},
        path::{Path, PathBuf},
        process::Command,
    },
};

#[cfg(target_os = "windows")]
fn windows_prefers_zip_update() -> bool {
    let current = match current_exe() {
        Ok(path) => path,
        Err(_) => return false,
    };

    if powershell_receipt_matches(&current) {
        return true;
    }

    default_powershell_install_path()
        .as_deref()
        .is_some_and(|path| same_windows_path(path, &current))
}

#[cfg(target_os = "windows")]
fn powershell_receipt_matches(current: &Path) -> bool {
    let Some(local_app_data) = var_os("LOCALAPPDATA") else {
        return false;
    };

    let receipt_path = PathBuf::from(local_app_data)
        .join("Hiverra")
        .join("Portal")
        .join("install.json");
    let Ok(contents) = std::fs::read_to_string(receipt_path) else {
        return false;
    };
    let Ok(receipt) = serde_json::from_str::<serde_json::Value>(&contents) else {
        return false;
    };

    let method_matches = receipt
        .get("method")
        .and_then(|method| method.as_str())
        .is_some_and(|method| method.eq_ignore_ascii_case("powershell"));
    let path_matches = receipt
        .get("bin_path")
        .and_then(|path| path.as_str())
        .map(PathBuf::from)
        .is_some_and(|path| same_windows_path(&path, current));

    method_matches && path_matches
}

#[cfg(target_os = "windows")]
fn default_powershell_install_path() -> Option<PathBuf> {
    var_os("LOCALAPPDATA").map(|local_app_data| {
        PathBuf::from(local_app_data)
            .join("Hiverra")
            .join("Portal")
            .join("bin")
            .join("portal.exe")
    })
}

#[cfg(target_os = "windows")]
fn same_windows_path(left: &Path, right: &Path) -> bool {
    left.to_string_lossy()
        .eq_ignore_ascii_case(&right.to_string_lossy())
}

pub async fn update_portal() -> Result<()> {
    //  Fetch latest release
    info!("Checking for updates on GitHub...");
    let release = spawn_blocking(|| {
        debug!(
            "Querying GitHub API (Spectra010s/portal) for releases newer than v{}",
            cargo_crate_version!()
        );
        let latest = Update::configure()
            .repo_owner("Spectra010s")
            .repo_name("portal")
            .bin_name("hiverra-portal")
            .current_version(cargo_crate_version!())
            .build()?
            .get_latest_release()
            .context("Failed to fetch latest release from GitHub")?;

        trace!("GitHub API responded with release metadata: {:?}", latest);
        Ok::<Release, Error>(latest)
    })
    .await
    .context("Updating failed")??;

    let current_v = cargo_crate_version!();
    let new_version = release.version.clone();

    // Check if newer
    if bump_is_greater(current_v, &new_version)? {
        info!("Update available: v{} -> v{}", current_v, new_version);
        println!(
            "New version found: {} (Current: v{})",
            new_version, current_v
        );

        let proceed = Confirm::new("Portal: Do you want to update?")
            .with_default(true)
            .prompt()?;

        if !proceed {
            info!("User cancelled update to v{}", new_version);
            println!("Portal: Update cancelled.");
            return Ok(());
        }

        println!("Portal: Downloading and applying update...");

        // Windows update
        #[cfg(target_os = "windows")]
        {
            spawn_blocking(move || -> Result<()> {
                let use_zip_update = windows_prefers_zip_update();
                let (asset_name, asset) = if use_zip_update {
                    info!("Target platform: Windows (PowerShell install)");
                    let asset_name = "hiverra-portal-x86_64-pc-windows-msvc.zip";
                    let asset = release
                        .asset_for(asset_name, None)
                        .context("Windows update zip asset not found")?;
                    (asset_name, asset)
                } else {
                    info!("Target platform: Windows (MSI)");
                    let asset = release.asset_for("windows", Some("msi")).ok_or_else(|| {
                        error!("MSI asset not found for Windows");
                        anyhow!("Could not find MSI for Windows")
                    })?;
                    ("portal_update.msi", asset)
                };

                debug!("Downloading Windows update from: {}", asset.download_url);
                let client = Client::builder()
                    .timeout(Duration::from_secs(300))
                    .build()?;
                let mut response = client
                    .get(&asset.download_url)
                    .header("Accept", "application/octet-stream")
                    .header("User-Agent", "portal-updater")
                    .send()?
                    .error_for_status()?;

                trace!(
                    "Download connected. Status: {}, Content-Length: {:?}",
                    response.status(),
                    response.content_length()
                );

                let zip_tmp_dir = if use_zip_update {
                    Some(Builder::new().prefix("portal-").tempdir()?)
                } else {
                    None
                };
                let dest_path = zip_tmp_dir
                    .as_ref()
                    .map(|tmp_dir| tmp_dir.path().join(asset_name))
                    .unwrap_or_else(|| temp_dir().join(asset_name));
                let mut tmp_file = File::create(&dest_path)?;
                let total_bytes = response.content_length();
                debug!("Streaming payload to temporary file...");
                let _ = stream_download_with_spinner(
                    &mut response,
                    &mut tmp_file,
                    total_bytes,
                    if use_zip_update {
                        "Downloading archive"
                    } else {
                        "Downloading MSI"
                    },
                )?;
                tmp_file.sync_all()?;
                debug!("Download complete and synced to disk.");

                if use_zip_update {
                    let tmp_dir = zip_tmp_dir.context("Windows zip update tempdir missing")?;
                    let extract_dir = tmp_dir.path().join("extract");
                    std::fs::create_dir_all(&extract_dir)?;
                    debug!("Extracting archive to {}", extract_dir.display());
                    let status = Command::new("powershell")
                        .arg("-NoProfile")
                        .arg("-Command")
                        .arg(
                            "& { param($archive, $dest) Expand-Archive -LiteralPath $archive -DestinationPath $dest -Force }",
                        )
                        .arg(&dest_path)
                        .arg(&extract_dir)
                        .status()
                        .context("Failed to extract Windows update archive")?;

                    if !status.success() {
                        error!("Windows archive extraction failed with status: {}", status);
                        anyhow::bail!("Windows archive extraction failed");
                    }

                    let new_bin = extract_dir.join("portal.exe");
                    info!("Replacing binary with {}", new_bin.display());
                    if let Err(e) = self_replace(&new_bin) {
                        error!("self_replace failed: {}", e);
                        return Err(e).context("Binary swap failed");
                    }
                    debug!("Binary replaced successfully.");
                } else {
                    println!("Portal: Launching installer. This will close the current app...");
                    info!("Executing msiexec for {}", dest_path.display());
                    Command::new("msiexec")
                        .arg("/i")
                        .arg(&dest_path)
                        .arg("/passive")
                        .spawn()
                        .context("Failed to launch MSI")?;
                }

                Ok(())
            })
            .await
            .context("Portal: Windows update failed")??;
        }

        // Non-Windows update (Linux/macOS/Android)
        #[cfg(not(target_os = "windows"))]
        {
            spawn_blocking(move || -> Result<()> {
                // Determine asset
                let (asset_name, is_gz) = if cfg!(target_os = "android") {
                    ("hiverra-portal-aarch64-linux-android.tar.gz", true)
                } else if cfg!(target_os = "macos") && cfg!(target_arch = "aarch64") {
                    ("hiverra-portal-aarch64-apple-darwin.tar.xz", false)
                } else if cfg!(target_os = "macos") && cfg!(target_arch = "x86_64") {
                    ("hiverra-portal-x86_64-apple-darwin.tar.xz", false)
                } else if cfg!(target_arch = "aarch64") {
                    ("hiverra-portal-aarch64-unknown-linux-gnu.tar.xz", false)
                } else if cfg!(target_arch = "x86_64") {
                    ("hiverra-portal-x86_64-unknown-linux-gnu.tar.xz", false)
                } else {
                    ("hiverra-portal-aarch64-unknown-linux-gnu.tar.xz", false)
                };

                info!("Selected asset: {}", asset_name);

                let asset = release
                    .asset_for(asset_name, None)
                    .context("Asset not found for this platform")?;

                // Download with streaming to file
                debug!("Downloading archive from: {}", asset.download_url);
                let client = Client::builder()
                    .timeout(Duration::from_secs(300))
                    .build()?;
                let mut response = client
                    .get(&asset.download_url)
                    .header("Accept", "application/octet-stream")
                    .header("User-Agent", "portal-updater")
                    .send()?
                    .error_for_status()?;

                trace!(
                    "Download connected. Status: {}, Content-Length: {:?}",
                    response.status(),
                    response.content_length()
                );

                let tmp_dir = Builder::new().prefix("portal-").tempdir()?;
                let tmp_file_path = tmp_dir.path().join(asset_name);
                let mut tmp_file = File::create(&tmp_file_path)?;
                let total_bytes = response.content_length();

                debug!("Streaming payload to temporary file...");
                let _ = stream_download_with_spinner(
                    &mut response,
                    &mut tmp_file,
                    total_bytes,
                    "Downloading archive",
                )?;
                tmp_file.sync_all()?;
                debug!("Download complete and synced to disk.");

                // Extract archive
                debug!("Extracting archive to {}", tmp_dir.path().display());
                let file = File::open(&tmp_file_path)?;
                if is_gz {
                    trace!("Using GzDecoder for extraction...");
                    let mut archive = Archive::new(GzDecoder::new(file));
                    archive.unpack(tmp_dir.path())?;
                } else {
                    trace!("Using XzDecoder for extraction...");
                    let mut archive = Archive::new(XzDecoder::new(file));
                    archive.unpack(tmp_dir.path())?;
                }
                debug!("Archive extracted successfully.");

                // Replace binary
                let package_dir = if is_gz {
                    asset_name.trim_end_matches(".tar.gz")
                } else {
                    asset_name.trim_end_matches(".tar.xz")
                };
                let new_bin = tmp_dir.path().join(package_dir).join("portal");
                info!("Replacing binary with {}", new_bin.display());

                if let Err(e) = self_replace(&new_bin) {
                    error!("self_replace failed: {}", e);
                    return Err(e).context("Binary swap failed");
                }
                debug!("Binary replaced successfully.");

                Ok(())
            })
            .await
            .context("Update process failed")??;
        }

        info!("Update applied successfully to v{}", new_version);
        println!("Portal: Update successful!");
    } else {
        debug!("Update check complete. System is up to date.");
        println!("Portal: Already up to date (v{}).", current_v);
    }

    Ok(())
}
