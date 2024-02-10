use std::path::PathBuf;

use clap::Parser;

/// Visualize a Prisma schema as a d2 diagram.
#[derive(Debug, Parser)]
#[command(version, about)]
struct Args {
    /// Parse the Prisma schema from a file.
    /// Defaults to stdin.
    #[arg()]
    input_file: Option<PathBuf>,
    /// Write the d2 diagram to a file.
    /// Defaults to stdout.
    #[arg(short, long)]
    output_file: Option<PathBuf>,
}

fn main() {
    let args = Args::parse();
    println!("{:?}", args);
}
