use std::{
    env, error::Error, ffi::OsStr, io::Write, os::unix::fs::PermissionsExt, path::Path, process::Command
};

use clap::ValueEnum;
use rand::{distributions::Alphanumeric, Rng};

#[derive(Clone, ValueEnum)]
pub enum Printer {
    Stdout,
    Executor,
}

impl Printer {
    pub fn print<I, S>(&self, content: &str, args: I) -> Result<(), Box<dyn Error>>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        match self {
            Self::Stdout => {
                print!("{}", content);
                Ok(())
            }
            Self::Executor => {
                let prefix: String = rand::thread_rng()
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
                    file.write_all(content.as_bytes())?;
                    file.flush()?;
                }
                Command::new(file_path).args(args).spawn()?.wait()?;
                std::fs::remove_file(path)?;
                Ok(())
            }
        }
    }
}
