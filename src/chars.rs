pub fn is_new_line(&(_, c): &(usize, char)) -> bool {
    c == '\n'
}

pub fn find_new_line_index(content: &str) -> Option<usize> {
    content.char_indices().find(is_new_line).and_then(|(i, _)| Some(i))
}

pub fn is_not_whitespace(&(_, c): &(usize, char)) -> bool {
    !c.is_whitespace()
}

pub fn find_not_whitespace(content: &str) -> Option<(usize, char)> {
    content.char_indices().find(is_not_whitespace)
}

pub fn is_valid_segment_name_char(char: char) -> bool {
    char.is_alphanumeric() || char == '/' || char == '_' || char == '-' || char == '.'
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_find_new_line() {
        
        assert_eq!(find_new_line_index("\ncontent"), Some(0));
    }
}
