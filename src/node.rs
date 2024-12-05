#[derive(Debug, PartialEq)]
pub enum Node<'a> {
    Content(&'a str),
    Segment {
        name: &'a str,
        content: &'a str,
        indentation: &'a str,
        dependencies: Vec<&'a str>,
    },
}

impl<'a> Node<'a> {
    pub fn description(&self) -> Option<Vec<&'a str>> {
        match self {
            Node::Content(_) => None,
            Node::Segment { content, .. } => {
                if content.is_empty() {
                    return None;
                }
                let mut lines = Vec::new();
                let mut hash_pos = None;
                for (i, c) in content.char_indices() {
                    if c == '#' {
                        if hash_pos.is_none() {
                            hash_pos = Some(i);
                        }
                    } else if c == '\n' {
                        if let Some(h) = hash_pos {
                            lines.push(&content[h + 1..i]);
                            hash_pos = None;
                            continue;
                        }
                        break;
                    }
                }
                if let Some(p) = hash_pos {
                    lines.push(&content[p + 1..]);
                }
                if lines.is_empty() {
                    return None;
                }
                Some(lines)
            }
        }
    }
}
