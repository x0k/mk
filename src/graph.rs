use std::collections::{HashMap, HashSet};

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

fn resolve_target<'a>(
    graph: &HashMap<&'a str, HashSet<&'a str>>,
    target: &'a str,
) -> HashSet<&'a str> {
    let mut visited = HashSet::new();

    let mut stack = Vec::new();
    stack.push(target);

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

fn resolve(target: &str, nodes: &[Node]) -> String {
    let graph = make_graph(nodes);
    let segments = resolve_target(&graph, target);
    let mut result = Vec::new();
    for node in nodes {
        match node {
            Node::Content(content) => result.push(*content),
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
                for line in content.lines() {
                    result.push(&line[l..]);
                }
                if content.ends_with("\n") {
                    result.push("");
                }
            }
        }
    }
    result.join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_resolve_common_content() {
        let nodes = &[Node::Content("common content")];
        assert_eq!(resolve("", nodes), "common content");
    }

    #[test]
    fn should_resolve_segment_content() {
        let nodes = &[Node::Segment {
            name: "foo",
            content: "foo content",
            dependencies: Vec::new(),
            indentation: "",
        }];
        assert_eq!(resolve("foo", nodes), "foo content");
    }

    #[test]
    fn should_resolve_all_content() {
        let nodes = &[
            Node::Content("common content"),
            Node::Segment {
                name: "foo",
                content: "foo content",
                dependencies: Vec::new(),
                indentation: "",
            },
        ];
        assert_eq!(resolve("foo", nodes), "common content\nfoo content");
    }

    #[test]
    fn should_resolve_dependency() {
        let nodes = &[
            Node::Segment {
                name: "foo",
                content: "foo content",
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
        assert_eq!(resolve("bar", nodes), "foo content\nbar content");
    }

    #[test]
    fn should_resolve_with_indentation() {
        let nodes = &[
            Node::Segment {
                name: "foo",
                content: "\tfoo content",
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
        assert_eq!(resolve("bar", nodes), "foo content\nbar content");
    }
}
