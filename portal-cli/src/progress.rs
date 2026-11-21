use anyhow::Result;
use indicatif::{MultiProgress, ProgressBar, ProgressDrawTarget, ProgressStyle};
use std::{
    io::{Read, Write},
    time::Duration,
};
use tracing::debug;

/// Which side of the transfer this [`ProgressManager`] is used for.
///
/// This is always required at construction time so call sites are explicit about
/// their role. There is intentionally no `Default` / `new()` shorthand — having
/// a default that silently assumes `Sender` is a footgun that caused a past bug
/// where the receiver showed "Sending item N of M" in its progress header.
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

/// Manages the terminal progress UI for a single transfer.
///
/// Layout (top → bottom, drawn on stderr):
/// ```text
/// Portal: Sending item 2 of 5  [━━━━━━━━━━╾──────────────────────────────]  2/5
/// Sending large_file.bin ████████████████░░░░░░░░░░░░  64% | 12 MB/s | 3s
/// ```
///
/// The header bar (`top`) is pinned at the top by using `MultiProgress::insert_after`
/// for every item bar, so new bars always appear below the header rather than
/// pushing it down. `enable_steady_tick` on the header keeps indicatif from
/// considering it "idle" and collapsing it during long per-file renders.
#[derive(Clone)]
pub struct ProgressManager {
    mp: MultiProgress,
    top: ProgressBar,
    side: Side,
}

impl ProgressManager {
    /// Create a new progress manager for the given transfer side.
    pub fn new_with_side(side: Side) -> Self {
        debug!("Progress UI initialized: {:?}", side);
        let mp = MultiProgress::with_draw_target(ProgressDrawTarget::stderr_with_hz(10));

        // The header bar lives at index 0. Every item bar is inserted after it
        // via insert_after(), so the header never moves.
        let top = mp.add(ProgressBar::new(0));
        let style = ProgressStyle::with_template("{msg} [{bar:40.green/white}] {pos}/{len}")
            .unwrap_or_else(|_| ProgressStyle::default_bar())
            .progress_chars("━╾─");
        top.set_style(style);
        top.set_message(format!("Portal: {}", side.verb()));
        // Steady tick prevents indicatif from treating the header as "idle"
        // and skipping redraws while item bars are active below it.
        top.enable_steady_tick(Duration::from_millis(100));

        Self { mp, top, side }
    }

    pub fn set_total_items(&self, total: usize) {
        debug!("Progress UI total items set to {}", total);
        self.top.set_length(total as u64);
        self.top.set_message(format!(
            "Portal: {} item 0 of {}",
            self.side.verb(),
            total
        ));
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

    /// Create a new per-file progress bar inserted *below* the sticky header.
    ///
    /// Using `insert_after` instead of `add` is what keeps the header pinned:
    /// indicatif renders bars in insertion order, so all item bars always appear
    /// after the header regardless of how many are active simultaneously.
    pub fn create_file_bar(&self, filename: &str, total_bytes: u64) -> ProgressBar {
        debug!(
            "Progress UI file bar created for '{}' ({} bytes)",
            filename, total_bytes
        );
        let total = if total_bytes == 0 { 1 } else { total_bytes };
        let pb = self.mp.insert_after(&self.top, ProgressBar::new(total));
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
        pb
    }

    /// Print a status line above the progress bars without corrupting their layout.
    pub fn println<S: AsRef<str>>(&self, msg: S) {
        let _ = self.mp.println(msg);
    }

    /// Stop the header bar and clear the entire progress UI.
    ///
    /// Call this once the stream completes (success or failure), before any
    /// conflict prompts or final status lines, so the terminal is clean.
    pub fn finish(&self) {
        self.top.finish_and_clear();
        let _ = self.mp.clear();
    }
}

// ── Blocking download spinner used by the update command ─────────────────────

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

// ── pxp trait bridge ──────────────────────────────────────────────────────────
//
// The pxp engine is kept free of terminal-specific code. It exposes
// ItemProgress and TransferProgress as abstract traits, and we implement them
// here using indicatif so the engine never imports a terminal library.

use pxp::{ItemProgress, TransferProgress};
use tokio::io::{AsyncRead, AsyncWrite};

/// Bridges a single `indicatif` `ProgressBar` to the `ItemProgress` trait.
pub struct IndicatifItemProgress {
    pb: ProgressBar,
}

impl ItemProgress for IndicatifItemProgress {
    /// Wrap the reader so indicatif intercepts byte reads and updates the bar.
    fn wrap_read(
        &self,
        reader: Box<dyn AsyncRead + Unpin + Send>,
    ) -> Box<dyn AsyncRead + Unpin + Send> {
        Box::new(self.pb.wrap_async_read(reader))
    }

    /// Wrap the writer so indicatif intercepts byte writes and updates the bar.
    fn wrap_write(
        &self,
        writer: Box<dyn AsyncWrite + Unpin + Send>,
    ) -> Box<dyn AsyncWrite + Unpin + Send> {
        Box::new(self.pb.wrap_async_write(writer))
    }

    fn finish_and_clear(&self) {
        self.pb.finish_and_clear();
    }
}

impl TransferProgress for ProgressManager {
    fn set_total_items(&self, total: usize) {
        ProgressManager::set_total_items(self, total);
    }

    fn set_current_item(&self, current: usize, total: usize) {
        ProgressManager::set_current_item(self, current, total);
    }

    fn create_item_progress(&self, name: &str, total_bytes: u64) -> Box<dyn ItemProgress> {
        let pb = self.create_file_bar(name, total_bytes);
        Box::new(IndicatifItemProgress { pb })
    }

    fn println(&self, msg: &str) {
        ProgressManager::println(self, msg);
    }
}
