use anyhow::Result;
use clap::Parser;
use rsbar_img::Args;

fn main() -> Result<()> {
    let args = Args::parse();

    rsbar_img::run(args)
}
