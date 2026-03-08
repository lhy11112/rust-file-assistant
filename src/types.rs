use std::path::PathBuf;
use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq)]
pub enum FileItemType {
    File,
    Directory,
    Symlink,
}

#[derive(Debug, Clone)]
pub struct FileItem {
    pub path: PathBuf,
    pub name: String,
    pub item_type: FileItemType,
    pub size: u64,
    pub modified: Option<DateTime<Local>>,
    pub is_selected: bool,
    pub depth: usize,
    pub is_expanded: bool,
    pub children_loaded: bool,
}

impl FileItem {
    pub fn new(path: PathBuf, depth: usize) -> Self {
        let name = path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| path.to_string_lossy().to_string());

        let item_type = if path.is_dir() {
            FileItemType::Directory
        } else if path.is_symlink() {
            FileItemType::Symlink
        } else {
            FileItemType::File
        };

        let size = if item_type == FileItemType::File {
            std::fs::metadata(&path).map(|m| m.len()).unwrap_or(0)
        } else {
            0
        };

        let modified = std::fs::metadata(&path)
            .and_then(|m| m.modified())
            .ok()
            .map(|t| DateTime::<Local>::from(t));

        FileItem {
            path,
            name,
            item_type,
            size,
            modified,
            is_selected: false,
            depth,
            is_expanded: false,
            children_loaded: false,
        }
    }

    pub fn icon(&self) -> &'static str {
        match self.item_type {
            FileItemType::Directory => "📁",
            FileItemType::Symlink => "🔗",
            FileItemType::File => self.file_icon(),
        }
    }

    fn file_icon(&self) -> &'static str {
        let ext = self
            .path
            .extension()
            .map(|e| e.to_string_lossy().to_lowercase());
        match ext.as_deref() {
            Some("rs") => "🦀",
            Some("py") => "🐍",
            Some("js" | "ts") => "📜",
            Some("html" | "htm") => "🌐",
            Some("css") => "🎨",
            Some("json" | "toml" | "yaml" | "yml") => "⚙",
            Some("md" | "txt" | "log") => "📝",
            Some("png" | "jpg" | "jpeg" | "gif" | "bmp" | "svg" | "webp") => "🖼",
            Some("mp4" | "avi" | "mov" | "mkv") => "🎬",
            Some("mp3" | "wav" | "flac" | "ogg") => "🎵",
            Some("zip" | "tar" | "gz" | "rar" | "7z") => "📦",
            Some("pdf") => "📄",
            Some("exe" | "msi") => "⚡",
            Some("sh" | "bash" | "zsh") => "💻",
            _ => "📃",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    pub timestamp: String,
    pub level: LogLevel,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum LogLevel {
    Info,
    Success,
    Warning,
    Error,
}

impl LogLevel {
    pub fn emoji(&self) -> &'static str {
        match self {
            LogLevel::Info => "ℹ",
            LogLevel::Success => "✅",
            LogLevel::Warning => "⚠",
            LogLevel::Error => "❌",
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ActiveTab {
    FileExplorer,
    BatchOperations,
    FileInfo,
    Logs,
}

#[derive(Debug, Clone, PartialEq)]
pub enum OperationMode {
    None,
    Copy,
    Move,
    Rename,
    CreateFile,
    CreateDir,
    BatchRename,
}

#[derive(Debug, Clone)]
pub struct FileStats {
    pub path: PathBuf,
    pub size: u64,
    pub modified: Option<DateTime<Local>>,
    pub created: Option<DateTime<Local>>,
    pub md5: Option<String>,
    pub permissions: String,
    pub line_count: Option<usize>,
}
