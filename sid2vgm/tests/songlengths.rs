use std::collections::HashMap;
use std::path::Path;

/// Maximum duration used in fidelity tests regardless of actual song length,
/// to keep CI times reasonable.
pub const MAX_TEST_SECS: u32 = 60;

pub struct SongLengths {
    by_hash: HashMap<String, Vec<u32>>,
    by_name: HashMap<String, Vec<u32>>,
}

impl SongLengths {
    pub fn load(path: &str) -> Self {
        let mut by_hash = HashMap::new();
        let mut by_name: HashMap<String, Vec<u32>> = HashMap::new();

        let Ok(content) = std::fs::read_to_string(path) else {
            return Self { by_hash, by_name };
        };

        let mut current_name: Option<String> = None;

        for line in content.lines() {
            if line.starts_with(';') {
                // "; /path/to/File.sid" — capture the basename
                let path_part = line.trim_start_matches(';').trim();
                current_name = path_part.split('/').last().map(str::to_string);
            } else if line.starts_with('[') || line.is_empty() {
                // section header or blank — reset context
                current_name = None;
            } else if let Some((hash, durations_str)) = line.split_once('=') {
                let durations: Vec<u32> = durations_str
                    .split_whitespace()
                    .filter_map(parse_duration)
                    .collect();
                if !durations.is_empty() {
                    by_hash.insert(hash.to_lowercase(), durations.clone());
                    if let Some(name) = current_name.take() {
                        by_name.insert(name, durations);
                    }
                }
            }
        }

        Self { by_hash, by_name }
    }

    /// Return the duration (seconds) for the default subtune of the given SID file.
    /// Looks up by MD5 hash first, then by filename.
    /// Returns `None` if not found in the database.
    pub fn duration_secs(&self, sid_path: &Path) -> Option<u32> {
        // Hash lookup
        if let Ok(data) = std::fs::read(sid_path) {
            let hash = format!("{:x}", md5::compute(&data));
            if let Some(d) = self.by_hash.get(&hash).and_then(|v| v.first()) {
                return Some(*d);
            }
        }
        // Filename fallback
        let name = sid_path.file_name()?.to_str()?;
        self.by_name.get(name)?.first().copied()
    }
}

fn parse_duration(s: &str) -> Option<u32> {
    let (m, s) = s.split_once(':')?;
    Some(m.parse::<u32>().ok()? * 60 + s.parse::<u32>().ok()?)
}
