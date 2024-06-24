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

use config::Config;
use printer::Printer;
use segments_scanner::SegmentsScanner;

fn main() {
    let args = env::args().collect::<Vec<_>>();
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
