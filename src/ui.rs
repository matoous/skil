use console::{style, Term};
use indicatif::{ProgressBar, ProgressStyle};

/// Prints a styled heading line.
pub fn heading(text: &str) {
    println!("{}", style(text).bold().cyan());
}

/// Prints a standard info line.
pub fn info(text: &str) {
    println!("{}", text);
}

/// Prints a success line.
pub fn success(text: &str) {
    println!("{}", style(text).green());
}

/// Prints a warning line to stderr.
pub fn warn(text: &str) {
    eprintln!("{}", style(text).yellow());
}

/// Prints an error line to stderr.
pub fn error(text: &str) {
    eprintln!("{}", style(text).red());
}

/// Prints a list item with a dimmed bullet.
pub fn list_item(text: &str) {
    println!("  {} {}", style("-").dim(), text);
}

/// Creates a spinner that is hidden when not running in a TTY.
pub fn spinner(message: &str) -> ProgressBar {
    let pb = if Term::stdout().is_term() {
        ProgressBar::new_spinner()
    } else {
        ProgressBar::hidden()
    };
    pb.set_style(
        ProgressStyle::with_template("{spinner} {msg}")
            .unwrap_or_else(|_| ProgressStyle::default_spinner()),
    );
    pb.set_message(message.to_string());
    pb.enable_steady_tick(std::time::Duration::from_millis(100));
    pb
}
