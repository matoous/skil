use console::{style, Term};
use indicatif::{ProgressBar, ProgressStyle};

pub fn heading(text: &str) {
    println!("{}", style(text).bold().cyan());
}

pub fn info(text: &str) {
    println!("{}", text);
}

pub fn success(text: &str) {
    println!("{}", style(text).green());
}

pub fn warn(text: &str) {
    eprintln!("{}", style(text).yellow());
}

pub fn error(text: &str) {
    eprintln!("{}", style(text).red());
}

pub fn list_item(text: &str) {
    println!("  {} {}", style("-").dim(), text);
}

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
