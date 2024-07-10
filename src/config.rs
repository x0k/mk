use std::path::PathBuf;

use super::printer::Printer;

pub struct Config {
    pub printer: Printer,
}

impl Config {
    pub fn new() -> Config {
        Config { printer: Printer::Stdout }
    }

    pub fn parse(&mut self, path: &PathBuf) {
        let str = path.to_str();
        if str.is_none() {
            return;
        }
        let mut str = str.unwrap();
        if !(str.starts_with("Mkfile") || str.starts_with("mkfile")) {
            return;
        }
        str = &str[6..];
        if let Some(p) = str.find(".") {
            str = &str[..p];
        }
        if str.is_empty() {
            return;
        }
        if str.contains('x') {
            self.printer = Printer::Executor;
        };
    }
}
