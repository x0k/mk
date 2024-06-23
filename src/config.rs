use std::path::PathBuf;

pub struct Config {
    executable: bool,
}

impl Config {
    pub fn new() -> Config {
        Config { executable: false }
    }

    pub fn assign(&mut self, path: &PathBuf) {
        let str = path.to_str();
        if str.is_none() {
            return;
        }
        let mut str = &str.unwrap()[6..];
        if let Some(p) = str.find(".") {
            str = &str[..p];
        }
        if str.is_empty() {
            return;
        }
        self.executable = self.executable || str.contains('x');
    }
}
