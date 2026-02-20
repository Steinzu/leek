use anyhow::Result;
use directories::UserDirs;
use rodio::{Decoder, OutputStream, OutputStreamHandle, Sink, Source};
use std::env;
use std::fs::{self, File};
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
    pub current_directory: PathBuf,
    pub browser_items: Vec<BrowserItem>,
    pub browser_index: usize,

    pub queue: Vec<PathBuf>,
    pub queue_index: usize,
    pub volume: u8,
    pub is_playing: bool,

    pub elapsed: Duration,
    pub duration: Option<Duration>,
    pub tick_counter: u64,

    _stream: OutputStream,
    stream_handle: OutputStreamHandle,
    sink: Sink,
}

impl App {
    pub fn new() -> Result<Self> {
        let (_stream, stream_handle) = OutputStream::try_default()?;
        let sink = Sink::try_new(&stream_handle)?;

        let args: Vec<String> = env::args().collect();
        let start_dir = Self::determine_start_dir(&args);

        let mut app = Self {
            current_directory: start_dir.clone(),
            browser_items: Vec::new(),
            browser_index: 0,
            queue: Vec::new(),
            queue_index: 0,
            volume: 50,
            is_playing: false,
            elapsed: Duration::ZERO,
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

    fn determine_start_dir(args: &[String]) -> PathBuf {
        if args.contains(&String::from("-steins")) {
            return PathBuf::from(r"D:\Soulseek\share");
        }
        if args.len() > 1 {
            return PathBuf::from(&args[1]);
        }
        UserDirs::new()
            .and_then(|ud| ud.audio_dir().map(|p| p.to_path_buf()))
            .or_else(|| UserDirs::new().map(|ud| ud.home_dir().to_path_buf()))
            .unwrap_or_else(|| PathBuf::from("."))
    }

    fn is_audio_file(path: &Path) -> bool {
        path.extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| matches!(ext.to_lowercase().as_str(), "mp3" | "wav" | "flac" | "ogg"))
            .unwrap_or(false)
    }

    pub fn load_directory(&mut self, path: &Path) {
        if !path.is_dir() {
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
                        .into_owned();

                    let file_type = if path.is_dir() {
                        FileType::Directory
                    } else if Self::is_audio_file(&path) {
                        FileType::AudioFile
                    } else {
                        FileType::Other
                    };

                    BrowserItem {
                        path,
                        name,
                        file_type,
                    }
                })
                .filter(|item| item.file_type != FileType::Other)
                .collect();

            items.sort_by(|a, b| {
                let a_is_dir = a.file_type == FileType::Directory;
                let b_is_dir = b.file_type == FileType::Directory;

                b_is_dir
                    .cmp(&a_is_dir)
                    .then_with(|| a.name.to_lowercase().cmp(&b.name.to_lowercase()))
            });

            self.browser_items = items;
        }
    }

    pub fn on_tick(&mut self) {
        if self.is_playing {
            self.tick_counter += 1;
            self.elapsed += Duration::from_millis(250);

            if self.sink.empty() && !self.queue.is_empty() && self.duration.is_some() {
                self.next_track();
            }
        }
    }

    pub fn enter_selected(&mut self) {
        if self.browser_items.is_empty() {
            return;
        }

        let selected = &self.browser_items[self.browser_index].clone();

        match selected.file_type {
            FileType::Directory => self.load_directory(&selected.path),
            FileType::AudioFile => {
                self.queue = self
                    .browser_items
                    .iter()
                    .filter(|item| item.file_type == FileType::AudioFile)
                    .map(|item| item.path.clone())
                    .collect();

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

        let selected = &self.browser_items[self.browser_index];
        if selected.file_type != FileType::Directory {
            return;
        }

        if let Ok(entries) = fs::read_dir(&selected.path) {
            let mut folder_files: Vec<PathBuf> = entries
                .flatten()
                .map(|e| e.path())
                .filter(|path| Self::is_audio_file(path))
                .collect();

            folder_files.sort();

            if !folder_files.is_empty() {
                self.queue = folder_files;
                self.queue_index = 0;
                self.play_queue_item();
            }
        }
    }

    pub fn go_up(&mut self) {
        if let Some(parent) = self.current_directory.parent() {
            self.load_directory(&parent.to_path_buf());
        }
    }

    fn play_queue_item(&mut self) {
        let Some(path) = self.queue.get(self.queue_index) else {
            return;
        };

        self.sink.stop();
        if let Ok(new_sink) = Sink::try_new(&self.stream_handle) {
            self.sink = new_sink;
            self.sink.set_volume(self.volume as f32 / 100.0);
        }

        if let Ok(file) = File::open(path) {
            let reader = BufReader::new(file);
            if let Ok(source) = Decoder::new(reader) {
                self.duration = source.total_duration();
                self.elapsed = Duration::ZERO;
                self.sink.append(source);
                self.sink.play();
                self.is_playing = true;
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
        self.queue_index = (self.queue_index + 1) % self.queue.len();
        self.play_queue_item();
    }

    pub fn prev_track(&mut self) {
        if self.queue.is_empty() {
            return;
        }
        self.queue_index = if self.queue_index > 0 {
            self.queue_index - 1
        } else {
            self.queue.len() - 1
        };
        self.play_queue_item();
    }

    pub fn next_item(&mut self) {
        if !self.browser_items.is_empty() {
            self.browser_index = (self.browser_index + 1) % self.browser_items.len();
        }
    }

    pub fn prev_item(&mut self) {
        if !self.browser_items.is_empty() {
            self.browser_index = if self.browser_index > 0 {
                self.browser_index - 1
            } else {
                self.browser_items.len() - 1
            };
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
