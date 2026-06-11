use std::collections::HashMap;
use std::path::Path;

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

    /// Return the duration (seconds) for the default subtune of the given SID file
    /// (the `startSong` from its header). Looks up by MD5 hash first, then by filename.
    /// Returns `None` if not found in the database.
    pub fn duration_secs(&self, sid_path: &Path) -> Option<u32> {
        let data = std::fs::read(sid_path).ok();
        let idx = data.as_deref().map_or(0, start_song_index);

        // Hash lookup
        if let Some(data) = &data {
            let hash = format!("{:x}", md5::compute(data));
            if let Some(v) = self.by_hash.get(&hash) {
                return v.get(idx).or_else(|| v.first()).copied();
            }
        }
        // Filename fallback
        let name = sid_path.file_name()?.to_str()?;
        let durations = self.by_name.get(name)?;
        durations.get(idx).or_else(|| durations.first()).copied()
    }
}

/// 0-based index of the default start song (`startSong` field at 0x10).
fn start_song_index(data: &[u8]) -> usize {
    if data.len() < 0x12 || (&data[0..4] != b"PSID" && &data[0..4] != b"RSID") {
        return 0;
    }
    (u16::from_be_bytes([data[0x10], data[0x11]]).max(1) - 1) as usize
}

fn parse_duration(s: &str) -> Option<u32> {
    let (m, s) = s.split_once(':')?;
    Some(m.parse::<u32>().ok()? * 60 + s.parse::<u32>().ok()?)
}
