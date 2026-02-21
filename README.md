# Leek

**Leek** is a lightweight, terminal-based music player (TUI) written in Rust. It allows you to browse your file system, play individual audio files, or queue up entire folders of music, all from the comfort of your command line.

## Features

*   **File Browser**: Navigate your file system to find your music library.
*   **Format Support**: Plays MP3, FLAC, WAV, and OGG Vorbis files.
*   **Queue Management**: Play single files or enqueue entire directories.
*   **Playback Controls**: Play/Pause, Next/Previous Track, and seek (automatic).
*   **Volume Control**: Adjust volume directly from the TUI.
*   **Visual Feedback**:
    *   Now Playing information.
    *   Playback progress bar.
    *   Volume gauge.
    *   Highlighted file selection.

## Installation

### Prerequisites

You need to have **Rust** and **Cargo** installed. If you don't have them, install them from [rustup.rs](https://rustup.rs/).

### Building & Installing

To build the project and install the binary to your Cargo binary path (making it accessible from anywhere in your terminal):

```bash
cargo install --path .
```

Alternatively, if you just want to build the binary without installing it globally:

```bash
cargo build --release
```

The executable will be located at `target/release/leek` (or `leek.exe` on Windows).

## Usage

If you installed it via `cargo install`, simply run:

```bash
leek
```

Or specify a starting directory:

```bash
leek "C:\Users\YourName\Music"
```

If you built it locally without installing:

```bash
./target/release/leek
```