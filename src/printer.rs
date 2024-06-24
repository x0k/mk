use std::{
    error::Error,
    io::{self, Write},
    os::{self, unix::fs::PermissionsExt}, process::Command,
};

use tempfile;

use super::config::Config;

enum Printer {
    Stdout,
    Executor,
}

impl Printer {
    fn new(config: &Config) -> Self {
        if config.executable {
            Self::Executor
        } else {
            Self::Stdout
        }
    }

    fn print(&self, content: &str) -> Result<(), Box<dyn Error>> {
        match self {
            Self::Stdout => {
                println!("{}", content);
                Ok(())
            }
            Self::Executor => {
                let mut file = tempfile::tempfile()?;
                file.write_all(content.as_bytes())?;
                file.flush()?;
                file.metadata()?.permissions().set_mode(0o755);
                Command::new(file)
                    .stdin(io::stdin())
                    .stdout(io::stdout())
                    .stderr(io::stderr())
                    .args(&["/C", "echo hello"]).output()?;

                Ok(())
            }
        }
    }
}
