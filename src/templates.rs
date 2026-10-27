use std::collections::HashMap;
use std::fmt;

#[derive(Clone, PartialEq, Eq)]
pub struct Error {
    line: usize,
    message: String,
}

impl Error {
    fn new(line: usize, message: impl Into<String>) -> Self {
        Self {
            line,
            message: message.into(),
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "line {}: {}", self.line, self.message)
    }
}

impl fmt::Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}

impl std::error::Error for Error {}

#[derive(Debug, Clone)]
struct Template {
    param: Option<String>,
    body: String,
    line_map: Vec<usize>,
}

struct Collected {
    content: String,
    line_map: Vec<usize>,
    templates: HashMap<String, Template>,
}

pub fn expand(content: &str) -> Result<String, Error> {
    let Collected {
        content,
        line_map,
        templates,
    } = collect_templates(content)?;
    expand_uses(&content, &line_map, &templates, &mut Vec::new())
}

fn collect_templates(content: &str) -> Result<Collected, Error> {
    let lines: Vec<&str> = content.split_inclusive('\n').collect();
    let mut templates = HashMap::new();
    let mut output = String::new();
    let mut output_line_map = Vec::new();
    let mut i = 0;

    while i < lines.len() {
        let line = lines[i];
        let line_without_new_line = without_new_line(line);
        let trimmed = line_without_new_line.trim_start();

        if starts_with_directive(trimmed, "@template").is_none() {
            output.push_str(line);
            output_line_map.push(i + 1);
            i += 1;
            continue;
        }

        let (name, param) = parse_template_header(trimmed, i + 1)?;
        if templates.contains_key(&name) {
            return Err(Error::new(
                i + 1,
                format!("template '{}' is already defined", name),
            ));
        }

        let definition_indent = leading_whitespace(line_without_new_line);
        let definition_indent_len = definition_indent.len();
        let mut body = String::new();
        let mut body_line_map = Vec::new();
        let mut body_indent: Option<&str> = None;
        let mut body_started = false;
        let mut pending_blank_lines: Vec<(usize, &str)> = Vec::new();
        let mut j = i + 1;

        while j < lines.len() {
            let body_line = lines[j];
            let body_content = without_new_line(body_line);

            if body_content.trim().is_empty() {
                pending_blank_lines.push((j + 1, body_line));
                j += 1;
                continue;
            }

            let line_indent = leading_whitespace(body_content);
            if line_indent.len() <= definition_indent_len {
                break;
            }

            if starts_with_directive(body_content.trim_start(), "@template").is_some() {
                return Err(Error::new(j + 1, "template definitions cannot be nested"));
            }

            if body_started {
                for (line_number, blank_line) in pending_blank_lines.drain(..) {
                    body.push_str(blank_line);
                    body_line_map.push(line_number);
                }
            }
            pending_blank_lines.clear();
            body_started = true;

            let indent = *body_indent.get_or_insert(line_indent);
            let dedented = body_content
                .strip_prefix(indent)
                .ok_or_else(|| Error::new(j + 1, "inconsistent template body indentation"))?;

            body.push_str(dedented);
            if body_line.ends_with('\n') {
                body.push('\n');
            }
            body_line_map.push(j + 1);
            j += 1;
        }

        for (line_number, blank_line) in pending_blank_lines.drain(..) {
            output.push_str(blank_line);
            output_line_map.push(line_number);
        }

        templates.insert(
            name,
            Template {
                param,
                body,
                line_map: body_line_map,
            },
        );
        i = j;
    }

    Ok(Collected {
        content: output,
        line_map: output_line_map,
        templates,
    })
}

fn expand_uses(
    content: &str,
    line_map: &[usize],
    templates: &HashMap<String, Template>,
    stack: &mut Vec<String>,
) -> Result<String, Error> {
    let lines: Vec<&str> = content.split_inclusive('\n').collect();
    let mut output = String::new();

    for (index, line) in lines.iter().enumerate() {
        let line_without_new_line = without_new_line(line);
        let trimmed = line_without_new_line.trim_start();
        if starts_with_directive(trimmed, "@use").is_none() {
            output.push_str(line);
            continue;
        }

        let line_number = line_map.get(index).copied().unwrap_or(index + 1);
        let (name, value) = parse_use_header(trimmed, line_number)?;
        let template = templates.get(&name).ok_or_else(|| {
            Error::new(line_number, format!("template '{}' is not defined", name))
        })?;

        if stack.contains(&name) {
            let mut chain = stack.clone();
            chain.push(name.clone());
            return Err(Error::new(
                line_number,
                format!("cyclic template use: {}", chain.join(" -> ")),
            ));
        }

        if template.param.is_none() && !value.is_empty() {
            return Err(Error::new(
                line_number,
                format!("template '{}' does not accept an argument", name),
            ));
        }

        let mut body = if let Some(param) = template.param.as_deref() {
            template.body.replace(param, &value)
        } else {
            template.body.clone()
        };
        body = reindent(&body, leading_whitespace(line_without_new_line));

        stack.push(name);
        let expanded = expand_uses(&body, &template.line_map, templates, stack);
        stack.pop();
        let mut expanded = expanded?;

        if line.ends_with('\n') {
            if !expanded.ends_with('\n') {
                expanded.push('\n');
            }
        } else if expanded.ends_with('\n') {
            expanded.pop();
        }
        output.push_str(&expanded);
    }

    Ok(output)
}

fn parse_template_header(trimmed: &str, line: usize) -> Result<(String, Option<String>), Error> {
    let rest = starts_with_directive(trimmed, "@template")
        .ok_or_else(|| Error::new(line, "invalid template definition"))?;
    let rest = rest.trim_end();
    let rest = rest
        .strip_suffix(':')
        .ok_or_else(|| Error::new(line, "template definition must end with ':'"))?;
    let tokens: Vec<&str> = rest.split_whitespace().collect();

    if tokens.is_empty() {
        return Err(Error::new(line, "template name is missing"));
    }

    let name = tokens[0];
    let params = &tokens[1..];

    if name.contains('(')
        || name.contains(')')
        || params
            .iter()
            .any(|param| param.contains('(') || param.contains(')'))
    {
        return Err(Error::new(
            line,
            "parenthesized template parameters are reserved for multi-argument templates",
        ));
    }

    if params.len() > 1 {
        return Err(Error::new(
            line,
            format!("template '{}' supports at most one parameter", name),
        ));
    }

    if !is_identifier(name) {
        return Err(Error::new(
            line,
            format!("invalid template name '{}'", name),
        ));
    }

    let param = match params.first() {
        Some(param) if is_identifier(param) => Some((*param).to_string()),
        Some(param) => {
            return Err(Error::new(
                line,
                format!("invalid template parameter name '{}'", param),
            ));
        }
        None => None,
    };

    Ok((name.to_string(), param))
}

fn parse_use_header(trimmed: &str, line: usize) -> Result<(String, String), Error> {
    let rest = starts_with_directive(trimmed, "@use")
        .ok_or_else(|| Error::new(line, "invalid template use"))?;
    let rest = rest.trim_start();

    if rest.is_empty() {
        return Err(Error::new(line, "template name is missing"));
    }

    let name_end = rest
        .char_indices()
        .find_map(|(index, c)| c.is_whitespace().then_some(index))
        .unwrap_or(rest.len());
    let name = &rest[..name_end];

    if name.contains('(') || name.contains(')') {
        return Err(Error::new(
            line,
            "parenthesized template arguments are reserved for multi-argument templates",
        ));
    }

    if !is_identifier(name) {
        return Err(Error::new(
            line,
            format!("invalid template name '{}'", name),
        ));
    }

    let value = rest[name_end..].trim().to_string();
    Ok((name.to_string(), value))
}

fn starts_with_directive<'a>(content: &'a str, directive: &str) -> Option<&'a str> {
    let rest = content.strip_prefix(directive)?;
    match rest.chars().next() {
        None => Some(rest),
        Some(c) if c.is_whitespace() || c == ':' => Some(rest),
        Some(_) => None,
    }
}

fn is_identifier(value: &str) -> bool {
    let mut chars = value.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    if !first.is_alphabetic() && first != '_' {
        return false;
    }
    chars.all(|c| c.is_alphanumeric() || c == '_')
}

fn without_new_line(line: &str) -> &str {
    line.strip_suffix('\n').unwrap_or(line)
}

fn leading_whitespace(line: &str) -> &str {
    let len = line
        .char_indices()
        .take_while(|(_, c)| c.is_whitespace())
        .map(|(index, c)| index + c.len_utf8())
        .last()
        .unwrap_or(0);
    &line[..len]
}

fn reindent(content: &str, indentation: &str) -> String {
    let mut result = String::new();
    for line in content.split_inclusive('\n') {
        let has_new_line = line.ends_with('\n');
        let line_without_new_line = without_new_line(line);
        if line_without_new_line.trim().is_empty() {
            result.push_str(line);
            continue;
        }
        result.push_str(indentation);
        result.push_str(line_without_new_line);
        if has_new_line {
            result.push('\n');
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_expand_simple_template() {
        assert_eq!(
            expand("@template echo_cmd cmd:\n    echo cmd\nx:\n    @use echo_cmd hello\n"),
            Ok("x:\n    echo hello\n".to_string())
        );
    }

    #[test]
    fn should_pass_raw_value_with_spaces_and_quotes() {
        assert_eq!(
            expand("@template echo_cmd cmd:\n    echo cmd\n@use echo_cmd hello \"world\"\n"),
            Ok("echo hello \"world\"\n".to_string())
        );
    }

    #[test]
    fn should_pass_raw_value_with_parentheses() {
        assert_eq!(
            expand("@template run cmd:\n    cmd\n@use run echo \"(hello)\"\n"),
            Ok("echo \"(hello)\"\n".to_string())
        );
    }

    #[test]
    fn should_expand_zero_parameter_template() {
        assert_eq!(
            expand("@template tasks:\n    c:\n        pnpm run check\nx:\n    @use tasks\n"),
            Ok("x:\n    c:\n        pnpm run check\n".to_string())
        );
    }

    #[test]
    fn should_expand_nested_templates() {
        let content = "\
@template tasks:
    c:
        pnpm run check
@template theme theme_dir:
    pushd theme_dir
    @use tasks
    popd
basic/:
    @use theme packages/basic-theme
";
        assert_eq!(
            expand(content),
            Ok("\
basic/:
    pushd packages/basic-theme
    c:
        pnpm run check
    popd
"
            .to_string())
        );
    }

    #[test]
    fn should_forward_raw_value_to_nested_template() {
        let content = "\
@template inner value:
    echo value
@template outer value:
    @use inner value
@use outer hello world
";
        assert_eq!(expand(content), Ok("echo hello world\n".to_string()));
    }

    #[test]
    fn should_allow_template_defined_after_use() {
        assert_eq!(
            expand("@use later\n@template later:\n    echo later\n"),
            Ok("echo later\n".to_string())
        );
    }

    #[test]
    fn should_reject_parenthesized_template_definition() {
        let error = expand("@template theme(theme_dir):\n    echo theme_dir\n").unwrap_err();
        assert!(error.to_string().contains("reserved"));
    }

    #[test]
    fn should_reject_parenthesized_template_use() {
        let error = expand("@use theme(packages/basic-theme)\n").unwrap_err();
        assert!(error.to_string().contains("reserved"));
    }

    #[test]
    fn should_reject_multiple_parameters() {
        let error =
            expand("@template pnpm package_dir package_cmd:\n    echo package_dir\n").unwrap_err();
        assert!(error.to_string().contains("at most one parameter"));
    }

    #[test]
    fn should_reject_nested_template_definition() {
        let error =
            expand("@template outer value:\n    @template inner nested:\n        echo nested\n")
                .unwrap_err();
        assert!(error.to_string().contains("cannot be nested"));
    }

    #[test]
    fn should_reject_unknown_template() {
        let error = expand("@use missing\n").unwrap_err();
        assert!(error.to_string().contains("not defined"));
    }

    #[test]
    fn should_reject_argument_for_zero_parameter_template() {
        let error = expand("@template tasks:\n    echo\n@use tasks argument\n").unwrap_err();
        assert!(error.to_string().contains("does not accept an argument"));
    }

    #[test]
    fn should_detect_cyclic_template_use() {
        let error = expand(
            "\
@template first value:
    @use second value
@template second value:
    @use first value
@use first hello
",
        )
        .unwrap_err();
        assert!(error.to_string().contains("cyclic template use"));
    }

    #[test]
    fn should_preserve_blank_lines_after_template_body() {
        assert_eq!(
            expand("@template echo_cmd cmd:\n\n    echo cmd\n\n@use echo_cmd hello\n"),
            Ok("\necho hello\n".to_string())
        );
    }

    #[test]
    fn should_report_source_line_for_error_inside_template_body() {
        let error =
            expand("@template outer:\n    line1\n    @use missing\n@use outer\n").unwrap_err();
        assert_eq!(
            error.to_string(),
            "line 3: template 'missing' is not defined"
        );
    }

    #[test]
    fn should_preserve_crlf_blank_lines_inside_template_body() {
        assert_eq!(
            expand("@template t:\r\n    echo a\r\n\r\n    echo b\r\n@use t\r\n"),
            Ok("echo a\r\n\r\necho b\r\n".to_string())
        );
    }

    #[test]
    fn should_allow_empty_value_for_one_parameter_template() {
        assert_eq!(
            expand("@template echo_cmd cmd:\n    echo cmd\n@use echo_cmd\n"),
            Ok("echo \n".to_string())
        );
    }
}
