use std::collections::{HashMap, HashSet};

#[derive(Debug)]
enum Node<'a> {
    Content(&'a str),
    Segment {
        name: &'a str,
        content: &'a str,
        dependencies: Vec<&'a str>,
    },
}

fn make_graph<'a>(nodes: &Vec<Node<'a>>) -> HashMap<&'a str, HashSet<&'a str>> {
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

fn resolve_target<'a>(nodes: &Vec<Node<'a>>, target: &'a str) -> HashSet<&'a str> {
    let graph = make_graph(nodes);
    let mut visited = HashSet::new();

    let mut stack = Vec::new();
    stack.push(target);

    while let Some(node) = stack.pop() {
        if visited.contains(&node) {
            continue;
        }
        visited.insert(node.clone());
        if let Some(deps) = graph.get(node) {
            stack.extend(deps.iter());
        }
    }
    visited
}

fn resolve(target: &str, nodes: Vec<Node>) -> String {
    let segments = resolve_target(&nodes, &target);
    let mut result = Vec::new();
    for node in nodes {
        match node {
            Node::Content(content) => result.push(content),
            Node::Segment { name, content, .. } => {
                if segments.contains(&name) {
                    result.push(content)
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
        let nodes = vec![Node::Content("common content")];
        assert_eq!(resolve("", nodes), "common content");
    }

    #[test]
    fn should_resolve_segment_content() {
        let nodes = vec![Node::Segment {
            name: "foo",
            content: "foo content",
            dependencies: Vec::new(),
        }];
        assert_eq!(resolve("foo", nodes), "foo content");
    }

    #[test]
    fn should_resolve_all_content() {
        let nodes = vec![
            Node::Content("common content"),
            Node::Segment {
                name: "foo",
                content: "foo content",
                dependencies: Vec::new(),
            },
        ];
        assert_eq!(resolve("foo", nodes), "common content\nfoo content");
    }

    #[test]
    fn should_resolve_dependency() {
        let nodes = vec![
            Node::Segment {
                name: "foo",
                content: "foo content",
                dependencies: Vec::new(),
            },
            Node::Segment {
                name: "bar",
                content: "bar content",
                dependencies: vec!["foo"],
            },
        ];
        assert_eq!(resolve("bar", nodes), "foo content\nbar content");
    }
}

fn main() {
    let nodes = vec![
        Node::Content("common content"),
        Node::Segment {
            name: "foo",
            content: "foo content",
            dependencies: Vec::new(),
        },
        Node::Segment {
            name: "bar",
            content: "bar content",
            dependencies: vec!["foo"],
        },
    ];

    println!("{:?}", nodes);
}
