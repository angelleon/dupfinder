pub mod dupfinder;

use clap::Parser;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct CliOptions {
    #[clap(value_parser)]
    path: Vec<std::path::PathBuf>,
    #[arg(short = 's', long, default_value_t = 0)]
    min_size: u64,
    #[arg(short = 'S', long, default_value_t = std::u64::MAX)]
    max_size: u64,
    #[arg(short = 'v', long, default_value_t = false)]
    verbose: bool
}

fn main() {
    let cli_args = CliOptions::parse();
    let mut finder = dupfinder::DupFinder::new(cli_args.path, cli_args.min_size, cli_args.max_size, cli_args.verbose);
    finder.run();
}
