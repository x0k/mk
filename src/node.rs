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