use std::path::PathBuf;

use clap::Parser;

#[derive(Parser, Debug)]
#[command(version)]
struct Args {
    #[arg(short)]
    secret: String,

    #[arg(short)]
    output_path: PathBuf,
}

fn main() {
    tracing_subscriber::fmt::init();

    let args = Args::parse();
}
