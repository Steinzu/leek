use anyhow::Result;
use directories::UserDirs;
use rodio::{Decoder, OutputStream, OutputStreamHandle, Sink, Source};
use std::env;
use std::fs;
use std::fs::File;
use std::io::BufReader;
use std::path::{Path, PathBuf};
use std::time::Duration;

#[derive(Clone, Debug, PartialEq)]
pub enum FileType {
    Directory,
    AudioFile,
    Other,
}

#[derive(Clone, Debug)]
pub struct BrowserItem {
    pub path: PathBuf,
    pub name: String,
    pub file_type: FileType,
}

pub struct App {
    // Browser State
    pub current_directory: PathBuf,
    pub browser_items: Vec<BrowserItem>,
    pub browser_index: usize,

    // Playback State
    pub queue: Vec<PathBuf>,
    pub queue_index: usize,
    pub volume: u8, // 0-100
    pub is_playing: bool,

    // Progress
    pub elapsed: Duration,
    pub duration: Option<Duration>,
    pub tick_counter: u64,

    // Audio backend
    _stream: OutputStream,
    stream_handle: OutputStreamHandle,
    sink: Sink,
}

impl App {
    pub fn new() -> Result<Self> {
        let (_stream, stream_handle) = OutputStream::try_default()?;
        let sink = Sink::try_new(&stream_handle)?;

        let args: Vec<String> = env::args().collect();

        let start_dir = if args.contains(&String::from("-steins")) {
            PathBuf::from(r"D:\Soulseek\share")
        } else if args.len() > 1 {
            PathBuf::from(&args[1])
        } else if let Some(user_dirs) = UserDirs::new() {
            user_dirs
                .audio_dir()
                .unwrap_or(user_dirs.home_dir())
                .to_path_buf()
        } else {
            PathBuf::from(".")
        };

        let mut app = Self {
            current_directory: start_dir.clone(),
            browser_items: Vec::new(),
            browser_index: 0,

            queue: Vec::new(),
            queue_index: 0,

            volume: 50,
            is_playing: false,
            elapsed: Duration::from_secs(0),
            duration: None,
            tick_counter: 0,

            _stream,
            stream_handle,
            sink,
        };

        app.load_directory(&start_dir);
        app.sink.set_volume(app.volume as f32 / 100.0);

        Ok(app)
    }

    pub fn load_directory(&mut self, path: &Path) {
        if !path.exists() || !path.is_dir() {
            return;
        }

        self.browser_items.clear();
        self.browser_index = 0;
        self.current_directory = path.to_path_buf();

        if let Ok(entries) = fs::read_dir(path) {
            let mut items: Vec<BrowserItem> = entries
                .flatten()
                .map(|entry| {
                    let path = entry.path();
                    let name = path
                        .file_name()
                        .unwrap_or_default()
                        .to_string_lossy()
                        .to_string();
                    let file_type = if path.is_dir() {
                        FileType::Directory
                    } else if let Some(ext) = path.extension() {
                        let ext_str = ext.to_string_lossy().to_lowercase();
                        if ["mp3", "wav", "flac", "ogg"].contains(&ext_str.as_str()) {
                            FileType::AudioFile
                        } else {
                            FileType::Other
                        }
                    } else {
                        FileType::Other
                    };
                    BrowserItem {
                        path,
                        name,
                        file_type,
                    }
                })
                .filter(|item| item.file_type != FileType::Other) // Show only Dirs and Audio
                .collect();

            // Sort: Directories first, then files
            items.sort_by(|a, b| {
                match (
                    a.file_type == FileType::Directory,
                    b.file_type == FileType::Directory,
                ) {
                    (true, false) => std::cmp::Ordering::Less,
                    (false, true) => std::cmp::Ordering::Greater,
                    _ => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
                }
            });

            self.browser_items = items;
        }
    }

    pub fn on_tick(&mut self) {
        // Increment progress (Tick is ~250ms)
        if self.is_playing {
            self.tick_counter += 1;
            // Every 4 ticks is roughly 1 second (250ms tick rate)
            // But better to just add tick_rate if known, or approximation.
            // Events.rs uses 250ms.
            self.elapsed += Duration::from_millis(250);

            // Auto skip if finished (basic check)
            // If sink is empty but we had a duration set, assume finished.
            // Or use the duration to guess. Sink::empty() is more reliable for "nothing playing".
            if self.sink.empty() && !self.queue.is_empty() {
                // Check if we actually just finished a song or if it's just startup
                // If duration is set, it means we were playing something.
                if self.duration.is_some() {
                    self.next_track();
                }
            }
        }
    }

    pub fn enter_selected(&mut self) {
        if self.browser_items.is_empty() {
            return;
        }

        let selected = &self.browser_items[self.browser_index].clone();

        match selected.file_type {
            FileType::Directory => {
                self.load_directory(&selected.path);
            }
            FileType::AudioFile => {
                // Play all files in current dir starting from selected
                self.queue = self
                    .browser_items
                    .iter()
                    .filter(|item| item.file_type == FileType::AudioFile)
                    .map(|item| item.path.clone())
                    .collect();

                // Find index of selected file in the new queue
                if let Some(idx) = self.queue.iter().position(|p| p == &selected.path) {
                    self.queue_index = idx;
                    self.play_queue_item();
                }
            }
            FileType::Other => {}
        }
    }

    pub fn play_folder(&mut self) {
        if self.browser_items.is_empty() {
            return;
        }

        let selected = &self.browser_items[self.browser_index].clone();

        if selected.file_type == FileType::Directory {
            // Play all files inside the selected directory
            if let Ok(entries) = fs::read_dir(&selected.path) {
                let mut folder_files: Vec<PathBuf> = entries
                    .flatten()
                    .map(|e| e.path())
                    .filter(|path| {
                        if let Some(ext) = path.extension() {
                            let ext_str = ext.to_string_lossy().to_lowercase();
                            ["mp3", "wav", "flac", "ogg"].contains(&ext_str.as_str())
                        } else {
                            false
                        }
                    })
                    .collect();

                folder_files.sort();

                if !folder_files.is_empty() {
                    self.queue = folder_files;
                    self.queue_index = 0;
                    self.play_queue_item();
                }
            }
        }
    }

    pub fn go_up(&mut self) {
        if let Some(parent) = self.current_directory.parent() {
            let parent_path = parent.to_path_buf(); // Clone to avoid borrow issues
            self.load_directory(&parent_path);
        }
    }

    fn play_queue_item(&mut self) {
        if let Some(path) = self.queue.get(self.queue_index) {
            self.sink.stop();
            // Recreate sink to clear queue
            if let Ok(new_sink) = Sink::try_new(&self.stream_handle) {
                self.sink = new_sink;
                self.sink.set_volume(self.volume as f32 / 100.0);
            }

            if let Ok(file) = File::open(path) {
                let reader = BufReader::new(file);
                if let Ok(source) = Decoder::new(reader) {
                    // Capture duration before appending
                    self.duration = source.total_duration();
                    self.elapsed = Duration::from_secs(0);

                    self.sink.append(source);
                    self.sink.play();
                    self.is_playing = true;
                }
            }
        }
    }

    pub fn toggle_play(&mut self) {
        if self.sink.empty() && !self.queue.is_empty() {
            self.play_queue_item();
        } else if self.sink.is_paused() {
            self.sink.play();
            self.is_playing = true;
        } else {
            self.sink.pause();
            self.is_playing = false;
        }
    }

    pub fn next_track(&mut self) {
        if self.queue.is_empty() {
            return;
        }
        if self.queue_index + 1 < self.queue.len() {
            self.queue_index += 1;
        } else {
            self.queue_index = 0; // Loop queue
        }
        self.play_queue_item();
    }

    pub fn prev_track(&mut self) {
        if self.queue.is_empty() {
            return;
        }
        if self.queue_index > 0 {
            self.queue_index -= 1;
        } else {
            self.queue_index = self.queue.len() - 1;
        }
        self.play_queue_item();
    }

    // Browser Navigation
    pub fn next_item(&mut self) {
        if !self.browser_items.is_empty() {
            if self.browser_index + 1 < self.browser_items.len() {
                self.browser_index += 1;
            } else {
                self.browser_index = 0;
            }
        }
    }

    pub fn prev_item(&mut self) {
        if !self.browser_items.is_empty() {
            if self.browser_index > 0 {
                self.browser_index -= 1;
            } else {
                self.browser_index = self.browser_items.len() - 1;
            }
        }
    }

    pub fn volume_up(&mut self) {
        self.volume = (self.volume + 5).min(100);
        self.sink.set_volume(self.volume as f32 / 100.0);
    }

    pub fn volume_down(&mut self) {
        self.volume = self.volume.saturating_sub(5);
        self.sink.set_volume(self.volume as f32 / 100.0);
    }
}
