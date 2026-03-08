use eframe::egui::{self, Color32, RichText, Stroke, Vec2};
use crate::app::{ConfirmAction, FileAssistantApp, SortBy};
use crate::file_ops;
use crate::types::*;

// ── Color palette ─────────────────────────────────────────────────────────────
const COL_BG_PANEL: Color32 = Color32::from_rgb(28, 30, 36);
const COL_BG_ITEM: Color32 = Color32::from_rgb(36, 39, 46);
const COL_BG_ITEM_HOVER: Color32 = Color32::from_rgb(48, 52, 62);
const COL_BG_ITEM_SEL: Color32 = Color32::from_rgb(45, 95, 170);
const COL_ACCENT: Color32 = Color32::from_rgb(86, 156, 214);
const COL_SUCCESS: Color32 = Color32::from_rgb(78, 201, 176);
const COL_WARNING: Color32 = Color32::from_rgb(220, 180, 80);
const COL_ERROR: Color32 = Color32::from_rgb(240, 100, 80);
const COL_TEXT: Color32 = Color32::from_rgb(212, 212, 212);
const COL_TEXT_DIM: Color32 = Color32::from_rgb(140, 140, 150);
const COL_DIR: Color32 = Color32::from_rgb(240, 200, 60);
const COL_SEPARATOR: Color32 = Color32::from_rgb(55, 58, 68);

impl eframe::App for FileAssistantApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.apply_theme(ctx);
        self.render_top_bar(ctx);
        self.render_main_area(ctx);
        self.render_status_bar(ctx);
        self.render_confirm_dialog(ctx);
    }
}

impl FileAssistantApp {
    fn apply_theme(&self, ctx: &egui::Context) {
        let mut style = (*ctx.style()).clone();
        style.visuals.window_fill = COL_BG_PANEL;
        style.visuals.panel_fill = COL_BG_PANEL;
        style.visuals.extreme_bg_color = COL_BG_ITEM;
        style.visuals.widgets.noninteractive.bg_fill = COL_BG_ITEM;
        style.visuals.widgets.inactive.bg_fill = COL_BG_ITEM;
        style.visuals.widgets.hovered.bg_fill = COL_BG_ITEM_HOVER;
        style.visuals.widgets.active.bg_fill = COL_BG_ITEM_SEL;
        style.visuals.selection.bg_fill = COL_BG_ITEM_SEL;
        style.visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0, COL_TEXT);
        style.visuals.widgets.inactive.fg_stroke = Stroke::new(1.0, COL_TEXT);
        style.visuals.widgets.hovered.fg_stroke = Stroke::new(1.0, Color32::WHITE);
        style.visuals.override_text_color = Some(COL_TEXT);
        style.spacing.item_spacing = Vec2::new(6.0, 4.0);
        style.spacing.button_padding = Vec2::new(10.0, 5.0);
        ctx.set_style(style);
    }

    // ── Top toolbar ───────────────────────────────────────────────────────────

    fn render_top_bar(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::top("top_bar")
            .exact_height(52.0)
            .frame(egui::Frame::none()
                .fill(Color32::from_rgb(20, 22, 28))
                .inner_margin(egui::Margin::symmetric(12.0, 8.0)))
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    // Logo
                    ui.label(RichText::new("🗂 File Assistant").size(16.0).color(COL_ACCENT).strong());
                    ui.add_space(16.0);

                    // Navigation buttons
                    if ui.add(btn_icon("⬆ Up")).clicked() {
                        self.navigate_up();
                    }
                    if ui.add(btn_icon("🔄 Refresh")).clicked() {
                        self.reload_dir();
                    }
                    if ui.add(btn_icon("🏠 Home")).clicked() {
                        let home = home_dir();
                        self.navigate_to(home);
                    }

                    ui.separator();

                    // Path bar
                    ui.label(RichText::new("Path:").color(COL_TEXT_DIM));
                    let path_resp = ui.add(
                        egui::TextEdit::singleline(&mut self.path_input)
                            .desired_width(400.0)
                            .font(egui::TextStyle::Monospace),
                    );
                    if path_resp.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                        let p = std::path::PathBuf::from(&self.path_input);
                        self.navigate_to(p);
                    }

                    ui.separator();

                    // Search
                    ui.label(RichText::new("🔍").size(14.0));
                    let search_resp = ui.add(
                        egui::TextEdit::singleline(&mut self.search_query)
                            .hint_text("Search files...")
                            .desired_width(180.0),
                    );
                    if search_resp.changed() {
                        self.search_files();
                    }

                    ui.separator();

                    // Hidden files toggle
                    let hidden_label = if self.show_hidden { "👁 Hide Hidden" } else { "👁 Show Hidden" };
                    if ui.add(btn_icon(hidden_label)).clicked() {
                        self.show_hidden = !self.show_hidden;
                        self.reload_dir();
                    }
                });
            });
    }

    // ── Main area (sidebar + content) ─────────────────────────────────────────

    fn render_main_area(&mut self, ctx: &egui::Context) {
        egui::SidePanel::left("left_panel")
            .resizable(true)
            .default_width(240.0)
            .min_width(160.0)
            .max_width(400.0)
            .frame(egui::Frame::none()
                .fill(Color32::from_rgb(22, 24, 30))
                .inner_margin(egui::Margin::same(0.0)))
            .show(ctx, |ui| {
                self.render_left_panel(ui);
            });

        egui::CentralPanel::default()
            .frame(egui::Frame::none()
                .fill(COL_BG_PANEL)
                .inner_margin(egui::Margin::same(0.0)))
            .show(ctx, |ui| {
                self.render_tabs(ui);
                ui.separator();
                match self.active_tab {
                    ActiveTab::FileExplorer => self.render_file_explorer(ui),
                    ActiveTab::BatchOperations => self.render_batch_ops(ui),
                    ActiveTab::FileInfo => self.render_file_info(ui),
                    ActiveTab::Logs => self.render_logs(ui),
                }
            });
    }

    // ── Left panel (quick access + actions) ───────────────────────────────────

    fn render_left_panel(&mut self, ui: &mut egui::Ui) {
        // Quick access
        section_header(ui, "⚡ Quick Access");
        let quick_dirs: Vec<(&str, fn() -> std::path::PathBuf)> = vec![
            ("🏠 Home", || home_dir()),
            ("🖥 Desktop", || home_dir().join("Desktop")),
            ("📥 Downloads", || home_dir().join("Downloads")),
            ("📄 Documents", || home_dir().join("Documents")),
            ("🖼 Pictures", || home_dir().join("Pictures")),
        ];
        for (label, path_fn) in quick_dirs {
            let p = path_fn();
            if p.exists() {
                if ui.add(egui::Button::new(RichText::new(label).size(13.0))
                    .fill(Color32::TRANSPARENT)
                    .min_size(Vec2::new(220.0, 24.0))).clicked()
                {
                    self.navigate_to(p);
                }
            }
        }

        ui.add_space(8.0);
        ui.add(egui::Separator::default().horizontal());
        ui.add_space(4.0);

        // Actions panel
        section_header(ui, "🛠 Actions");

        let op_mode = self.operation_mode.clone();

        // Create file
        if ui.add(action_btn("📄 New File")).clicked() {
            self.operation_mode = if op_mode == OperationMode::CreateFile {
                OperationMode::None
            } else {
                OperationMode::CreateFile
            };
        }
        if self.operation_mode == OperationMode::CreateFile {
            ui.horizontal(|ui| {
                ui.add(egui::TextEdit::singleline(&mut self.new_file_input)
                    .hint_text("filename.txt")
                    .desired_width(140.0));
                if ui.small_button("✓").clicked() && !self.new_file_input.is_empty() {
                    let name = self.new_file_input.clone();
                    self.create_file(&name);
                }
            });
        }

        // Create directory
        if ui.add(action_btn("📁 New Folder")).clicked() {
            self.operation_mode = if op_mode == OperationMode::CreateDir {
                OperationMode::None
            } else {
                OperationMode::CreateDir
            };
        }
        if self.operation_mode == OperationMode::CreateDir {
            ui.horizontal(|ui| {
                ui.add(egui::TextEdit::singleline(&mut self.new_dir_input)
                    .hint_text("folder_name")
                    .desired_width(140.0));
                if ui.small_button("✓").clicked() && !self.new_dir_input.is_empty() {
                    let name = self.new_dir_input.clone();
                    self.create_dir(&name);
                }
            });
        }

        ui.add_space(4.0);
        ui.add(egui::Separator::default().horizontal());
        ui.add_space(4.0);

        // Selection actions
        section_header(ui, "📋 Selection");

        if ui.add(action_btn("⬆ Copy")).clicked() { self.copy_selected(); }
        if ui.add(action_btn("✂ Cut")).clicked() { self.cut_selected(); }

        let has_clipboard = !self.clipboard.is_empty();
        ui.add_enabled_ui(has_clipboard, |ui| {
            if ui.add(action_btn("📋 Paste")).clicked() { self.paste(); }
        });

        // Rename
        if ui.add(action_btn("✏ Rename")).clicked() {
            self.operation_mode = if op_mode == OperationMode::Rename {
                OperationMode::None
            } else {
                if self.selected_items.len() == 1 {
                    self.new_name_input = self.selected_items[0]
                        .file_name()
                        .unwrap_or_default()
                        .to_string_lossy()
                        .to_string();
                }
                OperationMode::Rename
            };
        }
        if self.operation_mode == OperationMode::Rename {
            ui.horizontal(|ui| {
                ui.add(egui::TextEdit::singleline(&mut self.new_name_input)
                    .desired_width(140.0));
                if ui.small_button("✓").clicked() && !self.new_name_input.is_empty() {
                    let name = self.new_name_input.clone();
                    self.rename_selected(&name);
                }
            });
        }

        if ui.add(action_btn("🗑 Delete").fill(Color32::from_rgb(120, 40, 40))).clicked() {
            self.delete_selected();
        }

        ui.add_space(4.0);
        ui.add(egui::Separator::default().horizontal());
        ui.add_space(4.0);

        // Select
        section_header(ui, "☑ Selection");
        ui.horizontal(|ui| {
            if ui.small_button("All").clicked() { self.select_all(); }
            if ui.small_button("None").clicked() { self.clear_selection(); }
        });

        ui.add_space(4.0);
        // Selection summary
        if !self.selected_items.is_empty() {
            ui.label(RichText::new(format!("{} selected", self.selected_items.len()))
                .size(11.0).color(COL_ACCENT));
        }

        // Search results
        if !self.search_results.is_empty() {
            ui.add_space(8.0);
            ui.add(egui::Separator::default().horizontal());
            section_header(ui, "🔍 Search Results");
            let results = self.search_results.clone();
            egui::ScrollArea::vertical()
                .id_source("search_scroll")
                .max_height(200.0)
                .show(ui, |ui| {
                    for path in &results {
                        let name = path.file_name()
                            .map(|n| n.to_string_lossy().to_string())
                            .unwrap_or_default();
                        if ui.add(
                            egui::Button::new(RichText::new(&name).size(12.0).color(COL_ACCENT))
                                .fill(Color32::TRANSPARENT)
                        ).on_hover_text(path.to_string_lossy())
                        .clicked() {
                            if let Some(parent) = path.parent() {
                                let parent = parent.to_path_buf();
                                self.navigate_to(parent);
                            }
                        }
                    }
                });
        }
    }

    // ── Tab bar ───────────────────────────────────────────────────────────────

    fn render_tabs(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.add_space(8.0);
            let tabs = [
                (ActiveTab::FileExplorer, "📂 Explorer"),
                (ActiveTab::BatchOperations, "⚙ Batch Ops"),
                (ActiveTab::FileInfo, "ℹ File Info"),
                (ActiveTab::Logs, "📋 Logs"),
            ];
            for (tab, label) in &tabs {
                let is_active = &self.active_tab == tab;
                let btn = egui::Button::new(
                    RichText::new(*label)
                        .size(13.0)
                        .color(if is_active { Color32::WHITE } else { COL_TEXT_DIM }),
                )
                .fill(if is_active { COL_BG_ITEM_SEL } else { Color32::TRANSPARENT })
                .stroke(Stroke::NONE);
                if ui.add(btn).clicked() {
                    self.active_tab = tab.clone();
                }
            }
        });
    }

    // ── File explorer ─────────────────────────────────────────────────────────

    fn render_file_explorer(&mut self, ui: &mut egui::Ui) {
        // Breadcrumb
        ui.horizontal(|ui| {
            ui.add_space(8.0);
            let parts: Vec<(String, std::path::PathBuf)> = {
                let mut parts = Vec::new();
                let mut p = self.current_dir.clone();
                let mut acc = Vec::new();
                loop {
                    let name = p.file_name()
                        .map(|n| n.to_string_lossy().to_string())
                        .unwrap_or_else(|| p.to_string_lossy().to_string());
                    acc.push((name, p.clone()));
                    if !p.pop() { break; }
                }
                acc.reverse();
                parts = acc;
                parts
            };
            for (i, (name, path)) in parts.iter().enumerate() {
                if i > 0 { ui.label(RichText::new("›").color(COL_TEXT_DIM)); }
                let label = if i == parts.len() - 1 {
                    RichText::new(name).color(Color32::WHITE).strong()
                } else {
                    RichText::new(name).color(COL_ACCENT)
                };
                if ui.add(egui::Button::new(label).fill(Color32::TRANSPARENT)).clicked() {
                    let p = path.clone();
                    self.navigate_to(p);
                }
            }
        });

        ui.add_space(4.0);

        // Column headers
        egui::Frame::none()
            .fill(Color32::from_rgb(32, 34, 42))
            .inner_margin(egui::Margin::symmetric(8.0, 4.0))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    let sort_icon = |by: &SortBy, current: &SortBy, asc: bool| -> &'static str {
                        if by == current { if asc { " ▲" } else { " ▼" } } else { "" }
                    };
                    if ui.add(egui::Button::new(
                        RichText::new(format!("Name{}", sort_icon(&SortBy::Name, &self.sort_by, self.sort_ascending)))
                            .size(12.0).color(COL_TEXT_DIM)
                    ).fill(Color32::TRANSPARENT).min_size(Vec2::new(300.0, 0.0))).clicked() {
                        self.toggle_sort(SortBy::Name);
                    }
                    if ui.add(egui::Button::new(
                        RichText::new(format!("Size{}", sort_icon(&SortBy::Size, &self.sort_by, self.sort_ascending)))
                            .size(12.0).color(COL_TEXT_DIM)
                    ).fill(Color32::TRANSPARENT).min_size(Vec2::new(80.0, 0.0))).clicked() {
                        self.toggle_sort(SortBy::Size);
                    }
                    if ui.add(egui::Button::new(
                        RichText::new(format!("Modified{}", sort_icon(&SortBy::Modified, &self.sort_by, self.sort_ascending)))
                            .size(12.0).color(COL_TEXT_DIM)
                    ).fill(Color32::TRANSPARENT).min_size(Vec2::new(160.0, 0.0))).clicked() {
                        self.toggle_sort(SortBy::Modified);
                    }
                    ui.label(RichText::new("Type").size(12.0).color(COL_TEXT_DIM));
                });
            });

        // File list
        let scroll_id = egui::Id::new("file_list_scroll");
        let items_clone: Vec<_> = self.file_items.iter().map(|i| {
            (i.path.clone(), i.name.clone(), i.item_type.clone(),
             i.size, i.modified.map(|m| m.format("%Y-%m-%d %H:%M").to_string()),
             i.icon().to_string())
        }).collect();

        egui::ScrollArea::vertical()
            .id_source(scroll_id)
            .auto_shrink([false; 2])
            .show(ui, |ui| {
                // Editor panel (if open)
                if self.editor_path.is_some() {
                    self.render_editor_inline(ui);
                    ui.add_space(4.0);
                    ui.add(egui::Separator::default().horizontal());
                }

                for (path, name, item_type, size, modified, icon) in &items_clone {
                    let is_selected = self.selected_items.contains(path);
                    let bg = if is_selected { COL_BG_ITEM_SEL } else { Color32::TRANSPARENT };

                    let row = egui::Frame::none()
                        .fill(bg)
                        .rounding(4.0)
                        .inner_margin(egui::Margin::symmetric(8.0, 3.0))
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                // Icon + name
                                let name_color = if *item_type == FileItemType::Directory {
                                    COL_DIR
                                } else {
                                    COL_TEXT
                                };
                                let label = RichText::new(format!("{} {}", icon, name))
                                    .size(13.0)
                                    .color(name_color);
                                ui.add_sized(Vec2::new(300.0, 18.0),
                                    egui::Label::new(label).truncate(true));

                                // Size
                                let size_str = if *item_type == FileItemType::Directory {
                                    "—".to_string()
                                } else {
                                    file_ops::format_size(*size)
                                };
                                ui.add_sized(Vec2::new(80.0, 18.0),
                                    egui::Label::new(RichText::new(&size_str).size(12.0).color(COL_TEXT_DIM)));

                                // Modified
                                let mod_str = modified.as_deref().unwrap_or("—");
                                ui.add_sized(Vec2::new(160.0, 18.0),
                                    egui::Label::new(RichText::new(mod_str).size(12.0).color(COL_TEXT_DIM)));

                                // Extension
                                let ext = path.extension()
                                    .map(|e| e.to_string_lossy().to_uppercase())
                                    .unwrap_or_else(|| if *item_type == FileItemType::Directory { "DIR".into() } else { "—".into() });
                                ui.label(RichText::new(ext.as_str()).size(11.0).color(COL_TEXT_DIM));
                            });
                        });

                    let resp = row.response;
                    let resp = resp.interact(egui::Sense::click());

                    // Hover highlight
                    if resp.hovered() && !is_selected {
                        ui.painter().rect_filled(resp.rect, 4.0, COL_BG_ITEM_HOVER);
                    }

                    if resp.clicked() {
                        let ctrl = ui.input(|i| i.modifiers.ctrl || i.modifiers.command);
                        let shift = ui.input(|i| i.modifiers.shift);
                        if ctrl {
                            if is_selected {
                                self.selected_items.retain(|p| p != path);
                            } else {
                                self.selected_items.push(path.clone());
                            }
                        } else if shift && !self.selected_items.is_empty() {
                            // Range select
                            if let Some(last) = self.selected_items.last().cloned() {
                                let paths: Vec<_> = items_clone.iter().map(|(p, ..)| p.clone()).collect();
                                if let (Some(i1), Some(i2)) = (
                                    paths.iter().position(|p| p == &last),
                                    paths.iter().position(|p| p == path),
                                ) {
                                    let (lo, hi) = if i1 < i2 { (i1, i2) } else { (i2, i1) };
                                    for p in &paths[lo..=hi] {
                                        if !self.selected_items.contains(p) {
                                            self.selected_items.push(p.clone());
                                        }
                                    }
                                }
                            }
                        } else {
                            self.selected_items = vec![path.clone()];
                            // Load info sidebar
                            if *item_type != FileItemType::Directory {
                                let p = path.clone();
                                if let Ok(stats) = file_ops::get_file_stats(&p) {
                                    self.file_stats = Some(stats);
                                }
                            }
                        }
                    }

                    if resp.double_clicked() {
                        if *item_type == FileItemType::Directory {
                            let p = path.clone();
                            self.navigate_to(p);
                        } else {
                            // Open in editor if text-like, else open with OS
                            let ext = path.extension()
                                .map(|e| e.to_string_lossy().to_lowercase())
                                .unwrap_or_default();
                            let text_exts = ["txt","md","rs","py","js","ts","html","css","json","toml","yaml","yml","sh","log","xml","csv","ini","cfg","conf","env"];
                            if text_exts.iter().any(|&e| e == ext.as_str()) {
                                let p = path.clone();
                                self.open_in_editor(&p);
                            } else {
                                let _ = open::that(path);
                                self.log(LogLevel::Info, format!("Opened with OS: {}", path.display()));
                            }
                        }
                    }

                    // Right-click context menu
                    resp.context_menu(|ui| {
                        let p = path.clone();
                        if ui.button("📋 Copy").clicked() {
                            if !self.selected_items.contains(&p) {
                                self.selected_items = vec![p.clone()];
                            }
                            self.copy_selected();
                            ui.close_menu();
                        }
                        if ui.button("✂ Cut").clicked() {
                            if !self.selected_items.contains(&p) {
                                self.selected_items = vec![p.clone()];
                            }
                            self.cut_selected();
                            ui.close_menu();
                        }
                        if ui.button("📋 Paste").clicked() {
                            self.paste();
                            ui.close_menu();
                        }
                        ui.separator();
                        if ui.button("✏ Rename").clicked() {
                            self.selected_items = vec![p.clone()];
                            self.new_name_input = p.file_name()
                                .unwrap_or_default()
                                .to_string_lossy()
                                .to_string();
                            self.operation_mode = OperationMode::Rename;
                            ui.close_menu();
                        }
                        ui.separator();
                        if ui.button("ℹ Properties").clicked() {
                            self.show_file_info(&p);
                            ui.close_menu();
                        }
                        if *item_type != FileItemType::Directory {
                            if ui.button("✏ Edit").clicked() {
                                self.open_in_editor(&p);
                                ui.close_menu();
                            }
                            if ui.button("🚀 Open with OS").clicked() {
                                let _ = open::that(&p);
                                ui.close_menu();
                            }
                        }
                        ui.separator();
                        if ui.button(RichText::new("🗑 Delete").color(COL_ERROR)).clicked() {
                            self.selected_items = vec![p];
                            self.delete_selected();
                            ui.close_menu();
                        }
                    });
                }
            });
    }

    fn render_editor_inline(&mut self, ui: &mut egui::Ui) {
        let path_str = self.editor_path.as_ref()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_default();

        egui::Frame::none()
            .fill(Color32::from_rgb(20, 22, 28))
            .rounding(6.0)
            .inner_margin(egui::Margin::same(8.0))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(RichText::new("✏ Editor:").color(COL_ACCENT).strong());
                    ui.label(RichText::new(&path_str).size(11.0).color(COL_TEXT_DIM));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.add(btn_icon("✕ Close")).clicked() {
                            self.editor_path = None;
                            self.editor_content.clear();
                        }
                        let save_label = if self.editor_modified {
                            RichText::new("💾 Save*").color(COL_WARNING)
                        } else {
                            RichText::new("💾 Save").color(COL_TEXT)
                        };
                        if ui.button(save_label).clicked() {
                            self.save_editor();
                        }
                    });
                });
                ui.add_space(4.0);
                let te = egui::TextEdit::multiline(&mut self.editor_content)
                    .font(egui::FontId::monospace(12.5))
                    .desired_rows(18)
                    .desired_width(f32::INFINITY)
                    .code_editor();
                if ui.add(te).changed() {
                    self.editor_modified = true;
                }
            });
    }

    // ── Batch operations ──────────────────────────────────────────────────────

    fn render_batch_ops(&mut self, ui: &mut egui::Ui) {
        egui::ScrollArea::vertical().show(ui, |ui| {
            ui.add_space(8.0);
            ui.horizontal(|ui| {
                ui.add_space(12.0);
                ui.vertical(|ui| {
                    ui.label(RichText::new("⚙ Batch Rename").size(16.0).color(COL_ACCENT).strong());
                    ui.add_space(4.0);
                    ui.label(RichText::new(format!("{} file(s) selected", self.selected_items.len()))
                        .color(COL_TEXT_DIM));
                    ui.add_space(12.0);

                    egui::Grid::new("batch_grid")
                        .num_columns(2)
                        .spacing([16.0, 8.0])
                        .show(ui, |ui| {
                            ui.label("Prefix:");
                            ui.add(egui::TextEdit::singleline(&mut self.batch_prefix)
                                .hint_text("add before name").desired_width(200.0));
                            ui.end_row();

                            ui.label("Suffix:");
                            ui.add(egui::TextEdit::singleline(&mut self.batch_suffix)
                                .hint_text("add after name (before ext)").desired_width(200.0));
                            ui.end_row();

                            ui.label("Find:");
                            ui.add(egui::TextEdit::singleline(&mut self.batch_find)
                                .hint_text("text to find").desired_width(200.0));
                            ui.end_row();

                            ui.label("Replace:");
                            ui.add(egui::TextEdit::singleline(&mut self.batch_replace)
                                .hint_text("replacement text").desired_width(200.0));
                            ui.end_row();

                            ui.label("Numbering:");
                            ui.horizontal(|ui| {
                                ui.checkbox(&mut self.batch_use_numbering, "Enable");
                                if self.batch_use_numbering {
                                    ui.label("Start:");
                                    ui.add(egui::DragValue::new(&mut self.batch_start_number).speed(1.0).clamp_range(0..=9999));
                                    ui.label("Padding:");
                                    ui.add(egui::DragValue::new(&mut self.batch_number_padding).speed(1.0).clamp_range(1..=8));
                                }
                            });
                            ui.end_row();
                        });

                    ui.add_space(12.0);
                    ui.horizontal(|ui| {
                        if ui.button(RichText::new("👁 Preview").size(14.0)).clicked() {
                            self.update_batch_preview();
                        }
                        ui.add_space(8.0);
                        if ui.add(
                            egui::Button::new(RichText::new("✅ Apply").size(14.0).color(Color32::WHITE))
                                .fill(Color32::from_rgb(40, 120, 60))
                        ).clicked() {
                            self.apply_batch_rename();
                        }
                    });

                    if !self.batch_preview.is_empty() {
                        ui.add_space(12.0);
                        ui.label(RichText::new("Preview:").color(COL_ACCENT).strong());
                        ui.add_space(4.0);
                        egui::Frame::none()
                            .fill(COL_BG_ITEM)
                            .rounding(6.0)
                            .inner_margin(egui::Margin::same(8.0))
                            .show(ui, |ui| {
                                egui::ScrollArea::vertical()
                                    .max_height(300.0)
                                    .show(ui, |ui| {
                                        let preview = self.batch_preview.clone();
                                        for (src, dst) in &preview {
                                            let src_name = src.file_name().unwrap_or_default().to_string_lossy();
                                            let dst_name = dst.file_name().unwrap_or_default().to_string_lossy();
                                            ui.horizontal(|ui| {
                                                ui.label(RichText::new(src_name.as_ref()).size(12.0).color(COL_TEXT_DIM));
                                                ui.label(RichText::new("→").color(COL_ACCENT));
                                                ui.label(RichText::new(dst_name.as_ref()).size(12.0).color(COL_SUCCESS));
                                            });
                                        }
                                    });
                            });
                    }
                });
            });
        });
    }

    // ── File info panel ───────────────────────────────────────────────────────

    fn render_file_info(&mut self, ui: &mut egui::Ui) {
        if let Some(stats) = self.file_stats.clone() {
            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.add_space(12.0);
                ui.horizontal(|ui| {
                    ui.add_space(16.0);
                    ui.vertical(|ui| {
                        ui.label(RichText::new("ℹ File Information").size(16.0).color(COL_ACCENT).strong());
                        ui.add_space(12.0);

                        let rows: Vec<(&str, String)> = vec![
                            ("📂 Path", stats.path.to_string_lossy().to_string()),
                            ("📏 Size", file_ops::format_size(stats.size)),
                            ("📅 Modified", stats.modified.map(|d| d.format("%Y-%m-%d %H:%M:%S").to_string()).unwrap_or_else(|| "Unknown".to_string())),
                            ("🗓 Created", stats.created.map(|d| d.format("%Y-%m-%d %H:%M:%S").to_string()).unwrap_or_else(|| "Unknown".to_string())),
                            ("🔒 Permissions", stats.permissions.clone()),
                            ("🔑 MD5", stats.md5.clone().unwrap_or_else(|| "— (file too large)".to_string())),
                            ("📝 Lines", stats.line_count.map(|c| c.to_string()).unwrap_or_else(|| "—".to_string())),
                        ];

                        egui::Frame::none()
                            .fill(COL_BG_ITEM)
                            .rounding(8.0)
                            .inner_margin(egui::Margin::same(16.0))
                            .show(ui, |ui| {
                                egui::Grid::new("file_info_grid")
                                    .num_columns(2)
                                    .spacing([24.0, 10.0])
                                    .show(ui, |ui| {
                                        for (label, value) in &rows {
                                            ui.label(RichText::new(*label).color(COL_TEXT_DIM).size(13.0));
                                            ui.label(RichText::new(value).color(COL_TEXT).size(13.0));
                                            ui.end_row();
                                        }
                                    });
                            });

                        ui.add_space(16.0);
                        if ui.button("🔄 Recompute MD5").clicked() {
                            let path = stats.path.clone();
                            if let Ok(new_stats) = file_ops::get_file_stats(&path) {
                                self.file_stats = Some(new_stats);
                            }
                        }
                    });
                });
            });
        } else {
            ui.centered_and_justified(|ui| {
                ui.label(RichText::new("Select a file to view its information").color(COL_TEXT_DIM));
            });
        }
    }

    // ── Logs panel ────────────────────────────────────────────────────────────

    fn render_logs(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.add_space(8.0);
            ui.label(RichText::new(format!("📋 Operation Logs ({} entries)", self.logs.len()))
                .size(14.0).color(COL_ACCENT).strong());
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.add_space(8.0);
                if ui.button("🗑 Clear").clicked() {
                    self.logs.clear();
                }
                if ui.button("⬇ Scroll Bottom").clicked() {
                    self.log_scroll_to_bottom = true;
                }
            });
        });
        ui.add_space(4.0);

        let scroll_bottom = self.log_scroll_to_bottom;
        self.log_scroll_to_bottom = false;

        egui::ScrollArea::vertical()
            .auto_shrink([false; 2])
            .stick_to_bottom(scroll_bottom)
            .show(ui, |ui| {
                let logs = self.logs.clone();
                for entry in &logs {
                    let color = match entry.level {
                        LogLevel::Info => COL_TEXT,
                        LogLevel::Success => COL_SUCCESS,
                        LogLevel::Warning => COL_WARNING,
                        LogLevel::Error => COL_ERROR,
                    };
                    ui.horizontal(|ui| {
                        ui.label(RichText::new(&entry.timestamp).size(11.0).color(COL_TEXT_DIM).monospace());
                        ui.add_space(4.0);
                        ui.label(RichText::new(entry.level.emoji()).size(12.0));
                        ui.label(RichText::new(&entry.message).size(12.0).color(color));
                    });
                }
            });
    }

    // ── Status bar ────────────────────────────────────────────────────────────

    fn render_status_bar(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::bottom("status_bar")
            .exact_height(26.0)
            .frame(egui::Frame::none()
                .fill(Color32::from_rgb(14, 16, 22))
                .inner_margin(egui::Margin::symmetric(12.0, 4.0)))
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label(RichText::new(&self.status_message).size(11.0).color(COL_TEXT_DIM));
                    ui.add_space(16.0);
                    if !self.selected_items.is_empty() {
                        ui.label(RichText::new(format!("| {} selected", self.selected_items.len()))
                            .size(11.0).color(COL_ACCENT));
                    }
                    if !self.clipboard.is_empty() {
                        let op = if self.clipboard_is_cut { "cut" } else { "copied" };
                        ui.label(RichText::new(format!("| {} item(s) {}", self.clipboard.len(), op))
                            .size(11.0).color(COL_TEXT_DIM));
                    }
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(RichText::new(&self.current_dir.to_string_lossy().as_ref().to_string())
                            .size(11.0).color(COL_TEXT_DIM).monospace());
                    });
                });
            });
    }

    // ── Confirm dialog ────────────────────────────────────────────────────────

    fn render_confirm_dialog(&mut self, ctx: &egui::Context) {
        if !self.show_confirm_dialog { return; }

        let message = self.confirm_message.clone();
        let mut do_confirm = false;
        let mut do_cancel = false;

        egui::Window::new("⚠ Confirm")
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ctx, |ui| {
                ui.add_space(8.0);
                ui.label(RichText::new(&message).size(14.0));
                ui.add_space(16.0);
                ui.horizontal(|ui| {
                    if ui.add(
                        egui::Button::new(RichText::new("✅ Confirm").color(Color32::WHITE))
                            .fill(Color32::from_rgb(180, 40, 40))
                            .min_size(Vec2::new(100.0, 32.0))
                    ).clicked() {
                        do_confirm = true;
                    }
                    ui.add_space(16.0);
                    if ui.add(
                        egui::Button::new("Cancel")
                            .min_size(Vec2::new(80.0, 32.0))
                    ).clicked() {
                        do_cancel = true;
                    }
                });
            });

        if do_confirm {
            if let Some(action) = self.confirm_action.take() {
                match action {
                    ConfirmAction::DeleteItems(paths) => self.perform_delete(paths),
                    ConfirmAction::OverwriteFile(_) => {}
                }
            }
            self.show_confirm_dialog = false;
        } else if do_cancel {
            self.confirm_action = None;
            self.show_confirm_dialog = false;
        }
    }
}

// ── Widget helpers ────────────────────────────────────────────────────────────

fn btn_icon(label: &str) -> egui::Button {
    egui::Button::new(RichText::new(label).size(12.5))
        .fill(Color32::from_rgb(42, 46, 56))
        .stroke(Stroke::new(1.0, COL_SEPARATOR))
}

fn action_btn(label: &str) -> egui::Button {
    egui::Button::new(RichText::new(label).size(12.5))
        .fill(COL_BG_ITEM)
        .min_size(Vec2::new(220.0, 26.0))
}

fn section_header(ui: &mut egui::Ui, title: &str) {
    ui.add_space(4.0);
    ui.label(RichText::new(title).size(11.0).color(COL_TEXT_DIM).strong());
    ui.add_space(2.0);
}

fn home_dir() -> std::path::PathBuf {
    std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|_| std::path::PathBuf::from("/"))
}
