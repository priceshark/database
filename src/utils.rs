use indicatif::{ProgressBar, ProgressStyle};

pub fn progress_bar(len: u64) -> ProgressBar {
    ProgressBar::new(len).with_style(
        ProgressStyle::with_template("[{elapsed_precise}] {human_pos} {percent}% ({per_sec})")
            .expect("hardcoded"),
    )
}
