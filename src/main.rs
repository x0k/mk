use std::collections::{HashMap, HashSet};

#[derive(Debug)]
enum Node {
    Content(String),
    Segment {
        name: String,
        content: String,
        dependencies: Vec<String>,
    },
}

fn make_graph(nodes: &Vec<Node>) -> HashMap<String, HashSet<String>> {
    let mut graph: HashMap<String, HashSet<String>> = HashMap::new();
    for node in nodes {
        match node {
            Node::Content(_) => {}
            Node::Segment {
                name, dependencies, ..
            } => {
                graph.insert(name.clone(), dependencies.clone().into_iter().collect());
            }
        }
    }
    graph
}

fn resolve_target(nodes: &Vec<Node>, target: &str) -> HashSet<String> {
    let graph = make_graph(nodes);
    let mut visited = HashSet::new();

    let mut stack = Vec::new();
    stack.push(target.to_string());

    while let Some(node) = stack.pop() {
        if visited.contains(&node) {
            continue;
        }
        visited.insert(node.clone());
        if let Some(neighbors) = graph.get(&node) {
            stack.extend(neighbors.iter().cloned());
        }
    }
    visited
}

fn resolve(target: String, nodes: Vec<Node>) -> String {
    let segments = resolve_target(&nodes, &target);
    let mut result = String::new();
    for node in nodes {
        match node {
            Node::Content(content) => {
                if !result.is_empty() {
                    result.push_str("\n");
                }
                result.push_str(&content)
            }
            Node::Segment { name, content, .. } => {
                if !result.is_empty() {
                    result.push_str("\n");
                }
                if segments.contains(&name) {
                    result.push_str(&content);
                }
            }
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_resolve_common_content() {
        let nodes = vec![Node::Content("common content".to_string())];
        assert_eq!(resolve("".to_string(), nodes), "common content");
    }

    #[test]
    fn should_resolve_segment_content() {
        let nodes = vec![Node::Segment {
            name: "foo".to_string(),
            content: "foo content".to_string(),
            dependencies: Vec::new(),
        }];
        assert_eq!(resolve("foo".to_string(), nodes), "foo content");
    }

    #[test]
    fn should_resolve_all_content() {
        let nodes = vec![
            Node::Content("common content".to_string()),
            Node::Segment {
                name: "foo".to_string(),
                content: "foo content".to_string(),
                dependencies: Vec::new(),
            },
        ];
        assert_eq!(
            resolve("foo".to_string(), nodes),
            "common content\nfoo content"
        );
    }

    #[test]
    fn should_resolve_dependency() {
        let nodes = vec![
            Node::Segment {
                name: "foo".to_string(),
                content: "foo content".to_string(),
                dependencies: Vec::new(),
            },
            Node::Segment {
                name: "bar".to_string(),
                content: "bar content".to_string(),
                dependencies: vec!["foo".to_string()],
            },
        ];
        assert_eq!(
            resolve("bar".to_string(), nodes),
            "foo content\nbar content"
        );
    }
}

fn main() {
    let nodes = vec![
        Node::Content("common content".to_string()),
        Node::Segment {
            name: "foo".to_string(),
            content: "foo content".to_string(),
            dependencies: Vec::new(),
        },
        Node::Segment {
            name: "bar".to_string(),
            content: "bar content".to_string(),
            dependencies: vec!["foo".to_string()],
        },
    ];

    println!("{:?}", nodes);
}
