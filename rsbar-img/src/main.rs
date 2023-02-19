use clap::Parser;
use rsbar_img::Args;

fn main() {
    let args = Args::parse();

    if let Err(e) = rsbar_img::run(args) {
        eprintln!("ERROR: {e}");
    }
}
