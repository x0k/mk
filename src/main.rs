use std::env;
use std::fs;

use clap::{Arg, Command};
use glob::glob;
use toml;

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

const META: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/Cargo.toml"));

fn parse_meta<'a>(meta: &'a toml::Table) -> Option<(&'a str, &'a str, &'a str)> {
    let package = meta.get("package")?.as_table()?;
    let name = package.get("name")?.as_str()?;
    let description = package.get("description")?.as_str()?;
    let version = package.get("version")?.as_str()?;
    return Some((name, description, version));
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let meta: &'static toml::Table = Box::leak(Box::new(toml::from_str(META)?));
    let (name, description, version) = parse_meta(meta).unwrap();
    let matches = Command::new(name).version(version).about(description).arg(
        Arg::new("target")
            .help("target segment(s)")
            .required(true)
            .num_args(0..),
    ).get_matches();
    let mut config = Config::new();
    let mut files = Vec::new();
    let mut filenames: Vec<_> = glob("[Mm]kfile*").unwrap().filter_map(Result::ok).collect();
    if filenames.is_empty() {
        return Err("no mkfiles found".into());
    }
    filenames.sort();
    for path in filenames {
        config.assign(&path);
        match fs::read_to_string(path) {
            Ok(content) => files.push(content),
            Err(e) => return Err(e.into()),
        }
    }
    let content = syntax::desugar(files.join("\n").as_str());
    let nodes: Vec<_> = SegmentsScanner::new(content.as_str()).collect();
    let targets: Vec<String> = matches.get_many::<String>("target").unwrap().collect();
    match graph::resolve(&nodes, targets.as_slice()) {
        Ok(content) => {
            let printer = Printer::new(&config);
            printer.print(&content)
        }
        Err(target) => Err(format!("target not found: {}", target).into()),
    }
}
