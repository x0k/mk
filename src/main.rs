use std::env;
use std::fs;
use std::io::{IsTerminal, Read};
use std::iter;

use clap::{value_parser, Arg, ArgAction, Command};
use clap_complete::{generate, Shell, Generator};
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

struct Meta {
    name: &'static str,
    description: &'static str,
    version: &'static str,
}

fn parse_meta() -> Option<Meta> {
    let meta: &'static toml::Table = Box::leak(Box::new(toml::from_str(META).ok()?));
    let package = meta.get("package")?.as_table()?;
    return Some(Meta {
        name: package.get("name")?.as_str()?,
        description: package.get("description")?.as_str()?,
        version: package.get("version")?.as_str()?,
    });
}

fn read_content_from_files(pattern: &str) -> Result<String, Box<dyn std::error::Error>> {
    let mut files = Vec::new();
    let mut filenames: Vec<_> = glob(pattern).unwrap().filter_map(Result::ok).collect();
    if filenames.is_empty() {
        let mut cwd = env::current_dir()?;
        if cwd.pop() {
            env::set_current_dir(cwd)?;
            return read_content_from_files(pattern);
        }
        return Err("no mkfiles found".into());
    }
    filenames.sort();
    for path in filenames {
        match fs::read_to_string(path) {
            Ok(content) => files.push(content),
            Err(e) => return Err(e.into()),
        }
    }
    Ok(files.join("\n"))
}

fn build_cli(meta: &Meta) -> Command {
    Command::new(meta.name)
        .version(meta.version)
        .about(meta.description)
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
        .arg(
            Arg::new("generate-completions")
                .long("generate-completions")
                .action(ArgAction::Set)
                .value_parser(value_parser!(Shell)),
        )
        .arg(
            Arg::new("arguments")
                .help("Arguments passed to the executable script")
                .num_args(1..)
                .last(true),
        )
}

fn print_completions<G: Generator>(gen: G, cmd: &mut Command) {
    generate(gen, cmd, cmd.get_name().to_string(), &mut std::io::stdout());
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let meta = parse_meta().unwrap();
    let matches = build_cli(&meta).get_matches();
    if let Some(generator) = matches
        .get_one::<Shell>("generate-completions")
        .copied()
    {
        let mut cmd = build_cli(&meta);
        eprintln!("Generating completion file for {generator}...");
        print_completions(generator, &mut cmd);
        return Ok(());
    }

    let content = {
        let mut stdin = std::io::stdin();
        if stdin.is_terminal() {
            read_content_from_files(matches.get_one::<String>("input").unwrap())?
        } else {
            let mut input = String::new();
            stdin.read_to_string(&mut input)?;
            input
        }
    };
    let content = syntax::desugar(content.as_str());
    if matches.get_one::<Printer>("printer") == Some(&Printer::DesugarDebug) {
        return Printer::DesugarDebug.print(&content, iter::empty::<&str>());
    }
    let nodes: Vec<_> = SegmentsScanner::new(content.as_str()).collect();
    let targets: Vec<_> = matches
        .get_many::<String>("target")
        .unwrap_or_default()
        .map(|s| s.as_str())
        .collect();
    match graph::resolve(&nodes, targets.as_slice()) {
        Ok(content) => {
            let printer: Printer = if let Some(printer) = matches.get_one::<Printer>("printer") {
                printer.clone()
            } else {
                if std::io::stdout().is_terminal() {
                    Printer::Executor
                } else {
                    Printer::Stdout
                }
            };
            let args: Vec<&String> = matches.get_many("arguments").unwrap_or_default().collect();
            printer.print(&content, args.as_slice())
        }
        Err(target) => Err(format!("target not found: {}", target).into()),
    }
}
