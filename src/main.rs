use std::io::{IsTerminal, Read};
use std::iter;

mod chars;
mod cli;
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

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let Some(matches) = cli::get_matches()? else {
        return Ok(());
    };
    let content = {
        let mut stdin = std::io::stdin();
        if stdin.is_terminal() {
            cli::read_content_from_files(matches.get_one::<String>("input").unwrap())?
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
            let printer = matches.get_one::<Printer>("printer").unwrap_or(
                if std::io::stdout().is_terminal() {
                    &Printer::Executor
                } else {
                    &Printer::Stdout
                },
            );
            let args: Vec<&String> = matches.get_many("arguments").unwrap_or_default().collect();
            printer.print(&content, args.as_slice())
        }
        Err(target) => Err(format!("target not found: {}", target).into()),
    }
}
