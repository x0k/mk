use super::glob_pattern;
use super::groups;
use super::templates;

pub fn desugar(content: &str) -> Result<String, templates::Error> {
    let content = templates::expand(content)?;
    Ok(glob_pattern::desugar(
        groups::desugar(content.as_str()).as_str(),
    ))
}
