use std::path::{Path, PathBuf};
use chrono::Local;
use eframe::egui;
use serde::{Deserialize, Serialize};
use crate::types::*;
use crate::file_ops;

// ── Persisted config ──────────────────────────────────────────────────────────
#[derive(Serialize, Deserialize, Default)]
struct AppConfig {
    bookmarks: Vec<Bookmark>,
    recent_dirs: Vec<PathBuf>,
}

pub struct FileAssistantApp {
    // Navigation
    pub current_dir: PathBuf,
    pub path_input: String,

    // File tree
    pub file_items: Vec<FileItem>,
    pub selected_items: Vec<PathBuf>,

    // UI state
    pub active_tab: ActiveTab,
    pub operation_mode: OperationMode,
    pub show_hidden: bool,
    pub show_confirm_dialog: bool,
    pub confirm_message: String,
    pub confirm_action: Option<ConfirmAction>,

    // Input fields
    pub new_name_input: String,
    pub new_file_input: String,
    pub new_dir_input: String,
    pub destination_input: String,
    pub search_query: String,
    pub search_results: Vec<PathBuf>,

    // File editor
    pub editor_content: String,
    pub editor_path: Option<PathBuf>,
    pub editor_modified: bool,

    // File info
    pub file_stats: Option<FileStats>,

    // Batch operations
    pub batch_prefix: String,
    pub batch_suffix: String,
    pub batch_find: String,
    pub batch_replace: String,
    pub batch_use_numbering: bool,
    pub batch_start_number: usize,
    pub batch_number_padding: usize,
    pub batch_preview: Vec<(PathBuf, PathBuf)>,

    // Clipboard
    pub clipboard: Vec<PathBuf>,
    pub clipboard_is_cut: bool,

    // Logs
    pub logs: Vec<LogEntry>,
    pub log_scroll_to_bottom: bool,

    // Progress
    pub operation_in_progress: bool,
    pub operation_progress: f32,
    pub status_message: String,

    // Sort
    pub sort_by: SortBy,
    pub sort_ascending: bool,

    // ── New features ─────────────────────────────────────────────────────────
    /// User-defined bookmarks (persisted)
    pub bookmarks: Vec<Bookmark>,
    /// Input for adding a new bookmark name
    pub bookmark_name_input: String,
    /// Whether the add-bookmark inline form is shown
    pub show_add_bookmark: bool,

    /// Recently visited directories (persisted, max 15)
    pub recent_dirs: Vec<PathBuf>,

    /// Whether the right preview panel is visible
    pub show_preview: bool,
    /// Current preview content
    pub preview_content: PreviewContent,
    /// Path currently being previewed (to avoid redundant reads)
    pub preview_path: Option<PathBuf>,

    /// Disk space: (total_bytes, free_bytes)
    pub disk_space: Option<(u64, u64)>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SortBy {
    Name,
    Size,
    Modified,
    Type,
}

#[derive(Debug, Clone)]
pub enum ConfirmAction {
    DeleteItems(Vec<PathBuf>),
    OverwriteFile(PathBuf),
}

// ── Config persistence ────────────────────────────────────────────────────────

fn config_path() -> PathBuf {
    let home = std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("/"));
    home.join(".config").join("file-assistant").join("config.json")
}

fn load_config() -> AppConfig {
    let path = config_path();
    if let Ok(data) = std::fs::read_to_string(&path) {
        serde_json::from_str(&data).unwrap_or_default()
    } else {
        AppConfig::default()
    }
}

fn save_config(cfg: &AppConfig) {
    let path = config_path();
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    if let Ok(data) = serde_json::to_string_pretty(cfg) {
        let _ = std::fs::write(&path, data);
    }
}

impl FileAssistantApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // Load Chinese font support
        let mut fonts = egui::FontDefinitions::default();
        let font_paths = [
            "/usr/share/fonts/truetype/wqy/wqy-microhei.ttc",
            "/usr/share/fonts/wqy/wqy-microhei.ttc",
            "/usr/share/fonts/truetype/wqy/wqy-zenhei.ttc",
            "/usr/share/fonts/opentype/noto/NotoSansCJK-Regular.ttc",
            "/usr/share/fonts/noto-cjk/NotoSansCJK-Regular.ttc",
            "/usr/share/fonts/truetype/noto/NotoSansCJKsc-Regular.otf",
            "/usr/share/fonts/truetype/arphic/uming.ttc",
            "/usr/share/fonts/truetype/arphic/ukai.ttc",
            "C:/Windows/Fonts/msyh.ttc",
            "C:/Windows/Fonts/simsun.ttc",
            "/System/Library/Fonts/PingFang.ttc",
        ];
        for path in &font_paths {
            if let Ok(font_data) = std::fs::read(path) {
                fonts.font_data.insert(
                    "cn_font".to_owned(),
                    egui::FontData::from_owned(font_data),
                );
                if let Some(family) = fonts.families.get_mut(&egui::FontFamily::Proportional) {
                    family.insert(1, "cn_font".to_owned());
                }
                if let Some(family) = fonts.families.get_mut(&egui::FontFamily::Monospace) {
                    family.push("cn_font".to_owned());
                }
                break;
            }
        }
        cc.egui_ctx.set_fonts(fonts);

        // Load persisted config
        let cfg = load_config();

        let current_dir = std::env::current_dir()
            .unwrap_or_else(|_| PathBuf::from(dirs_home()));

        let disk_space = file_ops::get_disk_space(&current_dir);

        let mut app = Self {
            path_input: current_dir.to_string_lossy().to_string(),
            current_dir: current_dir.clone(),
            file_items: Vec::new(),
            selected_items: Vec::new(),
            active_tab: ActiveTab::FileExplorer,
            operation_mode: OperationMode::None,
            show_hidden: false,
            show_confirm_dialog: false,
            confirm_message: String::new(),
            confirm_action: None,
            new_name_input: String::new(),
            new_file_input: String::new(),
            new_dir_input: String::new(),
            destination_input: String::new(),
            search_query: String::new(),
            search_results: Vec::new(),
            editor_content: String::new(),
            editor_path: None,
            editor_modified: false,
            file_stats: None,
            batch_prefix: String::new(),
            batch_suffix: String::new(),
            batch_find: String::new(),
            batch_replace: String::new(),
            batch_use_numbering: false,
            batch_start_number: 1,
            batch_number_padding: 2,
            batch_preview: Vec::new(),
            clipboard: Vec::new(),
            clipboard_is_cut: false,
            logs: Vec::new(),
            log_scroll_to_bottom: false,
            operation_in_progress: false,
            operation_progress: 0.0,
            status_message: "就绪".to_string(),
            sort_by: SortBy::Name,
            sort_ascending: true,
            bookmarks: cfg.bookmarks,
            bookmark_name_input: String::new(),
            show_add_bookmark: false,
            recent_dirs: cfg.recent_dirs,
            show_preview: true,
            preview_content: PreviewContent::None,
            preview_path: None,
            disk_space,
        };

        app.reload_dir();
        app.log(LogLevel::Info, format!("已启动，工作目录：{}", current_dir.display()));
        app
    }

    // ── Config persistence ────────────────────────────────────────────────────

    pub fn persist_config(&self) {
        save_config(&AppConfig {
            bookmarks: self.bookmarks.clone(),
            recent_dirs: self.recent_dirs.clone(),
        });
    }

    // ── Bookmarks ─────────────────────────────────────────────────────────────

    pub fn add_bookmark(&mut self, name: String, path: PathBuf) {
        if self.bookmarks.iter().any(|b| b.path == path) {
            self.log(LogLevel::Warning, format!("书签已存在：{}", path.display()));
            return;
        }
        self.bookmarks.push(Bookmark { name, path: path.clone() });
        self.persist_config();
        self.log(LogLevel::Success, format!("已添加书签：{}", path.display()));
    }

    pub fn remove_bookmark(&mut self, idx: usize) {
        if idx < self.bookmarks.len() {
            let name = self.bookmarks[idx].name.clone();
            self.bookmarks.remove(idx);
            self.persist_config();
            self.log(LogLevel::Info, format!("已删除书签：{}", name));
        }
    }

    pub fn bookmark_current_dir(&mut self) {
        let path = self.current_dir.clone();
        let name = path.file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| path.to_string_lossy().to_string());
        if self.bookmark_name_input.is_empty() {
            self.bookmark_name_input = name;
        }
    }

    // ── Recent directories ────────────────────────────────────────────────────

    fn add_recent_dir(&mut self, path: &PathBuf) {
        self.recent_dirs.retain(|p| p != path);
        self.recent_dirs.insert(0, path.clone());
        if self.recent_dirs.len() > 15 {
            self.recent_dirs.truncate(15);
        }
        self.persist_config();
    }

    // ── File preview ──────────────────────────────────────────────────────────

    pub fn update_preview(&mut self, path: &Path) {
        // Skip if same path already loaded
        if self.preview_path.as_deref() == Some(path) {
            return;
        }
        self.preview_path = Some(path.to_path_buf());

        if path.is_dir() {
            let entries: Vec<(String, String)> = match file_ops::list_dir(path) {
                Ok(list) => list.iter().take(50).map(|p| {
                    let icon = if p.is_dir() { "📁" } else { "📄" }.to_string();
                    let name = p.file_name()
                        .map(|n| n.to_string_lossy().to_string())
                        .unwrap_or_default();
                    (icon, name)
                }).collect(),
                Err(_) => vec![("⚠".to_string(), "无法读取目录".to_string())],
            };
            let total = std::fs::read_dir(path).map(|rd| rd.count()).unwrap_or(0);
            self.preview_content = PreviewContent::Dir { entries, total };
        } else {
            let size = std::fs::metadata(path).map(|m| m.len()).unwrap_or(0);
            let (lines, total_lines, is_text) = file_ops::read_file_preview(path, 80);
            if is_text {
                self.preview_content = PreviewContent::Text { lines, total_lines };
            } else {
                self.preview_content = PreviewContent::Binary { size };
            }
        }
    }

    // ── Logging ───────────────────────────────────────────────────────────────

    pub fn log(&mut self, level: LogLevel, message: String) {
        let entry = LogEntry {
            timestamp: Local::now().format("%H:%M:%S").to_string(),
            level,
            message,
        };
        self.logs.push(entry);
        self.log_scroll_to_bottom = true;
        if self.logs.len() > 1000 {
            self.logs.remove(0);
        }
    }

    pub fn set_status(&mut self, msg: impl Into<String>) {
        self.status_message = msg.into();
    }

    // ── Directory navigation ──────────────────────────────────────────────────

    pub fn navigate_to(&mut self, path: PathBuf) {
        if path.is_dir() {
            self.current_dir = path.clone();
            self.path_input = path.to_string_lossy().to_string();
            self.reload_dir();
            self.selected_items.clear();
            self.preview_content = PreviewContent::None;
            self.preview_path = None;
            self.disk_space = file_ops::get_disk_space(&path);
            self.add_recent_dir(&path);
            self.log(LogLevel::Info, format!("已导航至：{}", path.display()));
        }
    }

    pub fn navigate_up(&mut self) {
        if let Some(parent) = self.current_dir.parent() {
            let parent = parent.to_path_buf();
            self.navigate_to(parent);
        }
    }

    pub fn reload_dir(&mut self) {
        match file_ops::list_dir(&self.current_dir) {
            Ok(entries) => {
                let mut items: Vec<FileItem> = entries
                    .into_iter()
                    .filter(|p| {
                        if !self.show_hidden {
                            !p.file_name()
                                .map(|n| n.to_string_lossy().starts_with('.'))
                                .unwrap_or(false)
                        } else {
                            true
                        }
                    })
                    .map(|p| FileItem::new(p, 0))
                    .collect();

                self.sort_items(&mut items);
                self.file_items = items;
                self.set_status(format!("共 {} 个项目", self.file_items.len()));
            }
            Err(e) => {
                self.log(LogLevel::Error, format!("读取目录失败：{}", e));
                self.set_status("读取目录失败");
            }
        }
    }

    fn sort_items(&self, items: &mut Vec<FileItem>) {
        items.sort_by(|a, b| {
            let a_dir = a.item_type == FileItemType::Directory;
            let b_dir = b.item_type == FileItemType::Directory;
            if a_dir != b_dir {
                return if a_dir { std::cmp::Ordering::Less } else { std::cmp::Ordering::Greater };
            }
            let ord = match self.sort_by {
                SortBy::Name => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
                SortBy::Size => a.size.cmp(&b.size),
                SortBy::Modified => a.modified.cmp(&b.modified),
                SortBy::Type => {
                    let ae = a.path.extension().map(|e| e.to_string_lossy().to_lowercase()).unwrap_or_default();
                    let be = b.path.extension().map(|e| e.to_string_lossy().to_lowercase()).unwrap_or_default();
                    ae.cmp(&be)
                }
            };
            if self.sort_ascending { ord } else { ord.reverse() }
        });
    }

    // ── File operations ───────────────────────────────────────────────────────

    pub fn create_file(&mut self, name: &str) {
        let path = self.current_dir.join(name);
        match file_ops::create_file(&path) {
            Ok(_) => {
                self.log(LogLevel::Success, format!("已创建文件：{}", name));
                self.reload_dir();
            }
            Err(e) => self.log(LogLevel::Error, format!("创建文件失败：{}", e)),
        }
        self.new_file_input.clear();
        self.operation_mode = OperationMode::None;
    }

    pub fn create_dir(&mut self, name: &str) {
        let path = self.current_dir.join(name);
        match file_ops::create_dir(&path) {
            Ok(_) => {
                self.log(LogLevel::Success, format!("已创建文件夹：{}", name));
                self.reload_dir();
            }
            Err(e) => self.log(LogLevel::Error, format!("创建文件夹失败：{}", e)),
        }
        self.new_dir_input.clear();
        self.operation_mode = OperationMode::None;
    }

    pub fn delete_selected(&mut self) {
        if self.selected_items.is_empty() {
            self.log(LogLevel::Warning, "未选择任何项目".to_string());
            return;
        }
        let items = self.selected_items.clone();
        self.confirm_message = format!("确定要删除 {} 个项目吗？此操作无法撤销。", items.len());
        self.confirm_action = Some(ConfirmAction::DeleteItems(items));
        self.show_confirm_dialog = true;
    }

    pub fn perform_delete(&mut self, paths: Vec<PathBuf>) {
        let mut success = 0;
        let mut failed = 0;
        for path in &paths {
            match file_ops::delete_item(path) {
                Ok(_) => {
                    success += 1;
                    self.log(LogLevel::Success, format!("已删除：{}", path.display()));
                }
                Err(e) => {
                    failed += 1;
                    self.log(LogLevel::Error, format!("删除失败 {}：{}", path.display(), e));
                }
            }
        }
        self.log(
            if failed == 0 { LogLevel::Success } else { LogLevel::Warning },
            format!("删除完成：{} 成功，{} 失败", success, failed),
        );
        self.selected_items.clear();
        self.reload_dir();
    }

    pub fn copy_selected(&mut self) {
        if self.selected_items.is_empty() {
            self.log(LogLevel::Warning, "未选择任何项目".to_string());
            return;
        }
        self.clipboard = self.selected_items.clone();
        self.clipboard_is_cut = false;
        self.log(LogLevel::Info, format!("已复制 {} 个项目到剪贴板", self.clipboard.len()));
    }

    pub fn cut_selected(&mut self) {
        if self.selected_items.is_empty() {
            self.log(LogLevel::Warning, "未选择任何项目".to_string());
            return;
        }
        self.clipboard = self.selected_items.clone();
        self.clipboard_is_cut = true;
        self.log(LogLevel::Info, format!("已剪切 {} 个项目到剪贴板", self.clipboard.len()));
    }

    pub fn paste(&mut self) {
        if self.clipboard.is_empty() {
            self.log(LogLevel::Warning, "剪贴板为空".to_string());
            return;
        }
        let items = self.clipboard.clone();
        let is_cut = self.clipboard_is_cut;
        let mut success = 0;
        let mut failed = 0;

        for src in &items {
            let name = src.file_name().unwrap_or_default();
            let dst = self.current_dir.join(name);
            let result = if is_cut {
                file_ops::move_item(src, &dst)
            } else if src.is_dir() {
                file_ops::copy_dir_recursive(src, &dst).map(|_| ())
            } else {
                file_ops::copy_file(src, &dst).map(|_| ())
            };

            match result {
                Ok(_) => {
                    success += 1;
                    self.log(
                        LogLevel::Success,
                        format!("{}：{} → {}", if is_cut { "已移动" } else { "已复制" }, src.display(), dst.display()),
                    );
                }
                Err(e) => {
                    failed += 1;
                    self.log(LogLevel::Error, format!("粘贴失败 {}：{}", src.display(), e));
                }
            }
        }

        if is_cut && failed == 0 { self.clipboard.clear(); }
        self.log(
            if failed == 0 { LogLevel::Success } else { LogLevel::Warning },
            format!("粘贴完成：{} 成功，{} 失败", success, failed),
        );
        self.reload_dir();
    }

    pub fn rename_selected(&mut self, new_name: &str) {
        if self.selected_items.len() != 1 {
            self.log(LogLevel::Warning, "重命名需选择恰好一个项目".to_string());
            return;
        }
        let path = self.selected_items[0].clone();
        match file_ops::rename_item(&path, new_name) {
            Ok(new_path) => {
                self.log(LogLevel::Success, format!("已重命名：{} → {}", path.display(), new_path.display()));
                self.selected_items.clear();
                self.reload_dir();
            }
            Err(e) => self.log(LogLevel::Error, format!("重命名失败：{}", e)),
        }
        self.new_name_input.clear();
        self.operation_mode = OperationMode::None;
    }

    pub fn open_in_editor(&mut self, path: &Path) {
        match file_ops::read_file_text(path) {
            Ok(content) => {
                self.editor_content = content;
                self.editor_path = Some(path.to_path_buf());
                self.editor_modified = false;
                self.active_tab = ActiveTab::FileExplorer;
                self.log(LogLevel::Info, format!("已在编辑器中打开：{}", path.display()));
            }
            Err(e) => self.log(LogLevel::Error, format!("无法打开文件：{}", e)),
        }
    }

    pub fn save_editor(&mut self) {
        if let Some(path) = &self.editor_path.clone() {
            match file_ops::write_file_text(path, &self.editor_content) {
                Ok(_) => {
                    self.editor_modified = false;
                    self.log(LogLevel::Success, format!("已保存：{}", path.display()));
                    self.reload_dir();
                }
                Err(e) => self.log(LogLevel::Error, format!("保存失败：{}", e)),
            }
        }
    }

    pub fn show_file_info(&mut self, path: &Path) {
        match file_ops::get_file_stats(path) {
            Ok(stats) => {
                self.file_stats = Some(stats);
                self.active_tab = ActiveTab::FileInfo;
            }
            Err(e) => self.log(LogLevel::Error, format!("无法获取文件信息：{}", e)),
        }
    }

    pub fn search_files(&mut self) {
        if self.search_query.is_empty() {
            self.search_results.clear();
            return;
        }
        self.search_results = file_ops::search_files(&self.current_dir, &self.search_query, 200);
        self.log(
            LogLevel::Info,
            format!("搜索 '{}'：找到 {} 个结果", self.search_query, self.search_results.len()),
        );
    }

    pub fn update_batch_preview(&mut self) {
        let selected: Vec<PathBuf> = self.selected_items.iter().filter(|p| p.is_file()).cloned().collect();
        if selected.is_empty() {
            self.batch_preview.clear();
            return;
        }
        let opts = file_ops::BatchRenameOptions {
            prefix: self.batch_prefix.clone(),
            suffix: self.batch_suffix.clone(),
            find: self.batch_find.clone(),
            replace: self.batch_replace.clone(),
            use_numbering: self.batch_use_numbering,
            start_number: self.batch_start_number,
            number_padding: self.batch_number_padding,
        };
        match file_ops::batch_rename(&selected, &opts) {
            Ok(preview) => self.batch_preview = preview,
            Err(e) => self.log(LogLevel::Error, format!("批量重命名预览错误：{}", e)),
        }
    }

    pub fn apply_batch_rename(&mut self) {
        if self.batch_preview.is_empty() {
            self.log(LogLevel::Warning, "没有可应用的批量重命名预览".to_string());
            return;
        }
        let results = file_ops::apply_batch_rename(&self.batch_preview);
        let mut success = 0;
        let mut failed = 0;
        for (src, dst, result) in results {
            match result {
                Ok(_) => {
                    success += 1;
                    self.log(LogLevel::Success, format!("已重命名：{} → {}",
                        src.file_name().unwrap_or_default().to_string_lossy(),
                        dst.file_name().unwrap_or_default().to_string_lossy()));
                }
                Err(e) => {
                    failed += 1;
                    self.log(LogLevel::Error, format!("批量重命名失败 {}：{}", src.display(), e));
                }
            }
        }
        self.log(
            if failed == 0 { LogLevel::Success } else { LogLevel::Warning },
            format!("批量重命名：{} 成功，{} 失败", success, failed),
        );
        self.batch_preview.clear();
        self.reload_dir();
    }

    pub fn select_all(&mut self) {
        self.selected_items = self.file_items.iter().map(|i| i.path.clone()).collect();
    }

    pub fn clear_selection(&mut self) {
        self.selected_items.clear();
    }

    pub fn toggle_sort(&mut self, by: SortBy) {
        if self.sort_by == by {
            self.sort_ascending = !self.sort_ascending;
        } else {
            self.sort_by = by;
            self.sort_ascending = true;
        }
        self.reload_dir();
    }
}

fn dirs_home() -> &'static str {
    #[cfg(target_os = "windows")] { "C:\\" }
    #[cfg(not(target_os = "windows"))] { "/" }
}
