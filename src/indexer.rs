use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
use walkdir::WalkDir;

/// Represents a program/executable entry
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProgramEntry {
    pub path: PathBuf,
    pub name: String,
    pub display_name: String,
    pub source: ProgramSource,
    pub icon_path: Option<PathBuf>,
}

/// Where the program was found
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum ProgramSource {
    StartMenu,
    ProgramFiles,
}

/// The program index
pub struct ProgramIndex {
    entries: Arc<RwLock<Vec<ProgramEntry>>>,
    is_indexing: Arc<RwLock<bool>>,
    indexed_count: Arc<RwLock<usize>>,
    icon_cache_dir: PathBuf,
    cache_path: PathBuf,
}

impl Default for ProgramIndex {
    fn default() -> Self {
        Self::new()
    }
}

impl ProgramIndex {
    pub fn new() -> Self {
        // Create icon cache directory
        let icon_cache_dir = dirs::cache_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("locksearch")
            .join("icons");
        let _ = fs::create_dir_all(&icon_cache_dir);

        let cache_path = dirs::cache_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("locksearch")
            .join("index_cache.json");

        Self {
            entries: Arc::new(RwLock::new(Vec::new())),
            is_indexing: Arc::new(RwLock::new(false)),
            indexed_count: Arc::new(RwLock::new(0)),
            icon_cache_dir,
            cache_path,
        }
    }

    pub async fn is_indexing(&self) -> bool {
        *self.is_indexing.read().await
    }

    pub async fn indexed_count(&self) -> usize {
        *self.indexed_count.read().await
    }

    pub async fn get_entries(&self) -> Vec<ProgramEntry> {
        self.entries.read().await.clone()
    }

    /// Load cached index from disk. Returns true if cache was loaded.
    pub async fn load_cache(&self) -> bool {
        if !self.cache_path.exists() {
            return false;
        }
        match fs::read_to_string(&self.cache_path) {
            Ok(data) => match serde_json::from_str::<Vec<ProgramEntry>>(&data) {
                Ok(cached) => {
                    let count = cached.len();
                    {
                        let mut e = self.entries.write().await;
                        *e = cached;
                    }
                    {
                        let mut c = self.indexed_count.write().await;
                        *c = count;
                    }
                    true
                }
                Err(_) => false,
            },
            Err(_) => false,
        }
    }

    /// Save current index to disk cache.
    fn save_cache_sync(cache_path: &PathBuf, entries: &[ProgramEntry]) {
        if let Ok(json) = serde_json::to_string(entries) {
            let _ = fs::write(cache_path, json);
        }
    }

    pub async fn start_indexing(&self) {
        {
            let mut indexing = self.is_indexing.write().await;
            if *indexing {
                return;
            }
            *indexing = true;
        }

        let entries = Arc::clone(&self.entries);
        let is_indexing = Arc::clone(&self.is_indexing);
        let indexed_count = Arc::clone(&self.indexed_count);
        let icon_cache_dir = self.icon_cache_dir.clone();
        let cache_path = self.cache_path.clone();

        tokio::task::spawn_blocking(move || {
            let mut programs: Vec<ProgramEntry> = Vec::new();
            let mut seen: HashMap<String, bool> = HashMap::new();

            // Index Start Menu (highest priority)
            let start_menu_paths = get_start_menu_paths();
            for start_path in start_menu_paths {
                if start_path.exists() {
                    index_directory(&start_path, ProgramSource::StartMenu, &mut programs, &mut seen, &icon_cache_dir);
                }
            }

            // Index Program Files
            let program_dirs = [
                PathBuf::from("C:\\Program Files"),
                PathBuf::from("C:\\Program Files (x86)"),
            ];
            for dir in &program_dirs {
                if dir.exists() {
                    index_directory(dir, ProgramSource::ProgramFiles, &mut programs, &mut seen, &icon_cache_dir);
                }
            }

            // Sort by source priority and name
            programs.sort_by(|a, b| {
                let priority_a = match a.source {
                    ProgramSource::StartMenu => 0,
                    ProgramSource::ProgramFiles => 1,
                };
                let priority_b = match b.source {
                    ProgramSource::StartMenu => 0,
                    ProgramSource::ProgramFiles => 1,
                };
                priority_a.cmp(&priority_b).then_with(|| a.display_name.cmp(&b.display_name))
            });

            let count = programs.len();

            // Update shared state in blocking context
            let rt = tokio::runtime::Handle::current();
            rt.block_on(async {
                {
                    let mut e = entries.write().await;
                    *e = programs;
                }
                {
                    let mut cnt = indexed_count.write().await;
                    *cnt = count;
                }
                {
                    let mut idx = is_indexing.write().await;
                    *idx = false;
                }
                // Save cache to disk
                let entries_snapshot = entries.read().await.clone();
                ProgramIndex::save_cache_sync(&cache_path, &entries_snapshot);
            });
        });
    }
}

fn get_start_menu_paths() -> Vec<PathBuf> {
    let mut paths = Vec::new();
    paths.push(PathBuf::from("C:\\ProgramData\\Microsoft\\Windows\\Start Menu\\Programs"));
    if let Some(appdata) = dirs::data_dir() {
        let user_start = appdata
            .parent()
            .map(|p| p.join("Roaming\\Microsoft\\Windows\\Start Menu\\Programs"));
        if let Some(p) = user_start {
            paths.push(p);
        }
    }
    paths
}

fn index_directory(
    dir: &PathBuf,
    source: ProgramSource,
    programs: &mut Vec<ProgramEntry>,
    seen: &mut HashMap<String, bool>,
    icon_cache_dir: &PathBuf,
) {
    let max_depth = match source {
        ProgramSource::StartMenu => 5,
        ProgramSource::ProgramFiles => 2,
    };

    let extensions: &[&str] = match source {
        ProgramSource::StartMenu => &["lnk"],
        ProgramSource::ProgramFiles => &["exe"],
    };

    for entry in WalkDir::new(dir)
        .max_depth(max_depth)
        .follow_links(false)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();

        if !path.is_file() {
            continue;
        }

        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .map(|e| e.to_lowercase());

        let is_valid_ext = ext.as_ref().map_or(false, |e| extensions.contains(&e.as_str()));

        if !is_valid_ext {
            continue;
        }

        // Skip uninstallers and updaters
        let name_lower = path
            .file_stem()
            .and_then(|n| n.to_str())
            .map(|n| n.to_lowercase())
            .unwrap_or_default();

        if name_lower.contains("uninstall")
            || name_lower.contains("uninst")
            || name_lower.contains("update")
            || name_lower.contains("updater")
            || name_lower.contains("setup")
        {
            continue;
        }

        let (display_name, target_path) = get_display_name_and_target(path, &ext);
        let key = display_name.to_lowercase();

        // Avoid duplicates
        if seen.contains_key(&key) {
            continue;
        }
        seen.insert(key, true);

        // Extract icon
        let icon_path = extract_icon(&target_path, &display_name, icon_cache_dir);

        programs.push(ProgramEntry {
            path: path.to_path_buf(),
            name: name_lower,
            display_name,
            source: source.clone(),
            icon_path,
        });
    }
}

fn get_display_name_and_target(path: &std::path::Path, ext: &Option<String>) -> (String, PathBuf) {
    if ext.as_ref().map_or(false, |e| e == "lnk") {
        // Wrap in catch_unwind because the lnk crate can panic on malformed .lnk files
        // (e.g. unwrap() on None in header.rs for missing fields)
        let path_buf = path.to_path_buf();
        let lnk_result = std::panic::catch_unwind(move || {
            lnk::ShellLink::open(&path_buf)
        });

        if let Ok(Ok(lnk)) = lnk_result {
            // Clone values to avoid ownership issues
            let name_opt = lnk.name().clone();
            let name = name_opt.map(|s| s.to_string()).filter(|s| !s.is_empty());
            
            // Get target path from link info
            let mut target = path.to_path_buf();
            let link_info_opt = lnk.link_info().clone();
            if let Some(li) = link_info_opt {
                if let Some(bp) = li.local_base_path() {
                    target = PathBuf::from(bp);
                }
            }
            
            let display = name.unwrap_or_else(|| {
                path.file_stem()
                    .and_then(|n| n.to_str())
                    .map(|n| n.to_string())
                    .unwrap_or_else(|| "Unknown".to_string())
            });
            return (display, target);
        }
    }
    
    let name = path.file_stem()
        .and_then(|n| n.to_str())
        .map(|n| n.to_string())
        .unwrap_or_else(|| "Unknown".to_string());
    (name, path.to_path_buf())
}

fn extract_icon(exe_path: &PathBuf, display_name: &str, cache_dir: &PathBuf) -> Option<PathBuf> {
    // Create a safe filename from display name
    let safe_name: String = display_name
        .chars()
        .filter(|c| c.is_alphanumeric() || *c == ' ' || *c == '-' || *c == '_')
        .take(50)
        .collect();
    let icon_filename = format!("{}.png", safe_name.replace(' ', "_"));
    let icon_path = cache_dir.join(&icon_filename);

    // Check if already cached
    if icon_path.exists() {
        return Some(icon_path);
    }

    // Try to extract icon
    let path_str = exe_path.to_string_lossy();
    if let Ok(icon_data) = systemicons::get_icon(&path_str, 48) {
        if fs::write(&icon_path, &icon_data).is_ok() {
            return Some(icon_path);
        }
    }

    None
}

impl Clone for ProgramIndex {
    fn clone(&self) -> Self {
        Self {
            entries: Arc::clone(&self.entries),
            is_indexing: Arc::clone(&self.is_indexing),
            indexed_count: Arc::clone(&self.indexed_count),
            icon_cache_dir: self.icon_cache_dir.clone(),
            cache_path: self.cache_path.clone(),
        }
    }
}
