pub mod dupfinder;

use clap::Parser;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct CliOptions {
    #[clap(value_parser)]
    path: Vec<std::path::PathBuf>,
}

fn main() {
    let cli_args = CliOptions::parse();
    let mut finder = dupfinder::DupFinder::new(cli_args.path);
    finder.run();
}
