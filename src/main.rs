use std::collections::{HashMap, HashSet};

#[derive(Debug, PartialEq)]
enum Node<'a> {
    Content(&'a str),
    Segment {
        name: &'a str,
        content: &'a str,
        indentation: &'a str,
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
            indentation: "",
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
                indentation: "",
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
}

#[derive(Debug, PartialEq, Eq)]
enum StateKind {
    SegmentNotDefined,
    SegmentStarts,
    SegmentContinued,
}

#[derive(Debug)]
struct ScannerState<'a> {
    kind: StateKind,
    segment: &'a str,
    dependencies: Vec<&'a str>,
    content_start_position: usize,
}

struct DependenciesCollector<'a> {
    content: &'a str,
    dependencies: Vec<&'a str>,
    word_begin: isize,
}

impl<'a> DependenciesCollector<'a> {
    fn new(content: &'a str) -> Self {
        Self {
            content,
            word_begin: -1,
            dependencies: Vec::new(),
        }
    }

    fn start_word_if_not_started(&mut self, start: usize) {
        if self.word_begin == -1 {
            self.word_begin = start as isize;
        }
    }

    fn collect_word_if_started(&mut self, end: usize) {
        if self.word_begin != -1 {
            self.dependencies
                .push(&self.content[self.word_begin as usize..end]);
            self.word_begin = -1;
        }
    }

    fn collect(&mut self) -> usize {
        for (i, c) in self.content.char_indices() {
            if c == '\n' {
                self.collect_word_if_started(i);
                return i + 1;
            }
            if c.is_whitespace() {
                self.collect_word_if_started(i);
                continue;
            }
            self.start_word_if_not_started(i);
        }
        let l = self.content.len();
        self.collect_word_if_started(l);
        return l;
    }
}

fn is_new_line(&(_, c): &(usize, char)) -> bool {
    c == '\n'
}

fn is_not_whitespace(&(_, c): &(usize, char)) -> bool {
    !c.is_whitespace()
}

fn find_new_line(content: &str) -> Option<(usize, char)> {
    content.char_indices().find(is_new_line)
}

#[derive(Debug)]
struct SegmentsScanner<'a> {
    content: &'a str,
    cursor: usize,
    states: [ScannerState<'a>; 2],
    current_state_index: usize,
    segment_indentation: &'a str,
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
                    dependencies: Vec::new(),
                    content_start_position: 0,
                },
                ScannerState {
                    kind: StateKind::SegmentNotDefined,
                    segment: "",
                    dependencies: Vec::new(),
                    content_start_position: 0,
                },
            ],
            current_state_index: 0,
            segment_indentation: "",
        }
    }

    fn next_state_index(&self) -> usize {
        (self.current_state_index + 1) % 2
    }

    fn done(&self) -> bool {
        self.cursor > self.content.len()
    }

    fn state(&self) -> &ScannerState<'a> {
        &self.states[self.current_state_index]
    }

    fn prev_state(&self) -> &ScannerState<'a> {
        &self.states[self.next_state_index()]
    }

    fn set_state(&mut self, state: ScannerState<'a>) {
        self.current_state_index = self.next_state_index();
        self.states[self.current_state_index] = state;
    }

    fn dependencies(&mut self) -> Vec<&'a str> {
        let mut collector = DependenciesCollector::new(&self.content[self.cursor..]);
        self.cursor += collector.collect();
        collector.dependencies
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
                let dependencies = self.dependencies();
                let content_start_position = self.cursor;
                self.set_state(ScannerState {
                    kind: StateKind::SegmentStarts,
                    segment: &content[..i],
                    dependencies,
                    content_start_position,
                });
                return true;
            }
            if !(c.is_alphanumeric() || c == '/' || c == '_' || c == '-' || c == '.') {
                self.cursor += i + 1;
                return false;
            }
        }
        self.cursor += content.len() + 1;
        false
    }

    fn continue_segment(&mut self) -> bool {
        let content = &self.content[self.cursor..];
        let p = content.char_indices().find(is_not_whitespace);
        if p.is_none() {
            self.cursor += content.len() + 1;
            return false;
        }
        let (i, _) = p.unwrap();
        // First char is not a whitespace
        // Do not advance cursor here, to try start a new segment
        if i == 0 {
            return false;
        }
        self.states[self.current_state_index].kind = StateKind::SegmentContinued;
        self.segment_indentation = &content[..i];
        if let Some(p) = find_new_line(&content[i..]) {
            self.cursor += i + p.0 + 1;
        } else {
            self.cursor += content.len() + 1;
        }
        true
    }

    fn finish_segment(&mut self, content_start_position: usize) {
        self.set_state(ScannerState {
            kind: StateKind::SegmentNotDefined,
            segment: "",
            dependencies: Vec::new(),
            content_start_position,
        });
    }

    fn complete_segment(&mut self) {
        while !self.done() {
            let content = &self.content[self.cursor..];
            if !content.starts_with(self.segment_indentation) {
                break;
            }
            if let Some(p) = find_new_line(content) {
                self.cursor += p.0 + 1;
            } else {
                self.cursor += content.len() + 1;
            }
        }
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

    #[test]
    fn should_start_simple_segment() {
        let mut scanner = SegmentsScanner::new("foo:");
        assert_eq!(scanner.start_segment(), true);
        assert_eq!(scanner.state().kind, StateKind::SegmentStarts);
        assert_eq!(scanner.state().segment, "foo");
        assert_eq!(scanner.state().dependencies.len(), 0);
    }

    #[test]
    fn should_start_segment_with_dependencies() {
        let mut scanner = SegmentsScanner::new("foo: bar\nbaz");
        assert_eq!(scanner.start_segment(), true);
        assert_eq!(scanner.state().kind, StateKind::SegmentStarts);
        assert_eq!(scanner.state().segment, "foo");
        let deps = &scanner.state().dependencies;
        assert!(deps.len() == 1 && deps.contains(&"bar"));
    }

    #[test]
    fn should_detect_indentation() {
        let mut scanner = SegmentsScanner::new("  content");
        assert_eq!(scanner.continue_segment(), true);
        assert_eq!(scanner.segment_indentation, "  ");
    }

    #[test]
    fn should_ignore_whitespace_tail() {
        let mut scanner = SegmentsScanner::new("\t\t  \t\t");
        assert_eq!(scanner.continue_segment(), false);
    }
}

impl<'a> Iterator for SegmentsScanner<'a> {
    type Item = Node<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.done() {
            return None;
        }
        loop {
            let initial_cursor = self.cursor;
            match self.state().kind {
                StateKind::SegmentNotDefined => {
                    if self.start_segment() {
                        let s = self.prev_state().content_start_position;
                        if initial_cursor - s > 0 {
                            return Some(Node::Content(&self.content[s..initial_cursor]));
                        }
                    }
                }
                // TODO: Merge with `SegmentContinued` and rename to `SegmentDefined`
                StateKind::SegmentStarts => {
                    if !self.continue_segment() {
                        let segment = Node::Segment {
                            name: self.state().segment,
                            dependencies: self.state().dependencies.clone(),
                            indentation: "",
                            content: "",
                        };
                        self.finish_segment(initial_cursor);
                        return Some(segment);
                    }
                }
                StateKind::SegmentContinued => {
                    self.complete_segment();
                    let segment_end = self.cursor;
                    let segment = Node::Segment {
                        name: self.state().segment,
                        dependencies: self.state().dependencies.clone(),
                        indentation: self.segment_indentation,
                        content: &self.content[self.state().content_start_position..segment_end],
                    };
                    self.finish_segment(segment_end);
                    return Some(segment);
                }
            }
            if self.done() {
                if self.state().kind == StateKind::SegmentNotDefined {
                    return Some(Node::Content(&self.content[initial_cursor..]));
                }
                return Some(Node::Segment {
                    name: self.state().segment,
                    content: &self.content[self.state().content_start_position..],
                    dependencies: self.state().dependencies.clone(),
                    indentation: self.segment_indentation,
                });
            }
        }
    }
}

#[cfg(test)]
mod segments_scanners_iterator_tests {
    use super::*;

    // For debug purposes
    fn collect<'a>(scanner: SegmentsScanner<'a>) -> Vec<Node<'a>> {
        let mut nodes = Vec::new();
        for node in scanner {
            nodes.push(node);
        }
        nodes
    }

    #[test]
    fn should_emit_simple_content() {
        let scanner = SegmentsScanner::new("content");
        assert!(collect(scanner) == vec![Node::Content("content")]);
    }

    #[test]
    fn should_emit_simple_segment() {
        let scanner = SegmentsScanner::new("foo:\n\tcontent");
        assert!(
            collect(scanner)
                == vec![Node::Segment {
                    name: "foo",
                    content: "\tcontent",
                    dependencies: Vec::new(),
                    indentation: "\t",
                }]
        );
    }

    #[test]
    fn should_emit_empty_content() {
        let scanner = SegmentsScanner::new("");
        assert!(collect(scanner) == vec![Node::Content("")]);
    }

    #[test]
    fn should_emit_simple_content_and_segment() {
        let scanner = SegmentsScanner::new("content\nfoo:\n\tcontent");
        let collected = collect(scanner);
        assert!(
            collected
                == vec![
                    Node::Content("content\n"),
                    Node::Segment {
                        name: "foo",
                        content: "\tcontent",
                        dependencies: Vec::new(),
                        indentation: "\t",
                    }
                ]
        );
    }

    #[test]
    fn should_emit_content_and_segments() {
        let scanner = SegmentsScanner::new("content\nfoo:\n\tfoo 1\n\tfoo 2\ncommon");
        let collected = collect(scanner);
        println!("{:?}", collected);
        assert!(
            collected
                == vec![
                    Node::Content("content\n"),
                    Node::Segment {
                        name: "foo",
                        content: "\tfoo 1\n\tfoo 2\n",
                        dependencies: Vec::new(),
                        indentation: "\t",
                    },
                    Node::Content("common")
                ]
        );
    }

    #[test]
    fn should_emit_empty_segment() {
        let scanner = SegmentsScanner::new("common\nfoo:\nbar:\nbaz");
        assert!(
            collect(scanner)
                == vec![
                    Node::Content("common\n"),
                    Node::Segment {
                        name: "foo",
                        content: "",
                        dependencies: Vec::new(),
                        indentation: "",
                    },
                    Node::Segment {
                        name: "bar",
                        content: "",
                        dependencies: Vec::new(),
                        indentation: "",
                    },
                    Node::Content("baz")
                ]
        );
    }

    #[test]
    fn should_emit_multiple_segments() {
        let scanner =
            SegmentsScanner::new("common\nfoo:\n\tfoo content\nbar:\n\tbar content\ncommon");
        assert!(
            collect(scanner)
                == vec![
                    Node::Content("common\n"),
                    Node::Segment {
                        name: "foo",
                        content: "\tfoo content\n",
                        dependencies: Vec::new(),
                        indentation: "\t",
                    },
                    Node::Segment {
                        name: "bar",
                        content: "\tbar content\n",
                        dependencies: Vec::new(),
                        indentation: "\t",
                    },
                    Node::Content("common")
                ]
        );
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
