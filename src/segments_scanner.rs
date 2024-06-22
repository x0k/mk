use super::chars::*;
use super::dependencies_collector::DependenciesCollector;
use super::node::Node;

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

#[derive(Debug)]
pub struct SegmentsScanner<'a> {
    content: &'a str,
    cursor: usize,
    states: [ScannerState<'a>; 2],
    current_state_index: usize,
    segment_indentation: &'a str,
}

impl<'a> SegmentsScanner<'a> {
    pub fn new(content: &'a str) -> Self {
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
        let (len, dependencies) =
            DependenciesCollector::new(&self.content[self.cursor..]).collect();
        self.cursor += len + 1;
        dependencies
    }

    fn start_segment(&mut self) -> bool {
        let content = &self.content[self.cursor..];
        for (i, c) in content.char_indices() {
            if i == 0 && !c.is_alphabetic() {
                self.cursor += 1;
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
            if !is_valid_segment_name_char(c) {
                self.cursor += i + 1;
                return false;
            }
        }
        self.cursor += content.len() + 1;
        false
    }

    fn continue_segment(&mut self) -> bool {
        let content = &self.content[self.cursor..];
        let p = find_not_whitespace(content);
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
        if let Some(p) = find_new_line_index(&content[i..]) {
            self.cursor += i + p + 1;
        } else {
            self.cursor += content.len() + 1;
        }
        true
    }

    fn complete_segment(&mut self) {
        while !self.done() {
            let content = &self.content[self.cursor..];
            if !content.starts_with(self.segment_indentation) {
                break;
            }
            if let Some(p) = find_new_line_index(content) {
                self.cursor += p + 1;
            } else {
                self.cursor += content.len() + 1;
            }
        }
    }

    fn finish_segment(&mut self, content_start_position: usize) {
        self.set_state(ScannerState {
            kind: StateKind::SegmentNotDefined,
            segment: "",
            dependencies: Vec::new(),
            content_start_position,
        });
    }
}

#[cfg(test)]
mod tests {
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
mod iterator_tests {
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

    #[test]
    fn should_emit_multiple_segments2() {
        let scanner = SegmentsScanner::new(
            "pushd folder

bar: /foo
    bar content
    
baz: bar
    baz content
    
popd",
        );
        assert!(
            collect(scanner)
                == vec![
                    Node::Content("pushd folder\n\n"),
                    Node::Segment {
                        name: "bar",
                        content: "    bar content\n    \n",
                        dependencies: vec!["/foo"],
                        indentation: "    ",
                    },
                    Node::Segment {
                        name: "baz",
                        content: "    baz content\n    \n",
                        dependencies: vec!["bar"],
                        indentation: "    ",
                    },
                    Node::Content("popd")
                ]
        )
    }
}
