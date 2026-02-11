fn main() {
    if let Err(err) = skil::run() {
        skil::ui::error(&err.to_string());
        std::process::exit(1);
    }
}
