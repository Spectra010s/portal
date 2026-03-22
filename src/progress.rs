use indicatif::{MultiProgress, ProgressBar, ProgressDrawTarget, ProgressStyle};
use std::time::Duration;
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
}
