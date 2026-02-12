use anyhow::{Context, Result};
use flate2::read::GzDecoder;
use inquire::Confirm;
use reqwest::blocking::Client;
use self_replace::self_replace;
use self_update::{
    backends::github::Update, cargo_crate_version, update::Release, version::bump_is_greater,
};
use std::{io::Cursor, time::Duration};
use tar::Archive;
use tempfile::Builder;
use tokio::task::spawn_blocking;
use xz2::read::XzDecoder;

// Only import these when compiling for Windows
#[cfg(target_os = "windows")]
use {
    anyhow::anyhow,
    self_update::Download,
    std::{env::temp_dir, fs::File, process::Command},
};

pub async fn update_portal() -> Result<()> {
    // 1. Fetch the latest release
    let release = spawn_blocking(|| {
        let latest = Update::configure()
            .repo_owner("Spectra010s")
            .repo_name("portal")
            .bin_name("hiverra-portal")
            .current_version(cargo_crate_version!())
            .build()?
            .get_latest_release()
            .context("Failed to fetch latest release from GitHub")?;

        Ok::<Release, anyhow::Error>(latest)
    })
    .await
    .context("Updating failed")??;

    let current_v = cargo_crate_version!();

    // 2. Check if the release is newer
    if bump_is_greater(current_v, &release.version)? {
        println!(
            "New version found: {} (Current: v{})",
            release.version, current_v
        );

        let proceed = Confirm::new("Portal: Do you want to update?")
            .with_default(true)
            .prompt()?;

        if !proceed {
            println!("Portal: Update cancelled.");
            return Ok(());
        }

        println!("Portal: Downloading and applying update...");

        #[cfg(target_os = "windows")]
        {
            spawn_blocking(move || -> Result<()> {
                let tmp_dir = temp_dir();
                let dest_path = tmp_dir.join("portal_update.msi");

                let source = Download::from_url(
                    &release
                        .asset_for("windows", Some("msi"))
                        .ok_or_else(|| anyhow!("Could not find MSI for Windows"))?
                        .download_url,
                );

                let mut dest_file = File::create(&dest_path)?;
                source.download_to(&mut dest_file)?;

                println!("Portal: Launching installer. This will close the current app...");

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

        #[cfg(not(target_os = "windows"))]
        {
            spawn_blocking(move || -> Result<()> {
                // 3. Determine correct asset for platform
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

                let asset = release
                    .asset_for(asset_name, None)
                    .context("Asset not found for this platform")?;

                // 4. Download asset with long timeout
                let client = Client::builder()
                    .timeout(Duration::from_secs(300))
                    .build()?;

                let response = client
                    .get(&asset.download_url)
                    .header("Accept", "application/octet-stream")
                    .header("User-Agent", "portal-updater")
                    .send()?;

                if !response.status().is_success() {
                    anyhow::bail!("Download failed: {}", response.status());
                }

                let bytes = response.bytes()?;
                let cursor = Cursor::new(bytes);

                // 5. Create temp dir with portal- prefix
                let tmp_dir = Builder::new().prefix("portal-").tempdir()?;

                // 6. Extract based on archive type
                if is_gz {
                    let mut archive = Archive::new(GzDecoder::new(cursor));
                    archive.unpack(tmp_dir.path())?;
                } else {
                    let mut archive = Archive::new(XzDecoder::new(cursor));
                    archive.unpack(tmp_dir.path())?;
                }

                // 7. binary replacement
                let new_bin = tmp_dir.path().join("hiverra-portal");
                self_replace(&new_bin).context("Binary swap failed")?;

                Ok(())
            })
            .await
            .context("Update process failed")??;
        }

        println!("Portal: Update successful!");
    } else {
        println!("Portal: Already up to date (v{}).", current_v);
    }

    Ok(())
}
