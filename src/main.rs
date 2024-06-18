#[derive(Debug)]
enum Node {
    Content(String),
    Segment {
        name: String,
        content: String,
        dependencies: Vec<String>,
    },
}

fn resolve(target: String, nodes: Vec<Node>) -> String {
    let mut result = String::new();
    for node in nodes {
        match node {
            Node::Content(content) => result.push_str(&content),
            Node::Segment { name, content, .. } => {
                if name == target {
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
        assert_eq!(
            resolve("".to_string(), nodes),
            "common content"
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
