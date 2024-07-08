use super::glob_pattern;
use super::groups;

pub fn desugar(content: &str) -> String {
    glob_pattern::desugar(groups::desugar(content).as_str())
}
