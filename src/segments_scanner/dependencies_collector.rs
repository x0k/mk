pub struct DependenciesCollector<'a> {
    content: &'a str,
    word_begin: isize,
}

impl<'a> DependenciesCollector<'a> {
    pub fn new(content: &'a str) -> Self {
        Self {
            content,
            word_begin: -1,
        }
    }

    fn start_if_not_started(&mut self, start: usize) {
        if self.word_begin == -1 {
            self.word_begin = start as isize;
        }
    }

    fn collect_if_started(&mut self, end: usize, deps: &mut Vec<&'a str>) {
        if self.word_begin != -1 {
            deps.push(&self.content[self.word_begin as usize..end]);
            self.word_begin = -1;
        }
    }

    pub fn collect(&mut self) -> (usize, Vec<&'a str>) {
        let mut deps = Vec::new();
        for (i, c) in self.content.char_indices() {
            if c == '\n' {
                self.collect_if_started(i, &mut deps);
                return (i + 1, deps);
            }
            if c.is_whitespace() {
                self.collect_if_started(i, &mut deps);
                continue;
            }
            self.start_if_not_started(i);
        }
        let l = self.content.len();
        self.collect_if_started(l, &mut deps);
        (l + 1, deps)
    }
}
