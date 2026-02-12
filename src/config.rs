use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/// Application configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Window width
    #[serde(default = "default_window_width")]
    pub window_width: f32,
    
    /// Window height
    #[serde(default = "default_window_height")]
    pub window_height: f32,
    
    /// Search icon size
    #[serde(default = "default_search_icon_size")]
    pub search_icon_size: u16,
    
    /// Program icon size
    #[serde(default = "default_program_icon_size")]
    pub program_icon_size: u16,
    
    /// Maximum results to show
    #[serde(default = "default_max_results")]
    pub max_results: usize,
    
    /// Theme colors
    #[serde(default)]
    pub theme: ThemeConfig,
    
    /// Directories to index (in addition to defaults)
    #[serde(default)]
    pub extra_index_paths: Vec<String>,
    
    /// Directories to exclude from indexing
    #[serde(default)]
    pub exclude_paths: Vec<String>,
    
    /// Initial sort order: "alphabetical" or "random"
    #[serde(default = "default_initial_sort")]
    pub initial_sort: String,

    /// Enable index caching for instant startup
    #[serde(default = "default_enable_cache")]
    pub enable_cache: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeConfig {
    /// Background color (hex)
    #[serde(default = "default_bg_color")]
    pub background: String,
    
    /// Panel background color (hex)
    #[serde(default = "default_panel_color")]
    pub panel: String,
    
    /// Accent/glow color (hex)
    #[serde(default = "default_accent_color")]
    pub accent: String,
    
    /// Selected item color (hex)
    #[serde(default = "default_selected_color")]
    pub selected: String,
}

// Default value functions
fn default_window_width() -> f32 { 500.0 }
fn default_window_height() -> f32 { 500.0 }
fn default_search_icon_size() -> u16 { 18 }
fn default_program_icon_size() -> u16 { 42 }
fn default_max_results() -> usize { 10 }
fn default_bg_color() -> String { "#1B1F28".to_string() }
fn default_panel_color() -> String { "#222733".to_string() }
fn default_accent_color() -> String { "#7A5CCB".to_string() }
fn default_selected_color() -> String { "#2E3546".to_string() }
fn default_initial_sort() -> String { "alphabetical".to_string() }
fn default_enable_cache() -> bool { true }

impl Default for Config {
    fn default() -> Self {
        Self {
            window_width: default_window_width(),
            window_height: default_window_height(),
            search_icon_size: default_search_icon_size(),
            program_icon_size: default_program_icon_size(),
            max_results: default_max_results(),
            theme: ThemeConfig::default(),
            extra_index_paths: Vec::new(),
            exclude_paths: Vec::new(),
            initial_sort: default_initial_sort(),
            enable_cache: default_enable_cache(),
        }
    }
}

impl Default for ThemeConfig {
    fn default() -> Self {
        Self {
            background: default_bg_color(),
            panel: default_panel_color(),
            accent: default_accent_color(),
            selected: default_selected_color(),
        }
    }
}

impl Config {
    /// Get the config file path (in project folder or next to executable)
    pub fn config_path() -> PathBuf {
        // Try current directory first
        let local_path = PathBuf::from("config.yaml");
        if local_path.exists() {
            return local_path;
        }
        
        // Try next to executable
        if let Ok(exe_path) = std::env::current_exe() {
            if let Some(exe_dir) = exe_path.parent() {
                let exe_config = exe_dir.join("config.yaml");
                if exe_config.exists() {
                    return exe_config;
                }
            }
        }
        
        // Default to local path
        local_path
    }

    /// Load config from file, or use defaults if not exists
    pub fn load() -> Self {
        let path = Self::config_path();
        
        if path.exists() {
            match fs::read_to_string(&path) {
                Ok(content) => {
                    match serde_yaml::from_str(&content) {
                        Ok(config) => return config,
                        Err(e) => {
                            eprintln!("Failed to parse config: {}", e);
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Failed to read config: {}", e);
                }
            }
        }
        
        Config::default()
    }
}
