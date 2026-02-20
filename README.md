# Leek 🎵

A modern, fancy terminal-based music player written in Rust, featuring a Hatsune Miku inspired theme.

## Features
- **File Browser**: Navigate your file system to find music (supports `mp3`, `wav`, `flac`, `ogg`).
- **One-Key Play**: Play an entire folder instantly with `Tab`.
- **Music Progression**: See the current song's elapsed time and total duration.
- **Controls**: Keyboard driven interface.
- **Theme**: Polished Cyan/Blue aesthetic.

## Controls
| Key | Action |
| --- | --- |
| `Space` | Play / Pause |
| `Enter` | Enter Directory / Play File |
| `Tab` | **Play Whole Folder** (Starts playing all songs in selected dir) |
| `Backspace` | Go Up Directory |
| `Left` / `Right` | Previous / Next Track |
| `↑` / `k` | Previous Item |
| `↓` / `j` | Next Item |
| `PageUp` | Volume Up (+5%) |
| `PageDown` | Volume Down (-5%) |
| `q` / `Esc` | Quit |

## How to Run

### 1. Run directly
By default, it starts in your standard Music directory (or Home if not found).
```bash
cargo run --release
```

You can also specify a starting directory:
```bash
cargo run --release -- "C:\Path\To\Music"
```

### 2. Install Globally (Recommended)
To run `leek` from anywhere in your terminal:

```bash
cargo install --path .
```

Then you can simply run:
```bash
leek
# OR
leek "C:\My\Music\Folder"
```
