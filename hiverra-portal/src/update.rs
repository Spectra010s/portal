use anyhow::{Context, Error, Result};
use inquire::Confirm;
use self_update::{
    backends::github::Update, cargo_crate_version, update::Release, version::bump_is_greater,
};
use std::process::exit;
use tokio::task::spawn_blocking;

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

        Ok::<Release, Error>(latest)
    })
    .await
    .context("Updating failed")??;

    let current_v = cargo_crate_version!();

    // 2. Check if the version we found is newer than current
    if bump_is_greater(current_v, &release.version)? {
        println!(
            "New version found: {} (Current: v{})",
            release.version, current_v
        );

        let proceed = Confirm::new("Portal: Do you want to update?")
            .with_default(true)
            .prompt()?;

        if proceed {
            println!("Portal: Downloading and applying update...");

            #[cfg(target_os = "windows")]
            {
                println!("Portal: Downloading Windows Installer (MSI)...");
                // On Windows, we download the MSI and let msiexec take over
                spawn_blocking(move || -> Result<()> {
                    let tmp_dir = temp_dir();
                    let dest_path = tmp_dir.join("portal_update.msi");

                    // Manual download of the MSI asset
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
                        .arg(dest_path)
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
                // 3. Perform the update
                spawn_blocking(move || -> Result<()> {
                    Update::configure()
                        .repo_owner("Spectra010s")
                        .repo_name("portal")
                        .bin_name("hiverra-portal")
                        .show_download_progress(true)
                        .current_version(cargo_crate_version!())
                        .build()?
                        .update()
                        .context("Failed to apply update")?;

                    Ok(())
                })
                .await
                .context("Downloading failed")??;
            }
            println!("Portal: Update successful!");
            exit(0);
        } else {
            println!("Portal: Update cancelled.");
        }
    } else {
        println!("Portal: Already up to date (v{}).", current_v);
    }

    Ok(())
}
