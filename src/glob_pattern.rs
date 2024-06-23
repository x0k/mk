use std::iter;

use glob::Pattern;

use super::chars::*;
use super::node::Node;
use super::segments_scanner::SegmentsScanner;

fn resolve_glob_pattern(segments: &[&str], pattern_str: &str) -> Vec<String> {
    let mut result = Vec::new();
    match Pattern::new(pattern_str) {
        Ok(pattern) => {
            for segment in segments {
                if pattern.matches(segment) {
                    result.push(segment.to_string());
                }
            }
        }
        Err(_) => {
            result.push(pattern_str.to_string());
        }
    }
    result
}

pub fn desugar(content: &str) -> String {
    let nodes: Vec<_> = SegmentsScanner::new(content).collect();
    let segments: Vec<_> = nodes
        .iter()
        .filter_map(|n| match n {
            Node::Segment { name, .. } => Some(*name),
            _ => None,
        })
        .collect();
    nodes
        .into_iter()
        .map(|n| match n {
            Node::Content(c) => c.to_string(),
            Node::Segment {
                name,
                content,
                dependencies,
                ..
            } => {
                format!(
                    "{}:{}\n{}",
                    name,
                    iter::once("".to_string())
                        .chain(dependencies.into_iter().flat_map(|d| {
                            if !contains_glob_pattern_symbols(d) {
                                return vec![d.to_string()].into_iter();
                            }
                            resolve_glob_pattern(&segments, d).into_iter()
                        }))
                        .collect::<Vec<_>>()
                        .join(" "),
                    content
                )
            }
        })
        .collect::<Vec<_>>()
        .join("")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_desugar_content() {
        assert_eq!(
            desugar("f/check:\nf/build:\nbuild: f/*"),
            "f/check:\nf/build:\nbuild: f/check f/build\n"
        );
    }
}
