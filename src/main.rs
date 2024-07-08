use std::env;
use std::fs;

use glob::glob;

mod chars;
mod config;
mod dependencies_collector;
mod glob_pattern;
mod graph;
mod groups;
mod node;
mod printer;
mod segments_scanner;
mod syntax;

use config::Config;
use printer::Printer;
use segments_scanner::SegmentsScanner;

fn main() {
    let args = env::args().collect::<Vec<_>>();
    let mut config = Config::new();
    let mut files = Vec::new();
    let mut filenames: Vec<_> = glob("[Mm]kfile*").unwrap().filter_map(Result::ok).collect();
    if filenames.is_empty() {
        eprintln!("No mkfiles found");
        return;
    }
    filenames.sort();
    for path in filenames {
        config.assign(&path);
        match fs::read_to_string(path) {
            Ok(content) => files.push(content),
            Err(e) => eprintln!("{:?}", e),
        }
    }
    let content = syntax::desugar(files.join("\n").as_str());
    let nodes: Vec<_> = SegmentsScanner::new(content.as_str()).collect();
    let targets = args[1..].iter().map(|s| s.as_str()).collect::<Vec<_>>();
    match graph::resolve(&nodes, targets.as_slice()) {
        Ok(content) => {
            let printer = Printer::new(&config);
            match printer.print(&content) {
                Ok(_) => {}
                Err(e) => eprintln!("{:?}", e),
            }
        }
        Err(target) => eprintln!("target not found: {}", target),
    }
}
