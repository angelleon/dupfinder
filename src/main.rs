use std::env;
pub mod dupfinder;

fn main() {
    let mut args: Vec<String> = env::args().collect();
    args.remove(0);
    let mut finder = dupfinder::DupFinder::new(args);
    finder.run();
}
