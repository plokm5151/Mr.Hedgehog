use dashmap::DashMap;

pub struct SourceManager {
    // path -> lines
    files: DashMap<String, Vec<String>>,
}

impl SourceManager {
    pub fn new(loaded_files: &[(String, String, String)]) -> Self {
        let sm = SourceManager {
            files: DashMap::new(),
        };
        for (_, file_path, content) in loaded_files {
            let lines: Vec<String> = content.lines().map(|s| s.to_string()).collect();
            sm.files.insert(file_path.clone(), lines);
        }
        sm
    }

    pub fn get_snippet(&self, file_path: &str, line_number: usize) -> Option<String> {
        if line_number == 0 { return None; }
        // Attempt to retrieve using the path as is
        if let Some(lines) = self.files.get(file_path) {
            if line_number <= lines.len() {
                return Some(lines[line_number - 1].trim().to_string());
            }
        }
        None
    }
}
