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
mod segments_scanner;

use config::Config;
use segments_scanner::SegmentsScanner;

fn main() {
    let args = env::args().collect::<Vec<_>>();
    if args.len() < 2 {
        eprintln!("Usage: {} <target>", args[0]);
        return;
    }
    let target = args[1].as_str();
    let mut config = Config::new();
    let mut files = Vec::new();
    for entry in glob("[Mm]kfile*").unwrap() {
        match entry {
            Ok(path) => {
                config.assign(&path);
                match fs::read_to_string(path) {
                    Ok(content) => files.push(content),
                    Err(e) => eprintln!("{:?}", e),
                }
            }
            Err(e) => eprintln!("{:?}", e),
        }
    }
    let content = glob_pattern::desugar(groups::desugar(files.join("\n").as_str()).as_str());
    let nodes: Vec<_> = SegmentsScanner::new(content.as_str()).collect();
    match graph::resolve(&nodes, target) {
        // TODO: Printer
        Some(content) => println!("{}", content),
        None => eprintln!("Target {} not found", target),
    }
}
