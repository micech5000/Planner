pub mod grades;
pub mod lessons;
pub mod schedule;
pub mod settings;
pub mod sleep;
pub mod tasks;
pub mod today;
pub mod widget;

use crate::glass::{
    self, DwmTracker, GlassSurfaceSpec, Icon, Theme, apply_egui_visuals, draw_glass_surface,
    draw_icon, draw_liquid_pill, icon_button, lerp_color, semibold,
};
use crate::inspect::Inspector;
use crate::model::{
    Data, Event, Grade, Night, Task, demo_data, ru_date_long, save_data, storage, today,
};
use crate::motion::{Motion, SpringParams, system_reduce_motion};
use chrono::Datelike;
use eframe::egui::{self, Color32, CornerRadius, FontId, Pos2, Rect, RichText, Sense, Stroke, Vec2};
use std::{fs, path::PathBuf};
use tray_icon::{
    TrayIcon, TrayIconBuilder,
    menu::{Menu, MenuEvent, MenuItem},
};

pub const SEC_TODAY: usize = 0;
pub const SEC_LESSONS: usize = 1;
pub const SEC_SCHEDULE: usize = 2;
pub const SEC_TASKS: usize = 3;
pub const SEC_GRADES: usize = 4;
pub const SEC_SLEEP: usize = 5;
pub const SEC_SETTINGS: usize = 6;

pub const SECTIONS: [(&str, Icon); 7] = [
    ("Сегодня", Icon::Sun),
    ("Уроки", Icon::Lessons),
    ("Расписание", Icon::Calendar),
    ("Задачи", Icon::Checklist),
    ("Оценки", Icon::Book),
    ("Сон", Icon::Moon),
    ("Настройки", Icon::Sliders),
];

pub struct PlannerApp {
    pub d: Data,
    pub section: usize,
    pub event: Event,
    pub task: Task,
    pub grade: Grade,
    pub night: Night,
    pub quick_task_title: String,
    pub quick_task_minutes: i64,
    pub lesson_day: usize,
    pub custom_subject_input: String,
    pub lesson_wizard_dismissed: bool,
    pub edit_event: Option<usize>,
    pub edit_task: Option<usize>,
    pub edit_grade: Option<usize>,
    pub edit_night: Option<usize>,
    pub show_event_form: bool,
    pub show_completed_tasks: bool,
    pub override_date_input: String,
    pub message: String,
    pub message_is_error: bool,
    pub path: PathBuf,
    pub last: String,
    pub load_failed: bool,
    pub undo: Option<Data>,
    pub tray: Option<TrayIcon>,
    pub open_id: String,
    pub widget_id: String,
    pub quit_id: String,
    pub quit: bool,
    pub selected_day: usize,
    pub motion: Motion,
    pub dwm: DwmTracker,
    pub system_reduce_motion: bool,
    pub inspector: Option<Inspector>,
}

impl PlannerApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let mut fonts = egui::FontDefinitions::default();
        let mac_font_paths = [
            "/System/Library/Fonts/SFPro.ttf",
            "/System/Library/Fonts/SFNS.ttf",
            "/System/Library/Fonts/SFNSDisplay.ttf",
            "/System/Library/Fonts/SFNSText.ttf",
            "/Library/Fonts/SF-Pro.ttf",
            "/System/Library/Fonts/HelveticaNeue.ttc",
            "/System/Library/Fonts/Helvetica.ttc",
        ];
        let mut loaded_primary = false;
        for path in mac_font_paths {
            if let Ok(bytes) = fs::read(path) {
                fonts
                    .font_data
                    .insert("SystemRegular".into(), egui::FontData::from_owned(bytes).into());
                fonts
                    .families
                    .entry(egui::FontFamily::Proportional)
                    .or_default()
                    .insert(0, "SystemRegular".into());
                loaded_primary = true;
                break;
            }
        }
        if !loaded_primary {
            if let Ok(bytes) = fs::read("C:/Windows/Fonts/segoeui.ttf") {
                fonts
                    .font_data
                    .insert("SegoeRegular".into(), egui::FontData::from_owned(bytes).into());
                fonts
                    .families
                    .entry(egui::FontFamily::Proportional)
                    .or_default()
                    .insert(0, "SegoeRegular".into());
            }
        }

        let mac_semibold_paths = [
            "/System/Library/Fonts/SFProSemibold.ttf",
            "/System/Library/Fonts/SFNSSymbols.ttf",
            "/System/Library/Fonts/HelveticaNeue-Medium.ttf",
        ];
        let mut loaded_semibold = false;
        for path in mac_semibold_paths {
            if let Ok(bytes) = fs::read(path) {
                fonts
                    .font_data
                    .insert("SystemSemibold".into(), egui::FontData::from_owned(bytes).into());
                let mut list = vec!["SystemSemibold".to_owned()];
                if let Some(prop) = fonts.families.get(&egui::FontFamily::Proportional) {
                    list.extend(prop.iter().cloned());
                }
                fonts
                    .families
                    .insert(egui::FontFamily::Name("Semibold".into()), list);
                loaded_semibold = true;
                break;
            }
        }
        if !loaded_semibold {
            if let Ok(bytes) = fs::read("C:/Windows/Fonts/seguisb.ttf") {
                fonts
                    .font_data
                    .insert("SegoeSemibold".into(), egui::FontData::from_owned(bytes).into());
                let mut list = vec!["SegoeSemibold".to_owned()];
                if let Some(prop) = fonts.families.get(&egui::FontFamily::Proportional) {
                    list.extend(prop.iter().cloned());
                }
                fonts
                    .families
                    .insert(egui::FontFamily::Name("Semibold".into()), list);
            }
        }

        let mac_bold_paths = [
            "/System/Library/Fonts/SFProBold.ttf",
            "/System/Library/Fonts/HelveticaNeue-Bold.ttf",
        ];
        let mut loaded_bold = false;
        for path in mac_bold_paths {
            if let Ok(bytes) = fs::read(path) {
                fonts
                    .font_data
                    .insert("SystemBold".into(), egui::FontData::from_owned(bytes).into());
                let mut list = vec!["SystemBold".to_owned()];
                if let Some(prop) = fonts.families.get(&egui::FontFamily::Proportional) {
                    list.extend(prop.iter().cloned());
                }
                fonts
                    .families
                    .insert(egui::FontFamily::Name("Bold".into()), list);
                loaded_bold = true;
                break;
            }
        }
        if !loaded_bold {
            if let Ok(bytes) = fs::read("C:/Windows/Fonts/segoeuib.ttf") {
                fonts
                    .font_data
                    .insert("SegoeBold".into(), egui::FontData::from_owned(bytes).into());
                let mut list = vec!["SegoeBold".to_owned()];
                if let Some(prop) = fonts.families.get(&egui::FontFamily::Proportional) {
                    list.extend(prop.iter().cloned());
                }
                fonts
                    .families
                    .insert(egui::FontFamily::Name("Bold".into()), list);
            }
        }

        let mac_mono_paths = [
            "/System/Library/Fonts/SFMono.ttf",
            "/System/Library/Fonts/SF-Mono.ttf",
            "/System/Library/Fonts/Menlo.ttc",
            "/System/Library/Fonts/Monaco.dfont",
            "/System/Library/Fonts/Monaco.ttf",
        ];
        let mut loaded_mono = false;
        for path in mac_mono_paths {
            if let Ok(bytes) = fs::read(path) {
                fonts
                    .font_data
                    .insert("SystemMono".into(), egui::FontData::from_owned(bytes).into());
                fonts
                    .families
                    .entry(egui::FontFamily::Monospace)
                    .or_default()
                    .insert(0, "SystemMono".into());
                loaded_mono = true;
                break;
            }
        }
        if !loaded_mono {
            if let Ok(bytes) = fs::read("C:/Windows/Fonts/consola.ttf") {
                fonts
                    .font_data
                    .insert("Consolas".into(), egui::FontData::from_owned(bytes).into());
                fonts
                    .families
                    .entry(egui::FontFamily::Monospace)
                    .or_default()
                    .insert(0, "Consolas".into());
            }
        }
        cc.egui_ctx.set_fonts(fonts);

        let mut style = (*cc.egui_ctx.style()).clone();
        style.spacing.item_spacing = Vec2::new(10.0, 8.0);
        style.spacing.button_padding = Vec2::new(12.0, 7.0);
        style.spacing.scroll = egui::style::ScrollStyle {
            bar_width: 6.0,
            handle_min_length: 24.0,
            ..Default::default()
        };
        style
            .text_styles
            .insert(egui::TextStyle::Body, FontId::proportional(14.5));
        style
            .text_styles
            .insert(egui::TextStyle::Button, FontId::proportional(14.0));
        style
            .text_styles
            .insert(egui::TextStyle::Heading, FontId::proportional(26.0));
        style
            .text_styles
            .insert(egui::TextStyle::Small, FontId::proportional(12.5));
        cc.egui_ctx.set_style(style);

        let path = storage();
        if let Some(parent) = path.parent() {
            let _ = fs::create_dir_all(parent);
        }

        let mut message = String::new();
        let mut load_failed = false;
        let d = match fs::read(&path) {
            Ok(b) => match serde_json::from_slice::<Data>(&b) {
                Ok(d) => d,
                Err(e) => {
                    load_failed = true;
                    message = format!(
                        "Не удалось прочитать данные: {e}. Исходный файл сохранён; запись отключена."
                    );
                    Data::default()
                }
            },
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Data::default(),
            Err(e) => {
                load_failed = true;
                message = e.to_string();
                Data::default()
            }
        };

        let menu = Menu::new();
        let open = MenuItem::new("Открыть планировщик", true, None);
        let widget = MenuItem::new("Показать / скрыть виджет", true, None);
        let quit = MenuItem::new("Выйти", true, None);
        let _ = menu.append_items(&[&open, &widget, &quit]);

        let rgba: Vec<u8> = (0..32 * 32)
            .flat_map(|i| {
                let x = (i % 32) as f32 - 15.5;
                let y = (i / 32) as f32 - 15.5;
                let dist = (x * x + y * y).sqrt();
                if dist > 14.5 {
                    [0, 0, 0, 0]
                } else if x.abs() < 6.5 && y.abs() < 6.5 {
                    [255, 255, 255, 255]
                } else {
                    [0, 104, 217, 255]
                }
            })
            .collect();

        let tray = tray_icon::Icon::from_rgba(rgba, 32, 32)
            .ok()
            .and_then(|icon| {
                TrayIconBuilder::new()
                    .with_menu(Box::new(menu))
                    .with_tooltip("Мой день — расписание и сон")
                    .with_icon(icon)
                    .build()
                    .ok()
            });

        let last = serde_json::to_string(&d).unwrap();
        let today_d = today();
        Self {
            d,
            section: 0,
            event: Event {
                day: today_d.weekday().num_days_from_monday() as usize,
                ..Default::default()
            },
            task: Task {
                due: today_d.to_string(),
                minutes: 30,
                ..Default::default()
            },
            grade: Grade::default(),
            night: Night::default(),
            quick_task_title: String::new(),
            quick_task_minutes: 25,
            lesson_day: today_d.weekday().num_days_from_monday() as usize,
            custom_subject_input: String::new(),
            lesson_wizard_dismissed: false,
            edit_event: None,
            edit_task: None,
            edit_grade: None,
            edit_night: None,
            show_event_form: false,
            show_completed_tasks: false,
            override_date_input: today_d.to_string(),
            message,
            message_is_error: load_failed,
            path,
            last,
            load_failed,
            undo: None,
            tray,
            open_id: open.id().0.clone(),
            widget_id: widget.id().0.clone(),
            quit_id: quit.id().0.clone(),
            quit: false,
            selected_day: today_d.weekday().num_days_from_monday() as usize,
            motion: Motion::default(),
            dwm: DwmTracker::default(),
            system_reduce_motion: system_reduce_motion(),
            inspector: Inspector::from_env(),
        }
    }

    pub fn notify(&mut self, msg: impl Into<String>) {
        self.message = msg.into();
        self.message_is_error = false;
        self.motion.reset(egui::Id::new("hud_toast_reveal"), 0.0);
    }

    pub fn notify_error(&mut self, msg: impl Into<String>) {
        self.message = msg.into();
        self.message_is_error = true;
        self.motion.reset(egui::Id::new("hud_toast_reveal"), 0.0);
    }

    pub fn push_undo(&mut self) {
        self.undo = Some(self.d.clone());
    }

    pub fn apply_undo(&mut self) {
        if let Some(prev) = self.undo.take() {
            self.d = prev;
            self.edit_event = None;
            self.edit_task = None;
            self.edit_grade = None;
            self.edit_night = None;
            self.notify("Изменение отменено");
        }
    }

    pub fn load_demo(&mut self) {
        self.push_undo();
        self.d = demo_data(today(), &self.d);
        self.notify("Загружены демонстрационные данные");
    }

    pub fn clear_demo(&mut self) {
        self.push_undo();
        self.d.events.clear();
        self.d.tasks.clear();
        self.d.grades.clear();
        self.d.nights.clear();
        for day in &mut self.d.lesson_plan.days {
            day.clear();
        }
        self.d.lesson_plan.confirmed_week.clear();
        self.d.has_demo_data = false;
        self.notify("Демонстрационные данные очищены");
    }

    fn handle_shortcuts(&mut self, ctx: &egui::Context) {
        let wants_kb = ctx.wants_keyboard_input();
        ctx.input(|i| {
            if i.modifiers.command || i.modifiers.ctrl || i.modifiers.mac_cmd {
                if i.key_pressed(egui::Key::Num1) {
                    self.section = SEC_TODAY;
                } else if i.key_pressed(egui::Key::Num2) {
                    self.section = SEC_LESSONS;
                } else if i.key_pressed(egui::Key::Num3) {
                    self.section = SEC_SCHEDULE;
                } else if i.key_pressed(egui::Key::Num4) {
                    self.section = SEC_TASKS;
                } else if i.key_pressed(egui::Key::Num5) {
                    self.section = SEC_GRADES;
                } else if i.key_pressed(egui::Key::Num6) {
                    self.section = SEC_SLEEP;
                } else if i.key_pressed(egui::Key::Num7) {
                    self.section = SEC_SETTINGS;
                } else if i.key_pressed(egui::Key::D) {
                    self.d.dark = !self.d.dark;
                } else if i.key_pressed(egui::Key::Z) && !wants_kb {
                    self.apply_undo();
                }
            }
        });
    }

    fn draw_ambient_backdrop(&self, ctx: &egui::Context, theme: &Theme) {
        let screen = ctx.content_rect();
        let bg_painter = ctx.layer_painter(egui::LayerId::background());
        bg_painter.rect_filled(screen, CornerRadius::ZERO, theme.canvas_bg);

        if theme.glass {
            // Subtle atmospheric macOS-style wallpaper tint bleeding into Liquid Glass surfaces
            let orb_a = Rect::from_min_size(
                screen.min - Vec2::new(60.0, 80.0),
                Vec2::new(screen.width() * 0.55, screen.height() * 0.55),
            );
            let orb_b = Rect::from_min_size(
                Pos2::new(screen.max.x - screen.width() * 0.45, screen.max.y - screen.height() * 0.45),
                Vec2::new(screen.width() * 0.52, screen.height() * 0.52),
            );
            let tint_a = if theme.dark {
                Color32::from_rgba_unmultiplied(46, 98, 178, 22)
            } else {
                Color32::from_rgba_unmultiplied(120, 175, 245, 28)
            };
            let tint_b = if theme.dark {
                Color32::from_rgba_unmultiplied(110, 88, 168, 16)
            } else {
                Color32::from_rgba_unmultiplied(185, 206, 238, 22)
            };
            bg_painter.add(
                egui::epaint::RectShape::filled(orb_a, CornerRadius::same(255), tint_a)
                    .with_blur_width(110.0),
            );
            bg_painter.add(
                egui::epaint::RectShape::filled(orb_b, CornerRadius::same(255), tint_b)
                    .with_blur_width(120.0),
            );
        }
    }

    fn draw_sidebar(&mut self, ui: &mut egui::Ui, theme: &Theme, full_rect: Rect) {
        let sidebar_rect = full_rect.shrink2(Vec2::new(10.0, 10.0));
        draw_glass_surface(
            ui.painter(),
            sidebar_rect,
            GlassSurfaceSpec {
                radius: 16,
                fill: theme.sidebar_bg,
                elevated: true,
                hover_t: 0.0,
            },
            theme,
        );

        let inner = sidebar_rect.shrink2(Vec2::new(10.0, 14.0));
        ui.scope_builder(egui::UiBuilder::new().max_rect(inner), |ui| {
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                ui.add_space(6.0);
                ui.label(semibold("Мой день", 19.0, theme.ink));
                if self.d.has_demo_data {
                    let badge = ui.allocate_response(Vec2::new(48.0, 20.0), Sense::hover());
                    ui.painter().rect_filled(badge.rect, CornerRadius::same(6), theme.accent_soft);
                    let g = ui.painter().layout_no_wrap(
                        "Демо".into(),
                        FontId::proportional(11.0),
                        theme.accent,
                    );
                    ui.painter().galley(
                        badge.rect.center() - g.size() * 0.5,
                        g,
                        theme.accent,
                    );
                }
            });
            ui.add_space(16.0);

            // Reserve space for top 6 sections + bottom-pinned Settings
            let row_h = 38.0;
            let row_gap = 4.0;
            let top_count = SECTIONS.len() - 1;
            let list_top = ui.cursor().min.y;
            let active_idx = self.section.min(SECTIONS.len() - 1);

            let settings_y = inner.max.y - row_h - 32.0;
            let target_y = if active_idx < top_count {
                list_top + active_idx as f32 * (row_h + row_gap)
            } else {
                settings_y
            };

            let anim_y = self
                .motion
                .spring(egui::Id::new("sidebar_pill_y"), target_y, SpringParams::SNAPPY);
            let pill_rect = Rect::from_min_size(
                Pos2::new(inner.min.x, anim_y),
                Vec2::new(inner.width(), row_h),
            );
            draw_liquid_pill(ui.painter(), pill_rect, 10, theme);

            // Draw top 6 navigation items
            for (idx, (label, icon)) in SECTIONS[..top_count].iter().enumerate() {
                let y = list_top + idx as f32 * (row_h + row_gap);
                let item_rect = Rect::from_min_size(
                    Pos2::new(inner.min.x, y),
                    Vec2::new(inner.width(), row_h),
                );
                let resp = ui.interact(item_rect, egui::Id::new(("nav_item", idx)), Sense::click());
                if resp.clicked() && self.section != idx {
                    self.section = idx;
                    self.message.clear();
                }
                let hover_t = self.motion.spring(
                    egui::Id::new(("nav_hover", idx)),
                    if resp.hovered() && self.section != idx {
                        1.0
                    } else {
                        0.0
                    },
                    SpringParams::MICRO,
                );
                if hover_t > 0.02 {
                    ui.painter().rect_filled(
                        item_rect,
                        CornerRadius::same(10),
                        theme.pill_track.gamma_multiply(hover_t),
                    );
                }
                let selected = self.section == idx;
                let icon_color = if selected {
                    theme.accent
                } else {
                    lerp_color(theme.ink_secondary, theme.ink, hover_t)
                };
                let text_color = if selected {
                    theme.ink
                } else {
                    lerp_color(theme.ink_secondary, theme.ink, hover_t)
                };
                draw_icon(
                    ui.painter(),
                    Pos2::new(item_rect.min.x + 20.0, item_rect.center().y),
                    16.0,
                    *icon,
                    icon_color,
                    if selected { 1.75 } else { 1.5 },
                );
                let font = if selected {
                    FontId::new(14.5, egui::FontFamily::Name("Semibold".into()))
                } else {
                    FontId::proportional(14.5)
                };
                let galley = ui.painter().layout_no_wrap((*label).to_owned(), font, text_color);
                ui.painter().galley(
                    Pos2::new(item_rect.min.x + 38.0, item_rect.center().y - galley.size().y * 0.5),
                    galley,
                    text_color,
                );
            }

            // Bottom-pinned Settings item
            let (s_label, s_icon) = SECTIONS[SEC_SETTINGS];
            let s_rect = Rect::from_min_size(
                Pos2::new(inner.min.x, settings_y),
                Vec2::new(inner.width(), row_h),
            );
            let s_resp = ui.interact(s_rect, egui::Id::new(("nav_item", SEC_SETTINGS)), Sense::click());
            if s_resp.clicked() && self.section != SEC_SETTINGS {
                self.section = SEC_SETTINGS;
                self.message.clear();
            }
            let s_hover = self.motion.spring(
                egui::Id::new(("nav_hover", SEC_SETTINGS)),
                if s_resp.hovered() && self.section != SEC_SETTINGS {
                    1.0
                } else {
                    0.0
                },
                SpringParams::MICRO,
            );
            if s_hover > 0.02 {
                ui.painter().rect_filled(
                    s_rect,
                    CornerRadius::same(10),
                    theme.pill_track.gamma_multiply(s_hover),
                );
            }
            let s_sel = self.section == SEC_SETTINGS;
            draw_icon(
                ui.painter(),
                Pos2::new(s_rect.min.x + 20.0, s_rect.center().y),
                16.0,
                s_icon,
                if s_sel { theme.accent } else { theme.ink_secondary },
                if s_sel { 1.75 } else { 1.5 },
            );
            let s_font = if s_sel {
                FontId::new(14.5, egui::FontFamily::Name("Semibold".into()))
            } else {
                FontId::proportional(14.5)
            };
            let s_col = if s_sel { theme.ink } else { theme.ink_secondary };
            let s_galley = ui.painter().layout_no_wrap(s_label.to_owned(), s_font, s_col);
            ui.painter().galley(
                Pos2::new(s_rect.min.x + 38.0, s_rect.center().y - s_galley.size().y * 0.5),
                s_galley,
                s_col,
            );

            // Quiet local storage footer
            let foot_galley = ui.painter().layout_no_wrap(
                "Локально · На этом ПК".into(),
                FontId::proportional(11.5),
                theme.ink_muted,
            );
            ui.painter().galley(
                Pos2::new(inner.min.x + 10.0, inner.max.y - 16.0),
                foot_galley,
                theme.ink_muted,
            );
        });
    }

    fn draw_compact_nav(&mut self, ui: &mut egui::Ui, theme: &Theme) {
        let nav_names: Vec<&str> = SECTIONS.iter().map(|(s, _)| *s).collect();
        let mut sel = self.section;
        if glass::segmented_control(
            ui,
            &mut self.motion,
            egui::Id::new("compact_top_nav"),
            &mut sel,
            &nav_names,
            theme,
        ) {
            self.section = sel;
            self.message.clear();
        }
    }

    fn draw_toolbar(&mut self, ui: &mut egui::Ui, theme: &Theme) {
        let today_d = today();
        ui.horizontal(|ui| {
            ui.label(RichText::new(ru_date_long(today_d)).size(13.0).color(theme.ink_secondary));
            if self.d.has_demo_data {
                ui.label(RichText::new("· Демонстрационные данные").size(12.5).color(theme.accent));
            }
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let settings_tip = if cfg!(target_os = "macos") {
                    "Настройки (⌘7)"
                } else {
                    "Настройки (Ctrl+7)"
                };
                if icon_button(
                    ui,
                    &mut self.motion,
                    egui::Id::new("tb_settings"),
                    Icon::Sliders,
                    self.section == SEC_SETTINGS,
                    settings_tip,
                    theme,
                )
                .clicked()
                {
                    self.section = SEC_SETTINGS;
                }
                if icon_button(
                    ui,
                    &mut self.motion,
                    egui::Id::new("tb_widget"),
                    Icon::WindowOpen,
                    self.d.widget,
                    "Показать / скрыть виджет рабочего стола",
                    theme,
                )
                .clicked()
                {
                    self.d.widget = !self.d.widget;
                }
                let theme_tip = if cfg!(target_os = "macos") {
                    "Переключить светлую и тёмную тему (⌘D)"
                } else {
                    "Переключить светлую и тёмную тему (Ctrl+D)"
                };
                if icon_button(
                    ui,
                    &mut self.motion,
                    egui::Id::new("tb_theme"),
                    if self.d.dark { Icon::Sun } else { Icon::Moon },
                    false,
                    theme_tip,
                    theme,
                )
                .clicked()
                {
                    self.d.dark = !self.d.dark;
                }
            });
        });
    }

    fn draw_hud_toast(&mut self, ctx: &egui::Context, theme: &Theme) {
        let show = !self.message.is_empty() || self.undo.is_some();
        if !show {
            return;
        }
        let t = self.motion.spring_from(
            egui::Id::new("hud_toast_reveal"),
            0.0,
            1.0,
            SpringParams::TACTILE,
        );
        let y_shift = if self.motion.is_reduced() {
            0.0
        } else {
            (1.0 - t) * 14.0
        };

        let offset_x = if ctx.content_rect().width() >= 760.0 {
            106.0
        } else {
            0.0
        };

        let toast_text = if self.message.is_empty() {
            "Можно отменить последнее изменение".to_owned()
        } else {
            self.message.clone()
        };
        let has_message = !self.message.is_empty();
        let has_undo = self.undo.is_some();
        let is_err = self.message_is_error;
        let allow_hide = !self.load_failed && has_message;

        let mut do_undo = false;
        let mut do_hide = false;

        egui::Area::new(egui::Id::new("hud_toast_area"))
            .order(egui::Order::Foreground)
            .anchor(egui::Align2::CENTER_BOTTOM, Vec2::new(offset_x, -16.0 + y_shift))
            .show(ctx, |ui| {
                let slot = ui.painter().add(egui::Shape::Noop);
                let inner = egui::Frame::new()
                    .inner_margin(egui::Margin::symmetric(16, 9))
                    .show(ui, |ui| {
                        let text_col = if is_err { theme.danger } else { theme.ink };
                        ui.horizontal(|ui| {
                            ui.spacing_mut().item_spacing.x = 10.0;
                            ui.label(RichText::new(&toast_text).size(13.0).color(text_col));
                            if has_undo
                                && glass::glass_button(
                                    ui,
                                    &mut self.motion,
                                    egui::Id::new("hud_undo_btn"),
                                    "Отменить",
                                    Some(Icon::Undo),
                                    false,
                                    theme,
                                )
                                .clicked()
                            {
                                do_undo = true;
                            }
                            if allow_hide
                                && ui
                                    .add(
                                        egui::Button::new(
                                            RichText::new("Скрыть")
                                                .size(12.5)
                                                .color(theme.ink_secondary),
                                        )
                                        .frame(false),
                                    )
                                    .clicked()
                            {
                                do_hide = true;
                            }
                        });
                    });

                let shapes = glass::glass_shapes(
                    inner.response.rect,
                    GlassSurfaceSpec {
                        radius: 14,
                        fill: theme.surface_elevated,
                        elevated: true,
                        hover_t: 0.0,
                    },
                    theme,
                );
                ui.painter().set(slot, egui::Shape::Vec(shapes));
            });

        if do_undo {
            self.apply_undo();
        } else if do_hide {
            self.message.clear();
            self.message_is_error = false;
        }
    }
}

impl eframe::App for PlannerApp {
    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        if self.inspector.as_ref().is_some_and(|i| i.is_widget_step()) {
            return [0.0, 0.0, 0.0, 0.0];
        }
        if self.d.glass_effect && !self.d.reduce_transparency {
            [0.0, 0.0, 0.0, 0.0]
        } else if self.d.dark {
            egui::Color32::from_rgb(24, 25, 29).to_normalized_gamma_f32()
        } else {
            egui::Color32::from_rgb(241, 242, 244).to_normalized_gamma_f32()
        }
    }

    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        ctx.request_repaint_after(std::time::Duration::from_secs(1));

        while let Ok(e) = MenuEvent::receiver().try_recv() {
            if e.id.0 == self.open_id {
                ctx.send_viewport_cmd(egui::ViewportCommand::Visible(true));
                ctx.send_viewport_cmd(egui::ViewportCommand::Focus);
            } else if e.id.0 == self.widget_id {
                self.d.widget = !self.d.widget;
            } else if e.id.0 == self.quit_id {
                self.quit = true;
            }
        }

        if ctx.input(|i| i.viewport().close_requested()) && !self.quit && self.tray.is_some() {
            ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
            ctx.send_viewport_cmd(egui::ViewportCommand::Visible(false));
        }

        if self.quit {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            return;
        }

        self.handle_shortcuts(ctx);

        let motion_moving = self.motion.is_moving();
        if let Some(insp) = self.inspector.as_mut() {
            insp.tick_main(
                ctx,
                &mut self.section,
                &mut self.d.dark,
                &mut self.d.widget,
                &mut self.d.schedule_grid,
                &mut self.d.sleep_tab,
                &mut self.show_completed_tasks,
                motion_moving,
            );
            if insp.finished {
                self.quit = true;
            }
        }

        let effective_reduce_motion = self.d.reduce_motion || self.system_reduce_motion;
        self.motion.begin_frame(ctx, effective_reduce_motion);

        let theme = Theme::from_data(&self.d);
        apply_egui_visuals(ctx, &theme);
        self.dwm.sync_main(frame, theme.dark, theme.glass);

        let inspect_widget_only = self
            .inspector
            .as_ref()
            .is_some_and(|i| i.is_widget_step());

        if inspect_widget_only {
            let mut open_main = false;
            let mut toggle_top = false;
            let mut visible = true;
            widget::paint_widget_surface(
                self,
                ctx,
                &theme,
                &mut open_main,
                &mut toggle_top,
                &mut visible,
            );
            self.motion.end_frame(ctx);
            return;
        }

        self.draw_ambient_backdrop(ctx, &theme);

        let screen = ctx.content_rect();
        let wide = screen.width() >= 760.0;

        if wide {
            egui::SidePanel::left("glass_sidebar")
                .exact_width(226.0)
                .resizable(false)
                .frame(egui::Frame::NONE)
                .show(ctx, |ui| {
                    let rect = ui.max_rect();
                    self.draw_sidebar(ui, &theme, rect);
                });
        }

        let (fade_alpha, slide_y) = self.motion.section_transition(self.section);

        egui::CentralPanel::default()
            .frame(egui::Frame::NONE.inner_margin(egui::Margin {
                left: if wide { 14 } else { 16 },
                right: 22,
                top: 12,
                bottom: 18,
            }))
            .show(ctx, |ui| {
                // Content card container (crisp readable surface contrasting with translucent sidebar)
                let panel_rect = ui.available_rect_before_wrap();
                draw_glass_surface(
                    ui.painter(),
                    panel_rect,
                    GlassSurfaceSpec {
                        radius: 16,
                        fill: theme.content_bg,
                        elevated: false,
                        hover_t: 0.0,
                    },
                    &theme,
                );

                let inner_rect = panel_rect.shrink2(Vec2::new(24.0, 18.0));
                ui.scope_builder(egui::UiBuilder::new().max_rect(inner_rect), |ui| {
                    if !wide {
                        self.draw_compact_nav(ui, &theme);
                        ui.add_space(10.0);
                    }
                    self.draw_toolbar(ui, &theme);
                    ui.add_space(6.0);
                    let line_y = ui.cursor().min.y;
                    ui.painter().line_segment(
                        [
                            Pos2::new(inner_rect.min.x, line_y),
                            Pos2::new(inner_rect.max.x, line_y),
                        ],
                        Stroke::new(1.0_f32, theme.divider),
                    );
                    ui.add_space(14.0);

                    ui.set_opacity(fade_alpha);
                    if slide_y.abs() > 0.1 {
                        ui.add_space(slide_y);
                    }

                    let scroll_w = ui.available_width();
                    egui::ScrollArea::vertical()
                        .id_salt(("main_scroll", self.section))
                        .auto_shrink([false, false])
                        .show(ui, |ui| {
                            ui.set_max_width(scroll_w);
                            match self.section {
                                SEC_TODAY => today::show(self, ui, &theme),
                                SEC_LESSONS => lessons::show(self, ui, &theme),
                                SEC_SCHEDULE => schedule::show(self, ui, &theme),
                                SEC_TASKS => tasks::show(self, ui, &theme),
                                SEC_GRADES => grades::show(self, ui, &theme),
                                SEC_SLEEP => sleep::show(self, ui, &theme),
                                _ => settings::show(self, ui, &theme),
                            }
                        });
                });
            });

        self.draw_hud_toast(ctx, &theme);

        if self.d.widget && self.inspector.is_none() {
            widget::show(self, ctx, &theme);
        }

        self.motion.end_frame(ctx);

        let serialized = serde_json::to_string(&self.d).unwrap();
        if serialized != self.last && !self.load_failed && self.inspector.is_none() {
            match save_data(&self.d, &self.path) {
                Ok(()) => self.last = serialized,
                Err(e) => self.notify_error(format!("Ошибка сохранения: {e}")),
            }
        }
    }
}

pub fn labeled_input(ui: &mut egui::Ui, label: &str, value: &mut String, width: f32, theme: &Theme) {
    ui.allocate_ui_with_layout(
        Vec2::new(width, 0.0),
        egui::Layout::top_down(egui::Align::Min),
        |ui| {
            ui.spacing_mut().item_spacing.y = 4.0;
            ui.add(
                egui::Label::new(RichText::new(label).size(12.5).color(theme.ink_secondary))
                    .wrap(),
            );
            ui.add_sized(
                [width, 32.0],
                egui::TextEdit::singleline(value).margin(egui::Margin::symmetric(10, 6)),
            );
        },
    );
}

pub fn form_columns(avail_w: f32, max_cols: usize, min_col_w: f32, gap_x: f32) -> (usize, f32) {
    let cols = ((avail_w + gap_x) / (min_col_w + gap_x))
        .floor()
        .clamp(1.0, max_cols.max(1) as f32) as usize;
    let col_w = ((avail_w - gap_x * cols.saturating_sub(1) as f32) / cols as f32).max(90.0);
    (cols, col_w)
}
