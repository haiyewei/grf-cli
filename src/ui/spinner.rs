use indicatif::{ProgressBar, ProgressStyle};

pub fn spinner(message: &str) -> ProgressBar {
    let bar = ProgressBar::new_spinner();
    bar.enable_steady_tick(std::time::Duration::from_millis(80));
    bar.set_style(
        ProgressStyle::with_template("{spinner} {msg}")
            .unwrap_or_else(|_| ProgressStyle::default_spinner()),
    );
    bar.set_message(message.to_string());
    bar
}
