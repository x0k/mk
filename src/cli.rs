use std::env;
use std::fs;
use std::io::{IsTerminal, Read};

use clap::ArgMatches;
use clap::ValueHint;
use clap::{Arg, ArgAction, Command, value_parser};
use clap_complete::engine::{ArgValueCompleter, CompletionCandidate};
use clap_complete::env::CompleteEnv;
use clap_complete::{Generator, Shell, generate};
use glob::glob;

use super::node::Node;
use super::printer::Printer;
use super::segments_scanner::SegmentsScanner;
use super::syntax;

const META: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/Cargo.toml"));
const DEFAULT_INPUT: &str = "[Mm]kfile*";

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

pub fn read_content_from_files(pattern: &str) -> Result<String, Box<dyn std::error::Error>> {
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

fn targets_completer(current: &std::ffi::OsStr) -> Vec<CompletionCandidate> {
    let Some(current) = current.to_str() else {
        return vec![];
    };
    let content = {
        let mut stdin = std::io::stdin();
        if stdin.is_terminal() {
            read_content_from_files(DEFAULT_INPUT).unwrap_or_default()
        } else {
            let mut input = String::new();
            stdin.read_to_string(&mut input).unwrap_or_default();
            input
        }
    };
    let content = syntax::desugar(content.as_str());
    SegmentsScanner::new(content.as_str())
        .filter_map(|node| match node {
            Node::Content(_) => None,
            Node::Segment { name, .. } => {
                if name.starts_with(current) {
                    Some(CompletionCandidate::new(name))
                } else {
                    None
                }
            }
        })
        .collect()
}

fn build_cli(meta: &Meta) -> Command {
    Command::new(meta.name)
        .version(meta.version)
        .about(meta.description)
        .arg(
            Arg::new("target")
                .help("target segment(s)")
                .add(ArgValueCompleter::new(targets_completer))
                .num_args(0..),
        )
        .arg(
            Arg::new("input")
                .help("input files glob pattern")
                .short('I')
                .long("input")
                .default_value(DEFAULT_INPUT)
                .value_hint(ValueHint::AnyPath),
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

fn print_completions<G: Generator>(g: G, cmd: &mut Command) {
    generate(g, cmd, cmd.get_name().to_string(), &mut std::io::stdout());
}

pub fn get_matches() -> Result<Option<ArgMatches>, Box<dyn std::error::Error>> {
    let meta = parse_meta().unwrap();
    CompleteEnv::with_factory(|| build_cli(&meta)).complete();
    let matches = build_cli(&meta).get_matches();
    if let Some(generator) = matches.get_one::<Shell>("generate-completions").copied() {
        let mut cmd = build_cli(&meta);
        eprintln!("Generating completion file for {generator}...");
        print_completions(generator, &mut cmd);
        return Ok(None);
    }
    Ok(Some(matches))
}
