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
    tracing::{debug, error, info, warn},
    xz2::read::XzDecoder,
};

// Windows-specific imports
#[cfg(target_os = "windows")]
use {
    anyhow::anyhow,
    std::{env::temp_dir, process::Command},
};

pub async fn update_portal() -> Result<()> {
    //  Fetch latest release
    info!("Checking for updates on GitHub...");
    let release = spawn_blocking(|| {
        let latest = Update::configure()
            .repo_owner("Spectra010s")
            .repo_name("portal")
            .bin_name("hiverra-portal")
            .current_version(cargo_crate_version!())
            .build()?
            .get_latest_release()
            .context("Failed to fetch latest release from GitHub")?;
        Ok::<Release, Error>(latest)
    })
    .await
    .context("Updating failed")??;

    let current_v = cargo_crate_version!();

    // 2. Check if newer
    if bump_is_greater(current_v, &release.version)? {
        info!("Update available: v{} -> v{}", current_v, release.version);
        println!(
            "New version found: {} (Current: v{})",
            release.version, current_v
        );

        let proceed = Confirm::new("Portal: Do you want to update?")
            .with_default(true)
            .prompt()?;

        if !proceed {
            info!("User cancelled update to v{}", release.version);
            println!("Portal: Update cancelled.");
            return Ok(());
        }

        println!("Portal: Downloading and applying update...");

        // Windows update
        #[cfg(target_os = "windows")]
        {
            spawn_blocking(move || -> Result<()> {
                info!("Target platform: Windows (MSI)");
                let tmp_dir = temp_dir();
                let dest_path = tmp_dir.join("portal_update.msi");

                let asset = release.asset_for("windows", Some("msi")).ok_or_else(|| {
                    error!("MSI asset not found for Windows");
                    anyhow!("Could not find MSI for Windows")
                })?;

                debug!("Downloading MSI from: {}", asset.download_url);
                let client = Client::builder()
                    .timeout(Duration::from_secs(300))
                    .build()?;
                let mut response = client
                    .get(&asset.download_url)
                    .header("Accept", "application/octet-stream")
                    .header("User-Agent", "portal-updater")
                    .send()?
                    .error_for_status()?;

                let mut tmp_file = File::create(&dest_path)?;
                response.copy_to(&mut tmp_file)?;
                tmp_file.sync_all()?;

                println!("Portal: Launching installer. This will close the current app...");
                info!("Executing msiexec for {}", dest_path.display());
                Command::new("msiexec")
                    .arg("/i")
                    .arg(&dest_path)
                    .arg("/passive")
                    .spawn()
                    .context("Failed to launch MSI")?;

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
                } else if cfg!(target_os = "macos") {
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

                let tmp_dir = Builder::new().prefix("portal-").tempdir()?;
                let tmp_file_path = tmp_dir.path().join(asset_name);
                let mut tmp_file = File::create(&tmp_file_path)?;
                response.copy_to(&mut tmp_file)?;
                tmp_file.sync_all()?;

                // Extract archive
                debug!("Extracting archive to {}", tmp_dir.path().display());
                let file = File::open(&tmp_file_path)?;
                if is_gz {
                    let mut archive = Archive::new(GzDecoder::new(file));
                    archive.unpack(tmp_dir.path())?;
                } else {
                    let mut archive = Archive::new(XzDecoder::new(file));
                    archive.unpack(tmp_dir.path())?;
                }

                // Replace binary
                let new_bin = tmp_dir.path().join("portal");
                info!("Replacing binary with {}", new_bin.display());
                self_replace(&new_bin).context("Binary swap failed")?;

                Ok(())
            })
            .await
            .context("Update process failed")??;
        }

        info!("Update applied successfully to v{}", release.version);
        println!("Portal: Update successful!");
    } else {
        debug!("Update check complete. System is up to date.");
        println!("Portal: Already up to date (v{}).", current_v);
    }

    Ok(())
}
