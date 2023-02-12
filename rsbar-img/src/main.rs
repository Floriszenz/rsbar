fn main() {
    if let Err(e) = rsbar_img::run() {
        eprintln!("ERROR: {e}");
    }
}
