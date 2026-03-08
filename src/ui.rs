use eframe::egui::{self, Color32, RichText, Stroke, Vec2};
use crate::app::{ConfirmAction, FileAssistantApp, SortBy};
use crate::file_ops;
use crate::types::*;

// ── Color palette ─────────────────────────────────────────────────────────────
const COL_BG_PANEL: Color32 = Color32::from_rgb(28, 30, 36);
const COL_BG_SIDEBAR: Color32 = Color32::from_rgb(22, 24, 30);
const COL_BG_TOPBAR: Color32 = Color32::from_rgb(18, 20, 26);
const COL_BG_ITEM: Color32 = Color32::from_rgb(36, 39, 46);
const COL_BG_ITEM_HOVER: Color32 = Color32::from_rgb(48, 52, 62);
const COL_BG_ITEM_SEL: Color32 = Color32::from_rgb(45, 95, 170);
const COL_BG_HEADER: Color32 = Color32::from_rgb(30, 33, 40);
const COL_ACCENT: Color32 = Color32::from_rgb(86, 156, 214);
const COL_ACCENT_BRIGHT: Color32 = Color32::from_rgb(120, 180, 240);
const COL_SUCCESS: Color32 = Color32::from_rgb(78, 201, 176);
const COL_WARNING: Color32 = Color32::from_rgb(220, 180, 80);
const COL_ERROR: Color32 = Color32::from_rgb(240, 100, 80);
const COL_TEXT: Color32 = Color32::from_rgb(212, 212, 212);
const COL_TEXT_DIM: Color32 = Color32::from_rgb(140, 140, 150);
const COL_DIR: Color32 = Color32::from_rgb(240, 200, 60);
const COL_SEPARATOR: Color32 = Color32::from_rgb(55, 58, 68);
const COL_BTN: Color32 = Color32::from_rgb(42, 46, 56);

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
        style.visuals.window_rounding = egui::Rounding::same(6.0);
        style.visuals.widgets.noninteractive.rounding = egui::Rounding::same(4.0);
        style.visuals.widgets.inactive.rounding = egui::Rounding::same(4.0);
        style.visuals.widgets.hovered.rounding = egui::Rounding::same(4.0);
        style.visuals.widgets.active.rounding = egui::Rounding::same(4.0);
        ctx.set_style(style);
    }

    // ── Top toolbar ───────────────────────────────────────────────────────────

    fn render_top_bar(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::top("top_bar")
            .exact_height(54.0)
            .frame(
                egui::Frame::none()
                    .fill(COL_BG_TOPBAR)
                    .inner_margin(egui::Margin::symmetric(14.0, 9.0))
                    .stroke(Stroke::new(1.0, COL_SEPARATOR)),
            )
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new("🗂 文件助手")
                            .size(16.0)
                            .color(COL_ACCENT_BRIGHT)
                            .strong(),
                    );
                    ui.add_space(12.0);

                    if ui.add(nav_btn("⬆ 上级")).on_hover_text("返回上级目录").clicked() {
                        self.navigate_up();
                    }
                    if ui.add(nav_btn("🔄 刷新")).on_hover_text("刷新当前目录").clicked() {
                        self.reload_dir();
                    }
                    if ui.add(nav_btn("🏠 主目录")).on_hover_text("返回用户主目录").clicked() {
                        let home = home_dir();
                        self.navigate_to(home);
                    }

                    ui.add(egui::Separator::default().vertical().spacing(10.0));

                    ui.label(RichText::new("路径:").color(COL_TEXT_DIM).size(12.0));
                    let path_resp = ui.add(
                        egui::TextEdit::singleline(&mut self.path_input)
                            .desired_width(380.0)
                            .font(egui::TextStyle::Monospace)
                            .hint_text("输入路径后按 Enter 跳转..."),
                    );
                    if path_resp.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                        let p = std::path::PathBuf::from(&self.path_input);
                        self.navigate_to(p);
                    }

                    ui.add(egui::Separator::default().vertical().spacing(10.0));

                    ui.label(RichText::new("🔍").size(14.0).color(COL_TEXT_DIM));
                    let search_resp = ui.add(
                        egui::TextEdit::singleline(&mut self.search_query)
                            .hint_text("搜索文件名...")
                            .desired_width(200.0),
                    );
                    if search_resp.changed() {
                        self.search_files();
                    }
                    if !self.search_query.is_empty() {
                        if ui
                            .add(
                                egui::Button::new(RichText::new("✕").size(11.0).color(COL_TEXT_DIM))
                                    .fill(Color32::TRANSPARENT)
                                    .stroke(Stroke::NONE)
                                    .min_size(Vec2::new(18.0, 18.0)),
                            )
                            .on_hover_text("清除搜索")
                            .clicked()
                        {
                            self.search_query.clear();
                            self.search_results.clear();
                        }
                    }

                    ui.add(egui::Separator::default().vertical().spacing(10.0));

                    let (hidden_label, hidden_tip) = if self.show_hidden {
                        ("👁 隐藏隐藏文件", "点击隐藏以点开头的文件")
                    } else {
                        ("👁 显示隐藏文件", "点击显示以点开头的文件")
                    };
                    if ui.add(nav_btn(hidden_label)).on_hover_text(hidden_tip).clicked() {
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
            .min_width(180.0)
            .max_width(380.0)
            .frame(
                egui::Frame::none()
                    .fill(COL_BG_SIDEBAR)
                    .inner_margin(egui::Margin::same(0.0))
                    .stroke(Stroke::new(1.0, COL_SEPARATOR)),
            )
            .show(ctx, |ui| {
                egui::ScrollArea::vertical()
                    .auto_shrink([false; 2])
                    .show(ui, |ui| {
                        ui.add_space(6.0);
                        self.render_left_panel(ui);
                        ui.add_space(12.0);
                    });
            });

        egui::CentralPanel::default()
            .frame(
                egui::Frame::none()
                    .fill(COL_BG_PANEL)
                    .inner_margin(egui::Margin::same(0.0)),
            )
            .show(ctx, |ui| {
                self.render_tabs(ui);
                ui.add(egui::Separator::default().horizontal().spacing(0.0));
                match self.active_tab {
                    ActiveTab::FileExplorer => self.render_file_explorer(ui),
                    ActiveTab::BatchOperations => self.render_batch_ops(ui),
                    ActiveTab::FileInfo => self.render_file_info(ui),
                    ActiveTab::Logs => self.render_logs(ui),
                }
            });
    }

    // ── Left panel ────────────────────────────────────────────────────────────

    fn render_left_panel(&mut self, ui: &mut egui::Ui) {
        // ── Quick access ──
        sidebar_section(ui, "⚡ 快速访问");
        let quick_dirs: &[(&str, &str, fn() -> std::path::PathBuf)] = &[
            ("🏠", "主目录", || home_dir()),
            ("🖥", "桌面", || home_dir().join("Desktop")),
            ("📥", "下载", || home_dir().join("Downloads")),
            ("📄", "文档", || home_dir().join("Documents")),
            ("🖼", "图片", || home_dir().join("Pictures")),
            ("🎵", "音乐", || home_dir().join("Music")),
            ("🎬", "视频", || home_dir().join("Videos")),
        ];
        for (icon, label, path_fn) in quick_dirs.iter() {
            let p = path_fn();
            if p.exists() {
                let full_label = format!("{} {}", icon, label);
                let is_current = p == self.current_dir;
                let btn = egui::Button::new(
                    RichText::new(&full_label)
                        .size(13.0)
                        .color(if is_current { COL_ACCENT_BRIGHT } else { COL_TEXT }),
                )
                .fill(if is_current {
                    Color32::from_rgb(30, 55, 90)
                } else {
                    Color32::TRANSPARENT
                })
                .min_size(Vec2::new(ui.available_width() - 8.0, 26.0));
                if ui.add(btn).clicked() {
                    self.navigate_to(p);
                }
            }
        }

        sidebar_divider(ui);

        // ── File operations ──
        sidebar_section(ui, "🛠 文件操作");

        let op_mode = self.operation_mode.clone();

        if ui
            .add(sidebar_btn("📄 新建文件", ui.available_width()))
            .clicked()
        {
            self.operation_mode = if op_mode == OperationMode::CreateFile {
                OperationMode::None
            } else {
                self.new_file_input.clear();
                OperationMode::CreateFile
            };
        }
        if self.operation_mode == OperationMode::CreateFile {
            ui.horizontal(|ui| {
                ui.add_space(8.0);
                let resp = ui.add(
                    egui::TextEdit::singleline(&mut self.new_file_input)
                        .hint_text("文件名.txt")
                        .desired_width(ui.available_width() - 36.0),
                );
                let enter = resp.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));
                if (ui.small_button("✓").clicked() || enter) && !self.new_file_input.is_empty() {
                    let name = self.new_file_input.clone();
                    self.create_file(&name);
                }
            });
        }

        if ui
            .add(sidebar_btn("📁 新建文件夹", ui.available_width()))
            .clicked()
        {
            self.operation_mode = if op_mode == OperationMode::CreateDir {
                OperationMode::None
            } else {
                self.new_dir_input.clear();
                OperationMode::CreateDir
            };
        }
        if self.operation_mode == OperationMode::CreateDir {
            ui.horizontal(|ui| {
                ui.add_space(8.0);
                let resp = ui.add(
                    egui::TextEdit::singleline(&mut self.new_dir_input)
                        .hint_text("文件夹名称")
                        .desired_width(ui.available_width() - 36.0),
                );
                let enter = resp.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));
                if (ui.small_button("✓").clicked() || enter) && !self.new_dir_input.is_empty() {
                    let name = self.new_dir_input.clone();
                    self.create_dir(&name);
                }
            });
        }

        sidebar_divider(ui);

        // ── Clipboard operations ──
        sidebar_section(ui, "📋 剪贴板操作");

        let has_sel = !self.selected_items.is_empty();
        let has_clipboard = !self.clipboard.is_empty();

        ui.add_enabled_ui(has_sel, |ui| {
            if ui
                .add(sidebar_btn("⬆ 复制", ui.available_width()))
                .on_hover_text("复制所选项目到剪贴板 (Ctrl+C)")
                .clicked()
            {
                self.copy_selected();
            }
            if ui
                .add(sidebar_btn("✂ 剪切", ui.available_width()))
                .on_hover_text("剪切所选项目到剪贴板 (Ctrl+X)")
                .clicked()
            {
                self.cut_selected();
            }
        });

        ui.add_enabled_ui(has_clipboard, |ui| {
            let paste_label = if !self.clipboard.is_empty() {
                if self.clipboard_is_cut {
                    format!("📋 粘贴（移动 {} 项）", self.clipboard.len())
                } else {
                    format!("📋 粘贴（复制 {} 项）", self.clipboard.len())
                }
            } else {
                "📋 粘贴".to_string()
            };
            if ui
                .add(sidebar_btn(&paste_label, ui.available_width()))
                .on_hover_text("将剪贴板内容粘贴到当前目录 (Ctrl+V)")
                .clicked()
            {
                self.paste();
            }
        });

        ui.add_enabled_ui(self.selected_items.len() == 1, |ui| {
            if ui
                .add(sidebar_btn("✏ 重命名", ui.available_width()))
                .on_hover_text("重命名选中的项目（需选中一项）(F2)")
                .clicked()
            {
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
        });
        if self.operation_mode == OperationMode::Rename {
            ui.horizontal(|ui| {
                ui.add_space(8.0);
                let resp = ui.add(
                    egui::TextEdit::singleline(&mut self.new_name_input)
                        .desired_width(ui.available_width() - 36.0),
                );
                let enter = resp.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));
                if (ui.small_button("✓").clicked() || enter) && !self.new_name_input.is_empty() {
                    let name = self.new_name_input.clone();
                    self.rename_selected(&name);
                }
            });
        }

        ui.add_enabled_ui(has_sel, |ui| {
            if ui
                .add(
                    egui::Button::new(
                        RichText::new("🗑 删除").size(13.0).color(COL_ERROR),
                    )
                    .fill(Color32::from_rgb(50, 20, 20))
                    .min_size(Vec2::new(ui.available_width() - 8.0, 26.0)),
                )
                .on_hover_text("永久删除选中的项目 (Delete)")
                .clicked()
            {
                self.delete_selected();
            }
        });

        sidebar_divider(ui);

        // ── Selection controls ──
        sidebar_section(ui, "☑ 选择控制");
        ui.horizontal(|ui| {
            ui.add_space(8.0);
            if ui
                .add(
                    egui::Button::new(RichText::new("全选").size(12.0))
                        .fill(COL_BTN)
                        .min_size(Vec2::new(60.0, 24.0)),
                )
                .clicked()
            {
                self.select_all();
            }
            if ui
                .add(
                    egui::Button::new(RichText::new("取消选择").size(12.0))
                        .fill(COL_BTN)
                        .min_size(Vec2::new(72.0, 24.0)),
                )
                .clicked()
            {
                self.clear_selection();
            }
        });
        if !self.selected_items.is_empty() {
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                ui.add_space(8.0);
                ui.label(
                    RichText::new(format!("已选择 {} 项", self.selected_items.len()))
                        .size(12.0)
                        .color(COL_ACCENT),
                );
            });
        }

        // ── Search results ──
        if !self.search_results.is_empty() {
            sidebar_divider(ui);
            sidebar_section(
                ui,
                &format!("🔍 搜索结果（{}）", self.search_results.len()),
            );
            let results = self.search_results.clone();
            egui::ScrollArea::vertical()
                .id_source("search_scroll")
                .max_height(180.0)
                .show(ui, |ui| {
                    for path in &results {
                        let name = path
                            .file_name()
                            .map(|n| n.to_string_lossy().to_string())
                            .unwrap_or_default();
                        let parent_str = path
                            .parent()
                            .map(|p| p.to_string_lossy().to_string())
                            .unwrap_or_default();
                        if ui
                            .add(
                                egui::Button::new(
                                    RichText::new(format!("🔍 {}", name))
                                        .size(12.0)
                                        .color(COL_ACCENT),
                                )
                                .fill(Color32::TRANSPARENT)
                                .min_size(Vec2::new(ui.available_width() - 8.0, 22.0)),
                            )
                            .on_hover_text(&parent_str)
                            .clicked()
                        {
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
        egui::Frame::none()
            .fill(COL_BG_HEADER)
            .inner_margin(egui::Margin::symmetric(8.0, 4.0))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    let tabs = [
                        (ActiveTab::FileExplorer, "📂 文件浏览"),
                        (ActiveTab::BatchOperations, "⚙ 批量操作"),
                        (ActiveTab::FileInfo, "ℹ 文件信息"),
                        (ActiveTab::Logs, "📋 操作日志"),
                    ];
                    for (tab, label) in &tabs {
                        let is_active = &self.active_tab == tab;
                        let btn = egui::Button::new(
                            RichText::new(*label)
                                .size(13.0)
                                .color(if is_active { Color32::WHITE } else { COL_TEXT_DIM }),
                        )
                        .fill(if is_active {
                            COL_BG_ITEM_SEL
                        } else {
                            Color32::TRANSPARENT
                        })
                        .stroke(if is_active {
                            Stroke::new(1.0, COL_ACCENT)
                        } else {
                            Stroke::NONE
                        })
                        .min_size(Vec2::new(0.0, 28.0));
                        if ui.add(btn).clicked() {
                            self.active_tab = tab.clone();
                        }
                    }
                    if !self.logs.is_empty() && self.active_tab != ActiveTab::Logs {
                        ui.label(
                            RichText::new(format!("[{}条]", self.logs.len()))
                                .size(10.0)
                                .color(COL_TEXT_DIM),
                        );
                    }
                });
            });
    }

    // ── File explorer ─────────────────────────────────────────────────────────

    fn render_file_explorer(&mut self, ui: &mut egui::Ui) {
        // Breadcrumb
        egui::Frame::none()
            .fill(Color32::from_rgb(24, 26, 33))
            .inner_margin(egui::Margin::symmetric(10.0, 5.0))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    let parts: Vec<(String, std::path::PathBuf)> = {
                        let mut acc = Vec::new();
                        let mut p = self.current_dir.clone();
                        loop {
                            let name = p
                                .file_name()
                                .map(|n| n.to_string_lossy().to_string())
                                .unwrap_or_else(|| p.to_string_lossy().to_string());
                            acc.push((name, p.clone()));
                            if !p.pop() {
                                break;
                            }
                        }
                        acc.reverse();
                        acc
                    };
                    for (i, (name, path)) in parts.iter().enumerate() {
                        if i > 0 {
                            ui.label(RichText::new("›").color(COL_TEXT_DIM).size(14.0));
                        }
                        let label = if i == parts.len() - 1 {
                            RichText::new(name).color(Color32::WHITE).strong().size(13.0)
                        } else {
                            RichText::new(name).color(COL_ACCENT).size(13.0)
                        };
                        if ui
                            .add(
                                egui::Button::new(label)
                                    .fill(Color32::TRANSPARENT)
                                    .stroke(Stroke::NONE),
                            )
                            .clicked()
                        {
                            let p = path.clone();
                            self.navigate_to(p);
                        }
                    }
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(
                            RichText::new(format!("{} 个项目", self.file_items.len()))
                                .size(11.0)
                                .color(COL_TEXT_DIM),
                        );
                    });
                });
            });

        // Column headers
        egui::Frame::none()
            .fill(COL_BG_HEADER)
            .inner_margin(egui::Margin::symmetric(10.0, 4.0))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    let sort_icon = |by: &SortBy, current: &SortBy, asc: bool| -> &'static str {
                        if by == current {
                            if asc { " ▲" } else { " ▼" }
                        } else {
                            ""
                        }
                    };
                    macro_rules! sort_btn {
                        ($label:expr, $sort:expr, $width:expr) => {
                            if ui
                                .add(
                                    egui::Button::new(
                                        RichText::new(format!(
                                            "{}{}",
                                            $label,
                                            sort_icon(&$sort, &self.sort_by, self.sort_ascending)
                                        ))
                                        .size(12.0)
                                        .color(if self.sort_by == $sort {
                                            COL_ACCENT
                                        } else {
                                            COL_TEXT_DIM
                                        }),
                                    )
                                    .fill(Color32::TRANSPARENT)
                                    .min_size(Vec2::new($width, 0.0)),
                                )
                                .clicked()
                            {
                                self.toggle_sort($sort);
                            }
                        };
                    }
                    sort_btn!("名称", SortBy::Name, 300.0);
                    sort_btn!("大小", SortBy::Size, 80.0);
                    sort_btn!("修改时间", SortBy::Modified, 160.0);
                    sort_btn!("类型", SortBy::Type, 60.0);
                });
            });

        // File list
        let items_clone: Vec<_> = self
            .file_items
            .iter()
            .map(|i| {
                (
                    i.path.clone(),
                    i.name.clone(),
                    i.item_type.clone(),
                    i.size,
                    i.modified.map(|m| m.format("%Y-%m-%d %H:%M").to_string()),
                    i.icon().to_string(),
                )
            })
            .collect();

        if items_clone.is_empty() {
            ui.add_space(60.0);
            ui.centered_and_justified(|ui| {
                ui.label(RichText::new("📭 当前目录为空").size(16.0).color(COL_TEXT_DIM));
            });
            return;
        }

        egui::ScrollArea::vertical()
            .id_source("file_list_scroll")
            .auto_shrink([false; 2])
            .show(ui, |ui| {
                if self.editor_path.is_some() {
                    self.render_editor_inline(ui);
                    ui.add_space(4.0);
                    ui.add(egui::Separator::default().horizontal());
                }

                for (path, name, item_type, size, modified, icon) in &items_clone {
                    let is_selected = self.selected_items.contains(path);
                    let bg = if is_selected {
                        COL_BG_ITEM_SEL
                    } else {
                        Color32::TRANSPARENT
                    };

                    let row = egui::Frame::none()
                        .fill(bg)
                        .rounding(3.0)
                        .inner_margin(egui::Margin::symmetric(10.0, 3.0))
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                let name_color = if *item_type == FileItemType::Directory {
                                    COL_DIR
                                } else {
                                    COL_TEXT
                                };
                                ui.add_sized(
                                    Vec2::new(300.0, 20.0),
                                    egui::Label::new(
                                        RichText::new(format!("{} {}", icon, name))
                                            .size(13.0)
                                            .color(name_color),
                                    )
                                    .truncate(true),
                                );

                                let size_str = if *item_type == FileItemType::Directory {
                                    "—".to_string()
                                } else {
                                    file_ops::format_size(*size)
                                };
                                ui.add_sized(
                                    Vec2::new(80.0, 20.0),
                                    egui::Label::new(
                                        RichText::new(&size_str).size(12.0).color(COL_TEXT_DIM),
                                    ),
                                );

                                let mod_str = modified.as_deref().unwrap_or("—");
                                ui.add_sized(
                                    Vec2::new(160.0, 20.0),
                                    egui::Label::new(
                                        RichText::new(mod_str).size(12.0).color(COL_TEXT_DIM),
                                    ),
                                );

                                let ext = path
                                    .extension()
                                    .map(|e| e.to_string_lossy().to_uppercase())
                                    .unwrap_or_else(|| {
                                        if *item_type == FileItemType::Directory {
                                            "目录".into()
                                        } else {
                                            "—".into()
                                        }
                                    });
                                ui.label(
                                    RichText::new(ext.as_str())
                                        .size(11.0)
                                        .color(COL_TEXT_DIM),
                                );
                            });
                        });

                    let resp = row.response.interact(egui::Sense::click());

                    if resp.hovered() && !is_selected {
                        ui.painter().rect_filled(resp.rect, 3.0, COL_BG_ITEM_HOVER);
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
                            if let Some(last) = self.selected_items.last().cloned() {
                                let paths: Vec<_> =
                                    items_clone.iter().map(|(p, ..)| p.clone()).collect();
                                if let (Some(i1), Some(i2)) = (
                                    paths.iter().position(|p| p == &last),
                                    paths.iter().position(|p| p == path),
                                ) {
                                    let (lo, hi) =
                                        if i1 < i2 { (i1, i2) } else { (i2, i1) };
                                    for p in &paths[lo..=hi] {
                                        if !self.selected_items.contains(p) {
                                            self.selected_items.push(p.clone());
                                        }
                                    }
                                }
                            }
                        } else {
                            self.selected_items = vec![path.clone()];
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
                            let ext = path
                                .extension()
                                .map(|e| e.to_string_lossy().to_lowercase())
                                .unwrap_or_default();
                            let text_exts = [
                                "txt", "md", "rs", "py", "js", "ts", "html", "css", "json",
                                "toml", "yaml", "yml", "sh", "log", "xml", "csv", "ini", "cfg",
                                "conf", "env",
                            ];
                            if text_exts.iter().any(|&e| e == ext.as_str()) {
                                let p = path.clone();
                                self.open_in_editor(&p);
                            } else {
                                let _ = open::that(path);
                                self.log(
                                    LogLevel::Info,
                                    format!("已用系统程序打开：{}", path.display()),
                                );
                            }
                        }
                    }

                    resp.context_menu(|ui| {
                        let p = path.clone();
                        ui.set_min_width(170.0);
                        if ui.add(ctx_menu_item("📋 复制")).clicked() {
                            if !self.selected_items.contains(&p) {
                                self.selected_items = vec![p.clone()];
                            }
                            self.copy_selected();
                            ui.close_menu();
                        }
                        if ui.add(ctx_menu_item("✂ 剪切")).clicked() {
                            if !self.selected_items.contains(&p) {
                                self.selected_items = vec![p.clone()];
                            }
                            self.cut_selected();
                            ui.close_menu();
                        }
                        if ui.add(ctx_menu_item("📋 粘贴")).clicked() {
                            self.paste();
                            ui.close_menu();
                        }
                        ui.separator();
                        if ui.add(ctx_menu_item("✏ 重命名")).clicked() {
                            self.selected_items = vec![p.clone()];
                            self.new_name_input = p
                                .file_name()
                                .unwrap_or_default()
                                .to_string_lossy()
                                .to_string();
                            self.operation_mode = OperationMode::Rename;
                            ui.close_menu();
                        }
                        if *item_type != FileItemType::Directory {
                            if ui.add(ctx_menu_item("✏ 在编辑器中打开")).clicked() {
                                self.open_in_editor(&p);
                                ui.close_menu();
                            }
                            if ui.add(ctx_menu_item("🚀 用系统程序打开")).clicked() {
                                let _ = open::that(&p);
                                ui.close_menu();
                            }
                        }
                        ui.separator();
                        if ui.add(ctx_menu_item("ℹ 查看属性")).clicked() {
                            self.show_file_info(&p);
                            ui.close_menu();
                        }
                        ui.separator();
                        if ui
                            .button(RichText::new("🗑 删除").color(COL_ERROR).size(13.0))
                            .clicked()
                        {
                            self.selected_items = vec![p];
                            self.delete_selected();
                            ui.close_menu();
                        }
                    });
                }
            });
    }

    fn render_editor_inline(&mut self, ui: &mut egui::Ui) {
        let path_str = self
            .editor_path
            .as_ref()
            .map(|p| p.file_name().unwrap_or_default().to_string_lossy().to_string())
            .unwrap_or_default();

        egui::Frame::none()
            .fill(Color32::from_rgb(18, 20, 26))
            .rounding(6.0)
            .stroke(Stroke::new(1.0, COL_SEPARATOR))
            .inner_margin(egui::Margin::same(10.0))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new("✏ 编辑器")
                            .color(COL_ACCENT)
                            .strong()
                            .size(13.0),
                    );
                    ui.label(RichText::new(&path_str).size(12.0).color(COL_TEXT_DIM));
                    if self.editor_modified {
                        ui.label(RichText::new("● 未保存").size(11.0).color(COL_WARNING));
                    }
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.add(nav_btn("✕ 关闭")).clicked() {
                            self.editor_path = None;
                            self.editor_content.clear();
                        }
                        let save_label = if self.editor_modified {
                            RichText::new("💾 保存*").color(COL_WARNING)
                        } else {
                            RichText::new("💾 保存").color(COL_TEXT)
                        };
                        if ui.button(save_label).on_hover_text("保存文件 (Ctrl+S)").clicked() {
                            self.save_editor();
                        }
                    });
                });
                ui.add_space(6.0);
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
            ui.add_space(12.0);
            ui.horizontal(|ui| {
                ui.add_space(16.0);
                ui.vertical(|ui| {
                    ui.label(
                        RichText::new("⚙ 批量重命名")
                            .size(17.0)
                            .color(COL_ACCENT)
                            .strong(),
                    );
                    ui.add_space(4.0);
                    let sel_count = self.selected_items.iter().filter(|p| p.is_file()).count();
                    let hint_color = if sel_count == 0 { COL_WARNING } else { COL_SUCCESS };
                    ui.label(
                        RichText::new(format!(
                            "已选择 {} 个文件（仅处理文件，不处理文件夹）",
                            sel_count
                        ))
                        .color(hint_color)
                        .size(12.0),
                    );
                    ui.add_space(12.0);

                    egui::Frame::none()
                        .fill(COL_BG_ITEM)
                        .rounding(8.0)
                        .inner_margin(egui::Margin::same(16.0))
                        .show(ui, |ui| {
                            egui::Grid::new("batch_grid")
                                .num_columns(2)
                                .spacing([16.0, 10.0])
                                .show(ui, |ui| {
                                    ui.label(RichText::new("前缀:").color(COL_TEXT_DIM));
                                    ui.add(
                                        egui::TextEdit::singleline(&mut self.batch_prefix)
                                            .hint_text("在文件名前添加")
                                            .desired_width(220.0),
                                    );
                                    ui.end_row();

                                    ui.label(RichText::new("后缀:").color(COL_TEXT_DIM));
                                    ui.add(
                                        egui::TextEdit::singleline(&mut self.batch_suffix)
                                            .hint_text("在扩展名前添加")
                                            .desired_width(220.0),
                                    );
                                    ui.end_row();

                                    ui.label(RichText::new("查找:").color(COL_TEXT_DIM));
                                    ui.add(
                                        egui::TextEdit::singleline(&mut self.batch_find)
                                            .hint_text("要查找的文本")
                                            .desired_width(220.0),
                                    );
                                    ui.end_row();

                                    ui.label(RichText::new("替换:").color(COL_TEXT_DIM));
                                    ui.add(
                                        egui::TextEdit::singleline(&mut self.batch_replace)
                                            .hint_text("替换为（留空则删除）")
                                            .desired_width(220.0),
                                    );
                                    ui.end_row();

                                    ui.label(RichText::new("序号:").color(COL_TEXT_DIM));
                                    ui.horizontal(|ui| {
                                        ui.checkbox(&mut self.batch_use_numbering, "启用");
                                        if self.batch_use_numbering {
                                            ui.label(
                                                RichText::new("起始:").color(COL_TEXT_DIM),
                                            );
                                            ui.add(
                                                egui::DragValue::new(
                                                    &mut self.batch_start_number,
                                                )
                                                .speed(1.0)
                                                .clamp_range(0..=9999),
                                            );
                                            ui.label(
                                                RichText::new("位数:").color(COL_TEXT_DIM),
                                            );
                                            ui.add(
                                                egui::DragValue::new(
                                                    &mut self.batch_number_padding,
                                                )
                                                .speed(1.0)
                                                .clamp_range(1..=8),
                                            );
                                        }
                                    });
                                    ui.end_row();
                                });
                        });

                    ui.add_space(14.0);
                    ui.horizontal(|ui| {
                        if ui
                            .add(
                                egui::Button::new(RichText::new("👁 预览").size(14.0))
                                    .fill(COL_BTN)
                                    .min_size(Vec2::new(100.0, 32.0)),
                            )
                            .clicked()
                        {
                            self.update_batch_preview();
                        }
                        ui.add_space(8.0);
                        ui.add_enabled_ui(!self.batch_preview.is_empty(), |ui| {
                            if ui
                                .add(
                                    egui::Button::new(
                                        RichText::new("✅ 应用重命名")
                                            .size(14.0)
                                            .color(Color32::WHITE),
                                    )
                                    .fill(Color32::from_rgb(35, 110, 55))
                                    .min_size(Vec2::new(130.0, 32.0)),
                                )
                                .clicked()
                            {
                                self.apply_batch_rename();
                            }
                        });
                    });

                    if !self.batch_preview.is_empty() {
                        ui.add_space(14.0);
                        ui.label(
                            RichText::new(format!(
                                "预览（共 {} 项）:",
                                self.batch_preview.len()
                            ))
                            .color(COL_ACCENT)
                            .strong(),
                        );
                        ui.add_space(6.0);
                        egui::Frame::none()
                            .fill(COL_BG_ITEM)
                            .rounding(6.0)
                            .inner_margin(egui::Margin::same(10.0))
                            .show(ui, |ui| {
                                egui::ScrollArea::vertical()
                                    .max_height(300.0)
                                    .show(ui, |ui| {
                                        let preview = self.batch_preview.clone();
                                        for (src, dst) in &preview {
                                            let src_name = src
                                                .file_name()
                                                .unwrap_or_default()
                                                .to_string_lossy();
                                            let dst_name = dst
                                                .file_name()
                                                .unwrap_or_default()
                                                .to_string_lossy();
                                            ui.horizontal(|ui| {
                                                ui.label(
                                                    RichText::new(src_name.as_ref())
                                                        .size(12.0)
                                                        .color(COL_TEXT_DIM),
                                                );
                                                ui.label(
                                                    RichText::new(" → ").color(COL_ACCENT),
                                                );
                                                ui.label(
                                                    RichText::new(dst_name.as_ref())
                                                        .size(12.0)
                                                        .color(COL_SUCCESS),
                                                );
                                            });
                                        }
                                    });
                            });
                    } else if sel_count == 0 {
                        ui.add_space(20.0);
                        ui.label(
                            RichText::new(
                                "💡 请先在「文件浏览」中选择文件，然后在此设置重命名规则",
                            )
                            .size(12.0)
                            .color(COL_TEXT_DIM),
                        );
                    }
                });
            });
        });
    }

    // ── File info panel ───────────────────────────────────────────────────────

    fn render_file_info(&mut self, ui: &mut egui::Ui) {
        if let Some(stats) = self.file_stats.clone() {
            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.add_space(16.0);
                ui.horizontal(|ui| {
                    ui.add_space(20.0);
                    ui.vertical(|ui| {
                        ui.label(
                            RichText::new("ℹ 文件详细信息")
                                .size(17.0)
                                .color(COL_ACCENT)
                                .strong(),
                        );
                        ui.add_space(4.0);
                        let file_name = stats
                            .path
                            .file_name()
                            .map(|n| n.to_string_lossy().to_string())
                            .unwrap_or_default();
                        ui.label(
                            RichText::new(&file_name)
                                .size(14.0)
                                .color(Color32::WHITE)
                                .strong(),
                        );
                        ui.add_space(14.0);

                        let rows: Vec<(&str, String)> = vec![
                            ("📂 完整路径", stats.path.to_string_lossy().to_string()),
                            ("📏 文件大小", file_ops::format_size(stats.size)),
                            (
                                "📅 修改时间",
                                stats
                                    .modified
                                    .map(|d| d.format("%Y-%m-%d %H:%M:%S").to_string())
                                    .unwrap_or_else(|| "未知".to_string()),
                            ),
                            (
                                "🗓 创建时间",
                                stats
                                    .created
                                    .map(|d| d.format("%Y-%m-%d %H:%M:%S").to_string())
                                    .unwrap_or_else(|| "未知".to_string()),
                            ),
                            ("🔒 文件权限", stats.permissions.clone()),
                            (
                                "🔑 MD5 校验",
                                stats
                                    .md5
                                    .clone()
                                    .unwrap_or_else(|| "—（文件过大，已跳过）".to_string()),
                            ),
                            (
                                "📝 文本行数",
                                stats
                                    .line_count
                                    .map(|c| format!("{} 行", c))
                                    .unwrap_or_else(|| "—（非文本文件）".to_string()),
                            ),
                        ];

                        egui::Frame::none()
                            .fill(COL_BG_ITEM)
                            .rounding(8.0)
                            .inner_margin(egui::Margin::same(18.0))
                            .show(ui, |ui| {
                                egui::Grid::new("file_info_grid")
                                    .num_columns(2)
                                    .spacing([24.0, 12.0])
                                    .show(ui, |ui| {
                                        for (label, value) in &rows {
                                            ui.label(
                                                RichText::new(*label)
                                                    .color(COL_TEXT_DIM)
                                                    .size(13.0),
                                            );
                                            ui.label(
                                                RichText::new(value).color(COL_TEXT).size(13.0),
                                            );
                                            ui.end_row();
                                        }
                                    });
                            });

                        ui.add_space(16.0);
                        if ui
                            .add(
                                egui::Button::new(
                                    RichText::new("🔄 重新计算 MD5").size(13.0),
                                )
                                .fill(COL_BTN)
                                .min_size(Vec2::new(160.0, 30.0)),
                            )
                            .on_hover_text("重新计算文件的 MD5 哈希值（大文件较慢）")
                            .clicked()
                        {
                            let path = stats.path.clone();
                            if let Ok(new_stats) = file_ops::get_file_stats(&path) {
                                self.file_stats = Some(new_stats);
                            }
                        }
                    });
                });
            });
        } else {
            ui.add_space(80.0);
            ui.centered_and_justified(|ui| {
                ui.vertical_centered(|ui| {
                    ui.label(RichText::new("ℹ").size(48.0).color(COL_TEXT_DIM));
                    ui.add_space(12.0);
                    ui.label(
                        RichText::new("单击文件即可在此查看详细信息")
                            .size(14.0)
                            .color(COL_TEXT_DIM),
                    );
                    ui.add_space(6.0);
                    ui.label(
                        RichText::new("也可右键文件选择「查看属性」")
                            .size(12.0)
                            .color(COL_TEXT_DIM),
                    );
                });
            });
        }
    }

    // ── Logs panel ────────────────────────────────────────────────────────────

    fn render_logs(&mut self, ui: &mut egui::Ui) {
        egui::Frame::none()
            .fill(COL_BG_HEADER)
            .inner_margin(egui::Margin::symmetric(12.0, 6.0))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new(format!("📋 操作日志（共 {} 条）", self.logs.len()))
                            .size(14.0)
                            .color(COL_ACCENT)
                            .strong(),
                    );
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui
                            .add(nav_btn("🗑 清空"))
                            .on_hover_text("清空所有日志记录")
                            .clicked()
                        {
                            self.logs.clear();
                        }
                        if ui.add(nav_btn("⬇ 滚到底部")).clicked() {
                            self.log_scroll_to_bottom = true;
                        }
                    });
                });
            });

        let scroll_bottom = self.log_scroll_to_bottom;
        self.log_scroll_to_bottom = false;

        if self.logs.is_empty() {
            ui.add_space(60.0);
            ui.centered_and_justified(|ui| {
                ui.label(
                    RichText::new("暂无操作记录")
                        .size(14.0)
                        .color(COL_TEXT_DIM),
                );
            });
            return;
        }

        egui::ScrollArea::vertical()
            .auto_shrink([false; 2])
            .stick_to_bottom(scroll_bottom)
            .show(ui, |ui| {
                ui.add_space(4.0);
                let logs = self.logs.clone();
                for entry in &logs {
                    let color = match entry.level {
                        LogLevel::Info => COL_TEXT,
                        LogLevel::Success => COL_SUCCESS,
                        LogLevel::Warning => COL_WARNING,
                        LogLevel::Error => COL_ERROR,
                    };
                    egui::Frame::none()
                        .inner_margin(egui::Margin::symmetric(12.0, 2.0))
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                ui.label(
                                    RichText::new(&entry.timestamp)
                                        .size(11.0)
                                        .color(COL_TEXT_DIM)
                                        .monospace(),
                                );
                                ui.add_space(4.0);
                                ui.label(RichText::new(entry.level.emoji()).size(12.0));
                                ui.label(
                                    RichText::new(&entry.message).size(12.0).color(color),
                                );
                            });
                        });
                }
                ui.add_space(4.0);
            });
    }

    // ── Status bar ────────────────────────────────────────────────────────────

    fn render_status_bar(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::bottom("status_bar")
            .exact_height(28.0)
            .frame(
                egui::Frame::none()
                    .fill(Color32::from_rgb(14, 16, 22))
                    .inner_margin(egui::Margin::symmetric(14.0, 5.0))
                    .stroke(Stroke::new(1.0, COL_SEPARATOR)),
            )
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new(&self.status_message)
                            .size(11.0)
                            .color(COL_TEXT_DIM),
                    );

                    if !self.selected_items.is_empty() {
                        ui.add(egui::Separator::default().vertical().spacing(6.0));
                        ui.label(
                            RichText::new(format!("已选 {} 项", self.selected_items.len()))
                                .size(11.0)
                                .color(COL_ACCENT),
                        );
                    }

                    if !self.clipboard.is_empty() {
                        ui.add(egui::Separator::default().vertical().spacing(6.0));
                        let op = if self.clipboard_is_cut { "待移动" } else { "待粘贴" };
                        ui.label(
                            RichText::new(format!(
                                "剪贴板：{} 项（{}）",
                                self.clipboard.len(),
                                op
                            ))
                            .size(11.0)
                            .color(COL_TEXT_DIM),
                        );
                    }

                    if !self.search_query.is_empty() {
                        ui.add(egui::Separator::default().vertical().spacing(6.0));
                        ui.label(
                            RichText::new(format!(
                                "🔍 \"{}\"：{} 个结果",
                                self.search_query,
                                self.search_results.len()
                            ))
                            .size(11.0)
                            .color(COL_WARNING),
                        );
                    }

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(
                            RichText::new(self.current_dir.to_string_lossy().as_ref())
                                .size(11.0)
                                .color(COL_TEXT_DIM)
                                .monospace(),
                        );
                    });
                });
            });
    }

    // ── Confirm dialog ────────────────────────────────────────────────────────

    fn render_confirm_dialog(&mut self, ctx: &egui::Context) {
        if !self.show_confirm_dialog {
            return;
        }

        let message = self.confirm_message.clone();
        let mut do_confirm = false;
        let mut do_cancel = false;

        egui::Window::new("⚠ 操作确认")
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .frame(
                egui::Frame::window(&ctx.style())
                    .fill(Color32::from_rgb(32, 34, 42))
                    .stroke(Stroke::new(1.0, COL_WARNING))
                    .rounding(8.0),
            )
            .show(ctx, |ui| {
                ui.add_space(10.0);
                ui.label(RichText::new("⚠").size(28.0).color(COL_WARNING));
                ui.add_space(6.0);
                ui.label(RichText::new(&message).size(14.0));
                ui.add_space(20.0);
                ui.horizontal(|ui| {
                    if ui
                        .add(
                            egui::Button::new(
                                RichText::new("✅ 确认删除").color(Color32::WHITE).size(14.0),
                            )
                            .fill(Color32::from_rgb(180, 40, 40))
                            .min_size(Vec2::new(110.0, 34.0)),
                        )
                        .clicked()
                    {
                        do_confirm = true;
                    }
                    ui.add_space(12.0);
                    if ui
                        .add(
                            egui::Button::new(RichText::new("取消").size(14.0))
                                .fill(COL_BTN)
                                .min_size(Vec2::new(80.0, 34.0)),
                        )
                        .clicked()
                    {
                        do_cancel = true;
                    }
                });
                ui.add_space(6.0);
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

fn nav_btn(label: &str) -> egui::Button {
    egui::Button::new(RichText::new(label).size(12.5))
        .fill(COL_BTN)
        .stroke(Stroke::new(1.0, COL_SEPARATOR))
}

fn sidebar_btn(label: &str, available_width: f32) -> egui::Button {
    egui::Button::new(RichText::new(label).size(13.0))
        .fill(COL_BG_ITEM)
        .min_size(Vec2::new(available_width - 8.0, 26.0))
}

fn ctx_menu_item(label: &str) -> egui::Button {
    egui::Button::new(RichText::new(label).size(13.0))
        .fill(Color32::TRANSPARENT)
        .min_size(Vec2::new(150.0, 24.0))
}

fn sidebar_section(ui: &mut egui::Ui, title: &str) {
    ui.horizontal(|ui| {
        ui.add_space(8.0);
        ui.label(RichText::new(title).size(11.0).color(COL_TEXT_DIM).strong());
    });
    ui.add_space(2.0);
}

fn sidebar_divider(ui: &mut egui::Ui) {
    ui.add_space(6.0);
    ui.add(egui::Separator::default().horizontal().spacing(4.0));
    ui.add_space(4.0);
}

fn home_dir() -> std::path::PathBuf {
    std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|_| std::path::PathBuf::from("/"))
}
