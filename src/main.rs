mod chars;
mod node;
mod graph;
mod dependencies_collector;
mod segments_scanner;
mod groups;
mod glob_pattern;

use node::Node;

fn main() {
    let nodes = vec![
        Node::Content("common content"),
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

    println!("{:?}", nodes);
}
