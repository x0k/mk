use std::{
    env, error::Error, io::Write, os::unix::fs::PermissionsExt, path::Path, process::Command,
};

use rand::{distributions::Alphanumeric, Rng};

use super::config::Config;

pub enum Printer {
    Stdout,
    Executor,
}

impl Printer {
    pub fn new(config: &Config) -> Self {
        if config.executable {
            Self::Executor
        } else {
            Self::Stdout
        }
    }

    pub fn print(&self, content: &str) -> Result<(), Box<dyn Error>> {
        match self {
            Self::Stdout => {
                println!("{}", content);
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
                Command::new(file_path).spawn()?.wait()?;
                std::fs::remove_file(path)?;
                Ok(())
            }
        }
    }
}
