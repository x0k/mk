use super::chars::*;
use super::dependencies_collector::DependenciesCollector;

fn slip_line_and_find_group_start(
    content: &str,
    level: usize,
    skip: usize,
) -> Option<(usize, usize)> {
    find_new_line(&content[skip..]).and_then(|j| {
        let k = skip + j + 1;
        find_group_start(&content[k..], level).and_then(|(s, l)| Some((k + s, l)))
    })
}

fn find_group_start(content: &str, level: usize) -> Option<(usize, usize)> {
    let mut prev_is_slash = false;
    for (i, c) in content.char_indices().skip_while(|&(i, _)| i < level) {
        // this line is not a segment/group name
        if i == 0 && !c.is_alphabetic() {
            return slip_line_and_find_group_start(content, level, 1);
        }
        if is_valid_segment_name_char(c) {
            prev_is_slash = c == '/';
            continue;
        }
        // group name should contain at least one char and end with slash
        if c != ':' || i < 2 || !prev_is_slash {
            return slip_line_and_find_group_start(content, level, i);
        }
        return Some((0, i));
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
        let p = find_new_line(&content[shift..]);
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

fn build_group_header(
    level: usize,
    content: &str,
    start: usize,
    length: usize,
    deps: Vec<&str>,
) -> String {
    // omit root slashes
    let deps_list = if level == 0 {
        deps.into_iter()
            .map(|s| {
                if s.starts_with("/") {
                    return &s[1..];
                }
                s
            })
            .collect::<Vec<_>>()
            .join(" ")
    } else {
        deps.join(" ")
    };
    return vec![
        &content[..start],
        // omit last slash
        &content[start..length - 1],
        ": ",
        deps_list.as_str(),
    ]
    .join("");
}

fn remove_root_indentation(content: &str, level: usize) -> String {
    if level == 0 {
        return content.to_string();
    }
    content
        .lines()
        .map(|c| &c[level..])
        .collect::<Vec<_>>()
        .join("\n")
}

fn desugar_group(content: &str, level: usize) -> String {
    let group_start = find_group_start(content, level);
    if group_start.is_none() {
        return remove_root_indentation(content, level);
    }
    let (start, length) = group_start.unwrap();
    let deps_start = start + length + 1;
    let (len, deps) = DependenciesCollector::new(&content[deps_start..]).collect();
    let group_content_start = deps_start + len + 1;
    if group_content_start >= content.len() {
        return build_group_header(level, content, start, length, deps);
    }
    let group_indentation = detect_group_indentation(&content[group_content_start..]);
    // empty group
    if group_indentation.is_none() {
        return vec![
            build_group_header(level, content, start, length, deps).as_str(),
            "\n",
            desugar_group(&content[group_content_start..], level).as_str(),
        ]
        .join("");
    }
    let group_indentation = group_indentation.unwrap();
    let group_content_len = get_group_len(&content[group_content_start..], group_indentation);
    return content.to_string();
}

pub fn desugar(content: &str) -> String {
    desugar_group(content, 0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_find_group_start() {
        assert_eq!(find_group_start("group/:", 0), Some((0, 6)));
        assert_eq!(find_group_start("skip\ngroup/:", 0), Some((5, 6)));
        assert_eq!(
            find_group_start("skip: this\n\tcontent\ngroup/:", 0),
            Some((20, 6))
        );
        assert_eq!(find_group_start("  group/:", 2), Some((0, 8)));
    }

    #[test]
    fn should_not_find_group_start() {
        assert_eq!(find_group_start("segment:", 0), None);
        assert_eq!(find_group_start("not/group:", 0), None);
        assert_eq!(find_group_start("invalid/\n", 0), None);
    }

    #[test]
    fn should_desugar_empty_group() {
        assert_eq!(desugar("group/: dep /root-dep"), "group: dep root-dep")
    }
}
