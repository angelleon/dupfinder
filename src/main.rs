//use std::env;
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
    //for _ in 0..cli_args.count {
    println!("{:?}", cli_args.path);
    println!("{:?}", cli_args.path.len());
    //}
    //panic!();
    //let mut args: Vec<String> = env::args().collect();
    //args.remove(0);
    let mut finder = dupfinder::DupFinder::new(cli_args.path);
    finder.run();
}
