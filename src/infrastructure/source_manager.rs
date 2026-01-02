use std::collections::HashMap;

#[derive(Default)]
pub struct SourceManager {
    // Map absolute file path -> Lines
    files: HashMap<String, Vec<String>>,
}

impl SourceManager {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn load_file(&mut self, file_path: String, content: String) {
        let lines = content.lines().map(|s| s.to_string()).collect();
        self.files.insert(file_path, lines);
    }

    pub fn get_snippet(&self, file_path: &str, line_number: usize) -> Option<String> {
        let lines = self.files.get(file_path)?;
        // line_number is 1-indexed
        if line_number == 0 || line_number > lines.len() {
            return None;
        }
        Some(lines[line_number - 1].trim().to_string())
    }
}
