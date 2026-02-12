# LockSearch

A fast, sleek Windows program launcher built with Rust and [iced](https://iced.rs). LockSearch indexes your installed programs and provides instant fuzzy search — think Spotlight for Windows, but lightweight and customizable.

![Rust](https://img.shields.io/badge/Rust-000000?style=flat&logo=rust&logoColor=white)
![Windows](https://img.shields.io/badge/Windows-0078D6?style=flat&logo=windows&logoColor=white)

## Features

- **Instant fuzzy search** — find any installed program by name with smart matching
- **Modern dark UI** — refined dark theme with glowing accents, rounded panels, and smooth styling
- **Index caching** — programs appear instantly on subsequent launches
- **Custom frameless window** — draggable title bar with minimize/maximize/close, resizable from edges
- **Auto-generated icons** — letter placeholders for programs without icons
- **Configurable** — YAML config for window size, colors, sort order, caching, and more

## Screenshot

<!-- Add a screenshot here: ![LockSearch](screenshots/locksearch.png) -->

## Installation

### Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) (1.70+)
- Windows 10/11

### Build from source

```bash
git clone https://github.com/YOUR_USERNAME/locksearch.git
cd locksearch
cargo build --release
```

The binary will be at `target/release/locksearch.exe`.

### Run

```bash
cargo run --release
```

## Configuration

LockSearch uses a `config.yaml` file in the project directory. It is created with sensible defaults on first run.

```yaml
# Window settings
window_width: 500.0
window_height: 500.0

# Icon sizes (in pixels)
search_icon_size: 18
program_icon_size: 42

# Maximum search results to display
max_results: 10

# Theme colors (hex format)
theme:
  background: "#1B1F28"
  panel: "#222733"
  accent: "#7A5CCB"
  selected: "#2E3546"

# Additional directories to index (besides Start Menu and Program Files)
extra_index_paths: []

# Directories to exclude from indexing
exclude_paths: []

# Initial sort order for program list: "alphabetical" or "random"
initial_sort: "alphabetical"

# Cache the program index for instant startup (true/false)
enable_cache: true
```

## How It Works

1. **Indexing** — On startup, LockSearch scans the Start Menu and Program Files directories for `.lnk` shortcuts and `.exe` files. Results are cached to disk for instant loading on the next launch.
2. **Search** — As you type, fuzzy matching scores each program by name. Start Menu items and prefix matches get a boost.
3. **Launch** — Press `Enter` to open the selected program, or use `↑`/`↓` to navigate results.

## Keyboard Shortcuts

| Key | Action |
|---|---|
| `↑` / `↓` | Navigate results |
| `Enter` | Launch selected program |
| `Escape` | Clear search / show all programs |

## Architecture

```
src/
├── main.rs       # Entry point, window configuration
├── ui.rs         # UI layout, styling, message handling (iced)
├── indexer.rs    # Program discovery, icon extraction, caching
├── search.rs     # Fuzzy search engine
├── config.rs     # YAML configuration loading
└── platform.rs   # Windows API integration (frameless resize)
```

## Contributing

Contributions are welcome! Please open an issue or submit a pull request.

## License

See [LICENSE](LICENSE) for details.
