use std::{
    env, error::Error, ffi::OsStr, io::Write, os::unix::fs::PermissionsExt, path::Path,
    process::Command,
};

use clap::ValueEnum;
use rand::{Rng, distr::Alphanumeric};

use crate::graph;
use crate::node::Node;
use crate::segments_scanner::SegmentsScanner;

#[derive(Clone, ValueEnum, PartialEq)]
pub enum Printer {
    Stdout,
    Executor,
    Targets,
    DesugarDebug,
}

fn target_not_found(target: &str) -> Box<dyn Error> {
    format!("target not found: {}", target).into()
}

impl Printer {
    pub fn print<I, S>(
        &self,
        targets: &[&str],
        content: &str,
        args: I,
    ) -> Result<(), Box<dyn Error>>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        let nodes: Vec<_> = SegmentsScanner::new(content).collect();
        match self {
            Self::DesugarDebug => {
                println!("{}", content);
                Ok(())
            }
            Self::Targets => {
                let segments =
                    graph::resolve_segments(&nodes, targets).map_err(target_not_found)?;
                for node in nodes {
                    if let Node::Segment { name, .. } = node {
                        if !segments.contains(name) {
                            continue;
                        }
                        println!("{}", name);
                        if let Some(desc) = node.description() {
                            for line in desc {
                                println!(" {}", line);
                            }
                        }
                    }
                }
                Ok(())
            }
            Self::Stdout => {
                let resolved = graph::resolve(&nodes, targets).map_err(target_not_found)?;
                print!("{}", resolved);
                Ok(())
            }
            Self::Executor => {
                let resolved = graph::resolve(&nodes, targets).map_err(target_not_found)?;
                let prefix: String = rand::rng()
                    .sample_iter(&Alphanumeric)
                    .take(5)
                    .map(char::from)
                    .collect();
                let path = Path::join(&env::temp_dir(), format!("mk-{}.tmp", prefix));
                let file_path = path.to_str().unwrap().to_string();
                {
                    let mut file = std::fs::File::create(&path)?;
                    let mut permissions = file.metadata()?.permissions();
                    permissions.set_mode(0o755);
                    file.set_permissions(permissions)?;
                    file.write_all(resolved.as_bytes())?;
                    file.flush()?;
                }
                Command::new(file_path).args(args).spawn()?.wait()?;
                std::fs::remove_file(path)?;
                Ok(())
            }
        }
    }
}
