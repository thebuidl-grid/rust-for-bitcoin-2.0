fn main() {
    if let Err(error) = wallet::run() {
        eprintln!("error: {error}");
        std::process::exit(1);
    }
}
