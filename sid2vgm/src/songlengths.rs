use std::collections::HashMap;
use std::path::Path;

pub struct SongLengths {
    by_hash: HashMap<String, Vec<u32>>,
    by_name: HashMap<String, Vec<u32>>,
}

impl SongLengths {
    pub fn load(path: &Path) -> Self {
        let mut by_hash = HashMap::new();
        let mut by_name: HashMap<String, Vec<u32>> = HashMap::new();

        let Ok(content) = std::fs::read_to_string(path) else {
            return Self { by_hash, by_name };
        };

        let mut current_name: Option<String> = None;

        for line in content.lines() {
            if line.starts_with(';') {
                let path_part = line.trim_start_matches(';').trim();
                current_name = path_part.split('/').next_back().map(str::to_string);
            } else if line.starts_with('[') || line.is_empty() {
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

    /// Duration in seconds for the given subtune (1-based; 0 means the SID's
    /// default start song from its header).
    pub fn duration_secs(&self, sid_path: &Path, subtune: u16) -> Option<u32> {
        let data = std::fs::read(sid_path).ok();
        let subtune = if subtune == 0 {
            data.as_deref()
                .map_or(1, crate::sid::loader::read_start_song)
        } else {
            subtune
        };
        let idx = (subtune - 1) as usize;

        if let Some(data) = &data {
            let hash = format!("{:x}", md5::compute(data));
            if let Some(v) = self.by_hash.get(&hash) {
                return v.get(idx).or_else(|| v.first()).copied();
            }
        }
        let name = sid_path.file_name()?.to_str()?;
        let durations = self.by_name.get(name)?;
        durations.get(idx).or_else(|| durations.first()).copied()
    }
}

/// Search for Songlengths.txt by walking up from `start_dir`.
/// Checks both `<dir>/Songlengths.txt` and `<dir>/DOCUMENTS/Songlengths.txt`.
pub fn find_songlengths(start_dir: &Path) -> Option<std::path::PathBuf> {
    let mut dir = start_dir;
    loop {
        let candidate = dir.join("Songlengths.txt");
        if candidate.exists() {
            return Some(candidate);
        }
        let candidate = dir.join("DOCUMENTS").join("Songlengths.txt");
        if candidate.exists() {
            return Some(candidate);
        }
        dir = dir.parent()?;
    }
}

fn parse_duration(s: &str) -> Option<u32> {
    let (m, s) = s.split_once(':')?;
    Some(m.parse::<u32>().ok()? * 60 + s.parse::<u32>().ok()?)
}
