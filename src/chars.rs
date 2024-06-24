use std::collections::HashSet;

use once_cell::sync::Lazy;

pub fn is_new_line(&(_, c): &(usize, char)) -> bool {
    c == '\n'
}

pub fn find_new_line_index(content: &str) -> Option<usize> {
    content
        .char_indices()
        .find(is_new_line)
        .and_then(|(i, _)| Some(i))
}

pub fn is_not_whitespace(&(_, c): &(usize, char)) -> bool {
    !c.is_whitespace()
}

pub fn find_not_whitespace(content: &str) -> Option<(usize, char)> {
    content.char_indices().find(is_not_whitespace)
}

static GLOB_PATTERN_SYMBOLS: Lazy<HashSet<char>> = Lazy::new(|| {
    let mut m = HashSet::new();
    m.insert('*');
    m.insert('!');
    m.insert('?');
    m.insert('[');
    m.insert(']');
    // contains in `ALLOWED_SYMBOLS` and disabled here
    // for proper pattern detection
    // m.insert('-');
    m
});

pub fn contains_glob_pattern_symbols(content: &str) -> bool {
    content.chars().any(|c| GLOB_PATTERN_SYMBOLS.contains(&c))
}

static ALLOWED_SYMBOLS: Lazy<HashSet<char>> = Lazy::new(|| {
    let mut m = HashSet::new();
    m.insert('/');
    m.insert('_');
    m.insert('-');
    m.insert('.');
    m
});

pub fn is_valid_segment_name_char(char: char) -> bool {
    char.is_alphanumeric() || ALLOWED_SYMBOLS.contains(&char) || GLOB_PATTERN_SYMBOLS.contains(&char)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_find_new_line() {
        assert_eq!(find_new_line_index("\ncontent"), Some(0));
    }
}
