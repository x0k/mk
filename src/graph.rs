use std::collections::{HashMap, HashSet};

use glob::Pattern;

use super::chars::*;
use super::node::Node;

fn make_graph<'a>(nodes: &[Node<'a>]) -> HashMap<&'a str, HashSet<&'a str>> {
    let mut graph: HashMap<&'a str, HashSet<&'a str>> = HashMap::new();
    for node in nodes {
        match node {
            Node::Content(_) => {}
            Node::Segment {
                name, dependencies, ..
            } => {
                graph.insert(name, dependencies.iter().cloned().collect());
            }
        }
    }
    graph
}

fn resolve_targets<'a>(
    graph: &HashMap<&'a str, HashSet<&'a str>>,
    targets: &[&'a str],
) -> HashSet<&'a str> {
    let mut visited = HashSet::new();

    let mut stack = Vec::new();
    stack.extend_from_slice(targets);

    while let Some(node) = stack.pop() {
        if visited.contains(&node) {
            continue;
        }
        visited.insert(node);
        if let Some(deps) = graph.get(node) {
            stack.extend(deps.iter());
        }
    }
    visited
}

pub fn resolve_segments<'a>(
    nodes: &[Node<'a>],
    targets_or_patterns: &[&'a str],
) -> Result<HashSet<&'a str>, &'a str> {
    let graph = make_graph(nodes);
    let mut targets = Vec::new();
    for target in targets_or_patterns {
        if graph.contains_key(target) {
            targets.push(*target);
            continue;
        }
        let old_size = targets.len();
        if contains_glob_pattern_symbols(*target) {
            let pattern = Pattern::new(*target).map_err(|_| *target)?;
            for name in graph.keys() {
                if pattern.matches(name) {
                    targets.push(name);
                }
            }
        }
        if targets.len() == old_size {
            return Err(*target);
        }
    }
    Ok(resolve_targets(&graph, &targets))
}

pub fn resolve<'a>(nodes: &[Node<'a>], targets_or_patterns: &[&'a str]) -> Result<String, &'a str> {
    let segments = resolve_segments(nodes, targets_or_patterns)?;
    let mut blocks = Vec::new();
    for node in nodes {
        match node {
            Node::Content(content) => blocks.push(*content),
            Node::Segment {
                name,
                content,
                indentation,
                ..
            } => {
                if !segments.contains(name) {
                    continue;
                }
                let l = indentation.len();
                if l == 0 {
                    blocks.push(content);
                    continue;
                }
                for line in content.lines() {
                    blocks.push(&line[l..]);
                    blocks.push("\n");
                }
                if !content.ends_with("\n") {
                    blocks.pop();
                }
            }
        }
    }
    Ok(blocks.join(""))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_resolve_common_content() {
        let nodes = &[Node::Content("common content")];
        assert_eq!(resolve(nodes, &[]), Ok("common content".to_string()));
    }

    #[test]
    fn should_resolve_segment_content() {
        let nodes = &[Node::Segment {
            name: "foo",
            content: "foo content",
            dependencies: Vec::new(),
            indentation: "",
        }];
        assert_eq!(resolve(nodes, &["foo"]), Ok("foo content".to_string()));
    }

    #[test]
    fn should_resolve_all_content() {
        let nodes = &[
            Node::Content("common content\n"),
            Node::Segment {
                name: "foo",
                content: "foo content",
                dependencies: Vec::new(),
                indentation: "",
            },
        ];
        assert_eq!(
            resolve(nodes, &["foo"]),
            Ok("common content\nfoo content".to_string())
        );
    }

    #[test]
    fn should_resolve_dependency() {
        let nodes = &[
            Node::Segment {
                name: "foo",
                content: "foo content\n",
                dependencies: Vec::new(),
                indentation: "",
            },
            Node::Segment {
                name: "bar",
                content: "bar content",
                dependencies: vec!["foo"],
                indentation: "",
            },
        ];
        assert_eq!(
            resolve(nodes, &["bar"]),
            Ok("foo content\nbar content".to_string())
        );
    }

    #[test]
    fn should_resolve_with_indentation() {
        let nodes = &[
            Node::Segment {
                name: "foo",
                content: "\tfoo content\n",
                dependencies: Vec::new(),
                indentation: "\t",
            },
            Node::Segment {
                name: "bar",
                content: "    bar content",
                dependencies: vec!["foo"],
                indentation: "    ",
            },
        ];
        assert_eq!(
            resolve(nodes, &["bar"]),
            Ok("foo content\nbar content".to_string())
        );
    }

    #[test]
    fn should_not_resolve() {
        let nodes = &[
            Node::Content("common content"),
            Node::Segment {
                name: "foo",
                content: "foo content",
                dependencies: Vec::new(),
                indentation: "",
            },
        ];
        assert_eq!(resolve(nodes, &["foo", "bar"]), Err("bar"));
    }
}
