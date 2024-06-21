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
        visited.insert(node);
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

enum StateKind {
    SegmentNotDefined,
    SegmentStarts,
    SegmentContinued,
}

struct ScannerState<'a> {
    kind: StateKind,
    segment: &'a str,
    targets: Vec<&'a str>,
}

struct SegmentsScanner<'a> {
    content: &'a str,
    cursor: usize,
    states: [ScannerState<'a>; 2],
    current_state_index: usize,
    segment_indentation: &'a str,
    segment_builder: Vec<&'a str>,
    done: bool,
}

impl<'a> SegmentsScanner<'a> {
    fn new(content: &'a str) -> Self {
        Self {
            content,
            cursor: 0,
            states: [
                ScannerState {
                    kind: StateKind::SegmentNotDefined,
                    segment: "",
                    targets: Vec::new(),
                },
                ScannerState {
                    kind: StateKind::SegmentNotDefined,
                    segment: "",
                    targets: Vec::new(),
                },
            ],
            current_state_index: 0,
            segment_indentation: "",
            segment_builder: Vec::new(),
            done: false,
        }
    }

    fn next_state_index(&self) -> usize {
        (self.current_state_index + 1) % 2
    }

    fn state(&self) -> &ScannerState {
        &self.states[self.current_state_index]
    }

    fn mut_state(&'a mut self) -> &mut ScannerState {
        &mut self.states[self.current_state_index]
    }

    fn prev_state(&self) -> &ScannerState {
        &self.states[self.next_state_index()]
    }

    fn set_state(&mut self, state: ScannerState<'a>) {
        self.current_state_index = self.next_state_index();
        self.states[self.current_state_index] = state;
    }

    fn dependencies(&mut self) -> Vec<&'a str> {
        let mut deps = Vec::new();
        let content = &self.content[self.cursor..];
        let mut start: i32 = -1;
        for (i, c) in content.char_indices() {
            if c == '\n' {
                if start != -1 {
                    deps.push(&content[start as usize..i]);
                }
                self.cursor += i + 1;
                return deps;
            }
            if c.is_whitespace() {
                // end of word
                if start != -1 {
                    deps.push(&content[start as usize..i]);
                    start = -1;
                }
                continue;
            }
            if start == -1 {
                start = i as i32;
            }
        }
        if start != -1 {
            deps.push(&content[start as usize..]);
        }
        self.cursor += content.len();
        deps
    }

    fn start_segment(&mut self) -> bool {
        let content = &self.content[self.cursor..];
        for (i, c) in content.char_indices() {
            if i == 0 && !c.is_alphabetic() {
                return false;
            }
            if c == '\n' {
                self.cursor += i + 1;
                return false;
            }
            if c == ':' {
                self.cursor += i + 1;
                let deps = self.dependencies();
                self.set_state(ScannerState {
                    kind: StateKind::SegmentStarts,
                    segment: &content[..i],
                    targets: deps,
                });
                return true;
            }
            if !c.is_alphanumeric() || c != '/' || c != '_' || c != '-' || c != '.' {
                return false;
            }
        }
        false
    }
}

#[cfg(test)]
mod segments_scanner_tests {
    use super::*;

    #[test]
    fn should_parse_simple_dependencies() {
        let mut scanner = SegmentsScanner::new("bar");
        let deps = scanner.dependencies();
        assert_eq!(deps, vec!["bar"]);
    }

    #[test]
    fn should_parse_multiple_dependencies() {
        let mut scanner = SegmentsScanner::new("foo  bar    baz");
        let deps = scanner.dependencies();
        assert_eq!(deps, vec!["foo", "bar", "baz"]);
    }

    #[test]
    fn should_parse_segments_till_newline() {
        let mut scanner = SegmentsScanner::new("foo\tbar\nbaz");
        let deps = scanner.dependencies();
        assert_eq!(deps, vec!["foo", "bar"]);
    }

    // TODO: Unicode tests
}

impl<'a> Iterator for SegmentsScanner<'a> {
    type Item = Node<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.state().kind {
            StateKind::SegmentNotDefined => {}
            StateKind::SegmentStarts => {}
            StateKind::SegmentContinued => {}
        }
        None
    }
}

fn make_nodes<'a>(lines: impl Iterator<Item = &'a str>) -> Vec<Node<'a>> {
    let mut nodes = Vec::new();
    nodes
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
