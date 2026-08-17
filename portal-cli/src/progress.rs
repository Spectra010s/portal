use anyhow::Result;
use indicatif::{MultiProgress, ProgressBar, ProgressDrawTarget, ProgressStyle};
use std::{
    io::{Read, Write},
    time::Duration,
};
use tracing::debug;

#[derive(Clone)]
pub struct ProgressManager {
    mp: MultiProgress,
    top: ProgressBar,
    side: Side,
}

// Which side of the transfer this manager is used for.
#[derive(Clone, Copy, Debug)]
pub enum Side {
    Sender,
    Receiver,
}

impl Side {
    fn verb(self) -> &'static str {
        match self {
            Side::Sender => "Sending",
            Side::Receiver => "Receiving",
        }
    }
}

impl ProgressManager {
    pub fn new() -> Self {
        Self::new_with_side(Side::Sender)
    }

    pub fn new_with_side(side: Side) -> Self {
        debug!("Progress UI initialized: {:?}", side);
        let mp = MultiProgress::with_draw_target(ProgressDrawTarget::stderr_with_hz(10));
        let top = mp.add(ProgressBar::new(0));
        let style = ProgressStyle::with_template("{msg} [{bar:40.green/white}] {pos}/{len}")
            .unwrap_or_else(|_| ProgressStyle::default_bar())
            .progress_chars("━╾─");
        top.set_style(style);
        top.set_message(format!("Portal: {}", side.verb()));
        Self { mp, top, side }
    }

    pub fn set_total_items(&self, total: usize) {
        debug!("Progress UI total items set to {}", total);
        self.top.set_length(total as u64);
        self.top
            .set_message(format!("Portal: {} item 0 of {}", self.side.verb(), total));
    }

    pub fn set_current_item(&self, current: usize, total: usize) {
        debug!("Progress UI current item: {} of {}", current, total);
        self.top.set_position(current as u64);
        self.top.set_message(format!(
            "Portal: {} item {} of {}",
            self.side.verb(),
            current,
            total
        ));
    }

    pub fn create_file_bar(&self, filename: &str, total_bytes: u64) -> ProgressBar {
        debug!(
            "Progress UI file bar created for '{}' ({} bytes)",
            filename, total_bytes
        );
        let total = if total_bytes == 0 { 1 } else { total_bytes };
        let pb = ProgressBar::new(total);
        let sty = ProgressStyle::with_template(
            "{msg} {bar:40.cyan/blue} {percent:>3}% | {bytes_per_sec} | {eta}",
        )
        .unwrap_or_else(|_| ProgressStyle::default_bar());
        pb.set_style(sty);
        pb.enable_steady_tick(Duration::from_millis(120));
        pb.set_message(format!("{} {}", self.side.verb(), filename));
        if total_bytes == 0 {
            pb.set_position(1);
        }
        self.mp.add(pb)
    }

    pub fn println<S: AsRef<str>>(&self, msg: S) {
        let _ = self.mp.println(msg);
    }

    /// Stops and clears the progress UI. Called once the stream completes, before any
    /// conflict prompts or final status output, so the terminal stays clean.
    pub fn finish(&self) {
        self.top.finish_and_clear();
        let _ = self.mp.clear();
    }
}

pub fn stream_download_with_spinner<R: Read, W: Write>(
    reader: &mut R,
    writer: &mut W,
    total_bytes: Option<u64>,
    label: &str,
) -> Result<u64> {
    let progress = match total_bytes {
        Some(total) if total > 0 => {
            let pb = ProgressBar::new(total);
            let style = ProgressStyle::with_template("{msg} {percent:>3}% [{bar:24.cyan/blue}]")
                .unwrap_or_else(|_| ProgressStyle::default_bar())
                .progress_chars("=> ");
            pb.set_style(style);
            pb.set_message(format!("Portal: {}...", label));
            pb
        }
        _ => {
            let pb = ProgressBar::new_spinner();
            let style = ProgressStyle::with_template("{spinner:.cyan} {msg}")
                .unwrap_or_else(|_| ProgressStyle::default_spinner())
                .tick_strings(&[
                    "[>>      ]",
                    "[=>>     ]",
                    "[==>>    ]",
                    "[ ===>>  ]",
                    "[  ===>> ]",
                    "[   ==>> ]",
                    "[    =>> ]",
                    "[     >> ]",
                ]);
            pb.set_style(style);
            pb.enable_steady_tick(Duration::from_millis(120));
            pb.set_message(format!("Portal: {}...", label));
            pb
        }
    };

    let mut downloaded = 0_u64;
    let mut buf = [0_u8; 16 * 1024];

    loop {
        let n = reader.read(&mut buf)?;
        if n == 0 {
            break;
        }
        writer.write_all(&buf[..n])?;
        downloaded += n as u64;
        if total_bytes.is_some() {
            progress.set_position(downloaded);
        }
    }

    writer.flush()?;
    progress.finish_with_message(format!("Portal: {} complete", label));
    Ok(downloaded)
}

/// PXP Trait Implementations 
//
// We want to keep the core `pxp` engine completely free of terminal-specific code (no println, no indicatif).
// To do that, the engine defines abstract traits `ItemProgress` and `TransferProgress`.
// Here in the CLI, we implement those traits using our terminal progress bar manager (`indicatif`).
// This acts as a bridge: `pxp` handles the raw data bytes, calls these hooks, and our adapters update the terminal screen!
use pxp::{ItemProgress, TransferProgress};
use tokio::io::{AsyncRead, AsyncWrite};

/// An adapter that wraps a standard `indicatif` ProgressBar to implement the core's `ItemProgress` trait.
pub struct IndicatifItemProgress {
    pb: ProgressBar,
}

impl ItemProgress for IndicatifItemProgress {
    // When the core starts reading/writing a file, it calls these wrapping functions.
    // We use indicatif's built-in wrapper streams so that as the core reads/writes bytes,
    // the progress bar updates automatically without any manual byte counting in the engine.
    fn wrap_read(
        &self,
        reader: Box<dyn AsyncRead + Unpin + Send>,
    ) -> Box<dyn AsyncRead + Unpin + Send> {
        Box::new(self.pb.wrap_async_read(reader))
    }

    fn wrap_write(
        &self,
        writer: Box<dyn AsyncWrite + Unpin + Send>,
    ) -> Box<dyn AsyncWrite + Unpin + Send> {
        Box::new(self.pb.wrap_async_write(writer))
    }

    // Called when the transfer of a single item is finished. We clean up the bar from the terminal.
    fn finish_and_clear(&self) {
        self.pb.finish_and_clear();
    }
}

impl TransferProgress for ProgressManager {
    // The core calls these to set overall transfer progress (e.g. "Sending file 2 of 5").
    fn set_total_items(&self, total: usize) {
        ProgressManager::set_total_items(self, total);
    }

    fn set_current_item(&self, current: usize, total: usize) {
        ProgressManager::set_current_item(self, current, total);
    }

    // When the core starts a new item, it requests an `ItemProgress` tracker from us.
    // We create a fresh file progress bar and wrap it in our adapter.
    fn create_item_progress(&self, name: &str, total_bytes: u64) -> Box<dyn ItemProgress> {
        let pb = self.create_file_bar(name, total_bytes);
        Box::new(IndicatifItemProgress { pb })
    }

    // Lets the core print text status messages cleanly without breaking the active progress bar layouts.
    fn println(&self, msg: &str) {
        ProgressManager::println(self, msg);
    }
}
