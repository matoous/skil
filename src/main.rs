fn main() {
    if let Err(err) = skillz::run() {
        skillz::ui::error(&err.to_string());
        std::process::exit(1);
    }
}
