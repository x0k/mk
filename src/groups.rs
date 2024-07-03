use std::iter;

use super::chars::*;
use super::dependencies_collector::DependenciesCollector;
use super::node::Node;
use super::segments_scanner::SegmentsScanner;

#[derive(Debug, PartialEq)]
struct Position {
    start: usize,
    length: usize,
}

fn skip_line_and_find_group_start(content: &str, skip: usize) -> Option<Position> {
    let j = find_new_line_index(&content[skip..])?;
    let k = skip + j + 1;
    let Position { start, length } = find_group_start(&content[k..])?;
    Some(Position {
        start: k + start,
        length,
    })
}

fn find_group_start(content: &str) -> Option<Position> {
    let mut prev_is_slash = false;
    for (i, c) in content.char_indices() {
        // this line is not a segment/group name
        if (i == 0 && c.is_alphabetic()) || is_valid_segment_name_char(c) {
            prev_is_slash = c == '/';
            continue;
        }
        // group name should contain at least one char and end with slash
        if c != ':' || i < 2 || !prev_is_slash {
            return skip_line_and_find_group_start(content, i);
        }
        return Some(Position {
            start: 0,
            length: i,
        });
    }
    None
}

fn detect_group_indentation(content: &str) -> Option<&str> {
    let (i, _) = find_not_whitespace(content)?;
    Some(&content[..i])
}

fn get_group_len(content: &str, indentation: &str) -> usize {
    let mut shift = indentation.len();
    loop {
        let p = find_new_line_index(&content[shift..]);
        if p.is_none() {
            return content.len();
        }
        shift += p.unwrap() + 1;
        if !content[shift..].starts_with(indentation) {
            return shift;
        }
        shift += indentation.len();
    }
}

fn build_group_header(prefix: &str, content: &str, length: usize, deps: Vec<&str>) -> String {
    let d_list: String = if prefix.is_empty() {
        iter::once("")
            .chain(deps.into_iter())
            .map(|d| {
                if d.starts_with("/") {
                    return &d[1..];
                }
                d
            })
            .collect::<Vec<_>>()
            .join(" ")
    } else {
        deps.into_iter()
            .flat_map(|d| {
                if d.starts_with("/") {
                    vec![" ", d].into_iter()
                } else {
                    vec![" ", prefix, "/", d].into_iter()
                }
            })
            .collect::<Vec<_>>()
            .join("")
    };
    // omit root slashes
    return vec![&content[..length - 1], ":", d_list.as_str()].join("");
}

fn remove_parent_indentation(content: &str, indentation_len: usize) -> String {
    if indentation_len == 0 {
        return content.to_string();
    }
    content
        .lines()
        .map(|c| &c[indentation_len..])
        .collect::<Vec<_>>()
        .join("\n")
}

struct DesugaredNode {
    name: String,
    content: String,
    dependencies: Vec<String>,
}

fn desugar_groups(content: &str, prefix: &str) -> String {
    let group_start = find_group_start(content);
    if group_start.is_none() {
        return content.to_string();
    }
    let Position { start, length } = group_start.unwrap();
    let deps_start = start + length + 1;
    let (len, deps) = DependenciesCollector::new(&content[deps_start..]).collect();
    let group_content_start = deps_start + len + 1;
    // end of file (no group content)
    if group_content_start >= content.len() {
        return build_group_header(prefix, content, length, deps);
    }
    let group_indentation = detect_group_indentation(&content[group_content_start..]);
    // empty group
    if group_indentation.is_none() {
        return vec![
            build_group_header(prefix, content, length, deps).as_str(),
            "\n",
            desugar_groups(&content[group_content_start..], prefix).as_str(),
        ]
        .join("");
    }
    let group_indentation = group_indentation.unwrap();
    let group_content_len = get_group_len(&content[group_content_start..], group_indentation);
    let group_content_end = group_content_start + group_content_len;
    let group_name = &content[start..start + length - 1];
    let group_name_with_prefix = if prefix.is_empty() {
        group_name.to_owned()
    } else {
        format!("{prefix}/{group_name}")
    };
    let group_content = desugar_groups(
        remove_parent_indentation(
            &content[group_content_start..group_content_end],
            group_indentation.len(),
        )
        .as_str(),
        group_name_with_prefix.as_str(),
    );
    let segments = iter::once(content[..start].to_string()).chain(
        SegmentsScanner::new(group_content.as_str())
            .map(|node| match node {
                Node::Content(content) => {
                    let lines = content.lines().map(|l| format!("{group_indentation}{l}"));
                    DesugaredNode {
                        name: group_name.to_string(),
                        content: if content.ends_with("\n") {
                            lines
                                .chain(iter::once("".to_string()))
                                .collect::<Vec<_>>()
                                .join("\n")
                        } else {
                            lines.collect::<Vec<_>>().join("\n")
                        },
                        dependencies: iter::once("".to_string())
                            .chain(deps.iter().map(|d| d.to_string()))
                            .collect(),
                    }
                }
                Node::Segment {
                    name,
                    content,
                    dependencies,
                    ..
                } => DesugaredNode {
                    name: format!("{group_name}/{name}"),
                    content: content.to_string(),
                    dependencies: iter::once("".to_string())
                        .chain(iter::once(group_name.to_string()))
                        .chain(dependencies.into_iter().map(|d| {
                            if d.starts_with("/") {
                                if prefix.is_empty() {
                                    d[1..].to_string()
                                } else {
                                    d.to_string()
                                }
                            } else {
                                format!("{group_name}/{d}")
                            }
                        }))
                        .collect(),
                },
            })
            .map(
                |DesugaredNode {
                     name,
                     content,
                     dependencies,
                 }| format!("{}:{}\n{}", name, dependencies.join(" "), content),
            ),
    );
    if group_content_end < content.len() {
        segments
            .chain(iter::once("\n".to_string()))
            .chain(iter::once(desugar_groups(
                &content[group_content_end..],
                prefix,
            )))
            .collect::<Vec<_>>()
            .join("")
    } else {
        segments.collect::<Vec<_>>().join("")
    }
}

pub fn desugar(content: &str) -> String {
    desugar_groups(content, "")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_find_group_start() {
        assert_eq!(
            find_group_start("group/:"),
            Some(Position {
                start: 0,
                length: 6
            })
        );
        assert_eq!(
            find_group_start("skip\ngroup/:"),
            Some(Position {
                start: 5,
                length: 6
            })
        );
        assert_eq!(
            find_group_start("skip: this\n\tcontent\ngroup/:"),
            Some(Position {
                start: 20,
                length: 6
            })
        );
        assert_eq!(
            find_group_start("\ngroup/:"),
            Some(Position {
                start: 1,
                length: 6
            })
        );
    }

    #[test]
    fn should_not_find_group_start() {
        assert_eq!(find_group_start("segment:"), None);
        assert_eq!(find_group_start("not/group:"), None);
        assert_eq!(find_group_start("invalid/\n"), None);
    }

    #[test]
    fn should_desugar_empty_group() {
        assert_eq!(desugar("group/: dep /root-dep"), "group: dep root-dep")
    }

    #[test]
    fn should_desugar_simple_group() {
        assert_eq!(desugar("group/:\n\tcontent"), "group:\n\tcontent")
    }

    #[test]
    fn should_desugar_group() {
        assert_eq!(
            desugar(
                "
group/:
    pushd folder
    
    bar: /foo
        bar content
        
    baz: bar
        baz content
        
    popd"
            ),
            "
group:
    pushd folder
    
group/bar: group foo
    bar content
    
group/baz: group group/bar
    baz content
    
group:
    popd"
        )
    }

    #[test]
    fn should_preserve_newlines_between_group_segments() {
        assert_eq!(
            desugar(
                "# Artifacts
a/:
  go/:
    pushd packages/testing-go/go
    build:
      GOOS=js GOARCH=wasm go build -o ../public/compiler.wasm cmd/compiler/main.go
    popd
  build: go/build
"
            ),
            "# Artifacts
a/go: a
  pushd packages/testing-go/go
a/go/build: a a/go
  GOOS=js GOARCH=wasm go build -o ../public/compiler.wasm cmd/compiler/main.go
a/go: a
  popd
a/build: a a/go/build
"
        );
    }

    #[test]
    fn should_not_panic_while_desugar_file() {
        let content = include_str!("testdata/mkfile.test");
        let result = std::panic::catch_unwind(|| {
            desugar(content);
        });
        assert!(result.is_ok(), "should not panic");
    }
}
