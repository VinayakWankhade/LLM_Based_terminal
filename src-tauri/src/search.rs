use regex::Regex;
use serde::Serialize;
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize)]
pub struct ScrollMatch {
    pub line_index: usize,
    pub start: usize,
    pub end: usize,
    pub line: String,
    pub line_content: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ContextLine {
    pub line_index: usize,
    pub line: String,
}

pub struct ScrollbackIndex {
    lines: Vec<String>,
    buf: String,
    max_lines: usize,
    ansi_re: Regex,
}

impl ScrollbackIndex {
    pub fn new(max_lines: usize) -> Self {
        // Basic ANSI escape matcher to strip sequences
        let ansi_re = Regex::new(r"\x1B\[[0-9;?]*[ -/]*[@-~]").unwrap();
        Self {
            lines: Vec::with_capacity(max_lines.min(1024)),
            buf: String::new(),
            max_lines,
            ansi_re,
        }
    }

    pub fn append(&mut self, data: &str) {
        // Strip ANSI and normalize line endings to \n
        let mut text = self.ansi_re.replace_all(data, "").to_string();
        // Convert CRLF and CR to LF
        text = text.replace("\r\n", "\n").replace('\r', "\n");

        for ch in text.chars() {
            if ch == '\n' {
                self.push_line();
            } else {
                self.buf.push(ch);
            }
        }
    }

    fn push_line(&mut self) {
        let line = std::mem::take(&mut self.buf);
        self.lines.push(line);
        if self.lines.len() > self.max_lines {
            let overflow = self.lines.len() - self.max_lines;
            self.lines.drain(0..overflow);
        }
    }

    pub fn finalize_line_if_any(&mut self) {
        if !self.buf.is_empty() {
            self.push_line();
        }
    }

    pub fn search(&self, query: &str, case_sensitive: bool, use_regex: bool, limit: usize) -> Vec<ScrollMatch> {
        if query.is_empty() { return Vec::new(); }
        let mut results = Vec::new();

        if use_regex {
            if let Ok(mut re) = Regex::new(&format!("{}", query)) {
                for (i, line) in self.lines.iter().enumerate() {
                    let hay = if case_sensitive { line.as_str().to_string() } else { line.to_lowercase() };
                    let mut last_index = 0usize;
                    // To make case-insensitive regex, rebuild with (?i)
                    if !case_sensitive {
                        if let Ok(rr) = Regex::new(&format!("(?i){}", query)) { re = rr; }
                    }
                    for m in re.find_iter(&hay) {
                        let start = m.start();
                        let end = m.end();
                        last_index = end.max(last_index);
                        results.push(ScrollMatch { line_index: i, start, end, line: line.clone(), line_content: line.clone() });
                        if results.len() >= limit { return results; }
                        if start == end { // avoid zero-length loops
                            last_index += 1;
                        }
                    }
                }
            }
        } else {
            let needle = if case_sensitive { query.to_string() } else { query.to_lowercase() };
            for (i, line) in self.lines.iter().enumerate() {
                let hay = if case_sensitive { line.as_str().to_string() } else { line.to_lowercase() };
                let mut idx = 0usize;
                while !needle.is_empty() {
                    if let Some(pos) = hay[idx..].find(&needle) {
                        let start = idx + pos;
                        let end = start + needle.len();
                        results.push(ScrollMatch { line_index: i, start, end, line: line.clone(), line_content: line.clone() });
                        if results.len() >= limit { return results; }
                        idx = end.max(idx + 1);
                    } else {
                        break;
                    }
                }
            }
        }

        results
    }

    #[allow(dead_code)]
    pub fn window(&self, start: usize, count: usize) -> Vec<String> {
        let mut out = Vec::new();
        let end = (start + count).min(self.lines.len());
        for i in start..end {
            out.push(self.lines[i].clone());
        }
        out
    }

    pub fn context(&self, line_index: usize, before: usize, after: usize) -> Vec<ContextLine> {
        let start = line_index.saturating_sub(before);
        let end = (line_index + after + 1).min(self.lines.len());
        let mut out = Vec::new();
        for i in start..end {
            out.push(ContextLine { line_index: i, line: self.lines[i].clone() });
        }
        out
    }

    pub fn tail(&self, count: usize) -> Vec<String> {
        let len = self.lines.len();
        let start = len.saturating_sub(count);
        self.lines[start..len].to_vec()
    }
}

pub struct SearchIndexManager {
    sessions: HashMap<String, ScrollbackIndex>,
    max_lines: usize,
}

impl SearchIndexManager {
    pub fn new() -> Self {
        Self { sessions: HashMap::new(), max_lines: 5000 }
    }

    pub fn create_session(&mut self, session_id: String) {
        self.sessions.insert(session_id, ScrollbackIndex::new(self.max_lines));
    }

    pub fn remove_session(&mut self, session_id: &str) {
        self.sessions.remove(session_id);
    }

    pub fn append_output(&mut self, session_id: &str, data: &str) {
        if let Some(idx) = self.sessions.get_mut(session_id) {
            idx.append(data);
        }
    }

    pub fn search(&self, session_id: &str, query: &str, case_sensitive: bool, use_regex: bool, limit: usize) -> Option<Vec<ScrollMatch>> {
        self.sessions.get(session_id).map(|i| i.search(query, case_sensitive, use_regex, limit))
    }

    pub fn context(&self, session_id: &str, line_index: usize, before: usize, after: usize) -> Option<Vec<ContextLine>> {
        self.sessions.get(session_id).map(|i| i.context(line_index, before, after))
    }

    pub fn tail(&self, session_id: &str, count: usize) -> Option<Vec<String>> {
        self.sessions.get(session_id).map(|i| i.tail(count))
    }
}
