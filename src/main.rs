use std::env;
use std::fs;
use std::io::Read;

use atty::Stream;
use clap::{value_parser, Arg, Command};
use glob::glob;
use toml;

mod chars;
mod dependencies_collector;
mod glob_pattern;
mod graph;
mod groups;
mod node;
mod printer;
mod segments_scanner;
mod syntax;

use printer::Printer;
use segments_scanner::SegmentsScanner;

const META: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/Cargo.toml"));

fn parse_meta() -> Option<(&'static str, &'static str, &'static str)> {
    let meta: &'static toml::Table = Box::leak(Box::new(toml::from_str(META).ok()?));
    let package = meta.get("package")?.as_table()?;
    return Some((
        package.get("name")?.as_str()?,
        package.get("description")?.as_str()?,
        package.get("version")?.as_str()?,
    ));
}

fn read_content_from_files(pattern: &str) -> Result<String, Box<dyn std::error::Error>> {
    let mut files = Vec::new();
    let mut filenames: Vec<_> = glob(pattern).unwrap().filter_map(Result::ok).collect();
    if filenames.is_empty() {
        return Err("no mkfiles found".into());
    }
    filenames.sort();
    for path in filenames {
        match fs::read_to_string(path) {
            Ok(content) => files.push(content),
            Err(e) => return Err(e.into()),
        }
    }
    Ok(syntax::desugar(files.join("\n").as_str()))
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (name, description, version) = parse_meta().unwrap();
    let matches = Command::new(name)
        .version(version)
        .about(description)
        .arg(Arg::new("target").help("target segment(s)").num_args(0..))
        .arg(
            Arg::new("input")
                .help("input files glob pattern")
                .short('I')
                .long("input")
                .default_value("[Mm]kfile*"),
        )
        .arg(
            Arg::new("printer")
                .short('P')
                .long("printer")
                .value_parser(value_parser!(Printer)),
        )
        .get_matches();
    let content = if atty::is(Stream::Stdin) {
        read_content_from_files(matches.get_one::<String>("input").unwrap())?
    } else {
        let mut input = String::new();
        std::io::stdin().read_to_string(&mut input)?;
        input
    };
    let nodes: Vec<_> = SegmentsScanner::new(content.as_str()).collect();
    let targets: Vec<_> = matches
        .get_many::<String>("target")
        .unwrap_or_default()
        .map(|s| s.as_str())
        .collect();
    let printer: Printer = if let Some(printer) = matches.get_one::<Printer>("printer") {
        printer.clone()
    } else {
        if atty::is(Stream::Stdout) {
            Printer::Executor
        } else {
            Printer::Stdout
        }
    };
    match graph::resolve(&nodes, targets.as_slice()) {
        Ok(content) => printer.print(&content),
        Err(target) => Err(format!("target not found: {}", target).into()),
    }
}
