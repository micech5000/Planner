use crate::glass::{
    Icon, Theme, draw_icon, glass_button, glass_panel, reminders_checkbox, segmented_control,
    semibold,
};
use crate::model::{
    DAYS, DAYS_SHORT, LessonSlot, iso_week_key, time, today, week_range_label,
};
use crate::ui::PlannerApp;
use eframe::egui::{self, Color32, CornerRadius, Pos2, RichText, Sense, Stroke, Vec2};

pub fn show(app: &mut PlannerApp, ui: &mut egui::Ui, theme: &Theme) {
    let today_d = today();
    let day = app.lesson_day.min(6);
    app.lesson_day = day;
    let needs_check = app.d.needs_weekly_lesson_check(today_d);
    let week_range = week_range_label(today_d);

    // Header
    let hdr_avail = ui.available_width();
    let badge_est_w = if needs_check { 205.0 } else { 282.0 };
    if hdr_avail >= 560.0 {
        let left_w = (hdr_avail - badge_est_w - 14.0).max(220.0);
        ui.horizontal_top(|ui| {
            let left_resp = ui.allocate_ui_with_layout(
                Vec2::new(left_w, 0.0),
                egui::Layout::top_down(egui::Align::Min),
                |ui| {
                    ui.label(semibold("Уроки", 28.0, theme.ink));
                    ui.add(
                        egui::Label::new(
                            RichText::new(
                                "Выберите предметы галочками по дням, расставьте по очереди и задайте начало и время возвращения домой",
                            )
                            .size(13.0)
                            .color(theme.ink_secondary),
                        )
                        .wrap(),
                    );
                },
            );
            let row_h = left_resp.response.rect.height().max(34.0);
            let rem_w = ui.available_width().max(badge_est_w);
            ui.allocate_ui_with_layout(
                Vec2::new(rem_w, row_h),
                egui::Layout::right_to_left(egui::Align::Center),
                |ui| {
                    draw_week_status_control(app, ui, theme, needs_check, &week_range, today_d);
                },
            );
        });
    } else {
        ui.label(semibold("Уроки", 28.0, theme.ink));
        ui.add(
            egui::Label::new(
                RichText::new(
                    "Выберите предметы галочками по дням, расставьте по очереди и задайте начало и время возвращения домой",
                )
                .size(13.0)
                .color(theme.ink_secondary),
            )
            .wrap(),
        );
        ui.add_space(6.0);
        draw_week_status_control(app, ui, theme, needs_check, &week_range, today_d);
    }

    ui.add_space(10.0);

    // Weekly check-in banner at the start of a new week
    if needs_check {
        let w = ui.available_width();
        let inner_w = (w - 20.0).max(200.0);
        let has_prev = app.d.lesson_plan.has_any_lessons();
        glass_panel(ui, theme, 14, false, 10, |ui| {
            ui.set_min_width(inner_w);
            let show_keep_inline = has_prev && inner_w >= 560.0;
            let keep_w = if show_keep_inline { 196.0 } else { 0.0 };
            let text_w = (inner_w - 36.0 - keep_w).max(160.0);

            ui.horizontal_top(|ui| {
                let (ic_r, _) = ui.allocate_exact_size(Vec2::splat(26.0), Sense::hover());
                ui.painter()
                    .circle_filled(ic_r.center(), 13.0, theme.accent_soft);
                draw_icon(
                    ui.painter(),
                    ic_r.center(),
                    14.0,
                    Icon::Lessons,
                    theme.accent,
                    1.6,
                );

                let text_resp = ui.allocate_ui_with_layout(
                    Vec2::new(text_w, 0.0),
                    egui::Layout::top_down(egui::Align::Min),
                    |ui| {
                        ui.spacing_mut().item_spacing.y = 2.0;
                        ui.label(semibold(
                            format!("Начало новой недели ({week_range})"),
                            14.5,
                            theme.ink,
                        ));
                        ui.add(
                            egui::Label::new(
                                RichText::new(
                                    "Отметьте галочками, какие уроки идут по очереди, укажите начало первого урока и во сколько домой.",
                                )
                                .size(12.5)
                                .color(theme.ink_secondary),
                            )
                            .wrap(),
                        );
                    },
                );

                if show_keep_inline {
                    let row_h = text_resp.response.rect.height().max(34.0);
                    let rem_w = ui.available_width().max(keep_w);
                    ui.allocate_ui_with_layout(
                        Vec2::new(rem_w, row_h),
                        egui::Layout::right_to_left(egui::Align::Center),
                        |ui| {
                            if glass_button(
                                ui,
                                &mut app.motion,
                                egui::Id::new("lessons_keep_prev_week"),
                                "Оставить как раньше",
                                None,
                                false,
                                theme,
                            )
                            .clicked()
                            {
                                confirm_current_week(app, today_d);
                            }
                        },
                    );
                }
            });

            if has_prev && !show_keep_inline {
                ui.add_space(8.0);
                ui.horizontal(|ui| {
                    ui.add_space(34.0);
                    if glass_button(
                        ui,
                        &mut app.motion,
                        egui::Id::new("lessons_keep_prev_week_narrow"),
                        "Оставить как раньше",
                        None,
                        false,
                        theme,
                    )
                    .clicked()
                    {
                        confirm_current_week(app, today_d);
                    }
                });
            }
        });
        ui.add_space(8.0);
    }

    // 1. School day times: when lessons start and when going home
    let w = ui.available_width();
    let panel_inner_w = (w - 24.0).max(200.0);
    let mut times_changed = false;
    glass_panel(ui, theme, 16, false, 12, |ui| {
        ui.set_min_width(panel_inner_w);
        ui.set_max_width(panel_inner_w);
        ui.label(semibold("Время уроков и во сколько домой", 15.0, theme.ink));
        ui.add_space(6.0);

        let inner_w = ui.available_width();
        ui.spacing_mut().interact_size.y = 26.0;

        let draw_start_group = |app: &mut PlannerApp, ui: &mut egui::Ui| -> bool {
            let mut changed = false;
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 6.0;
                ui.label(
                    RichText::new("Начало уроков")
                        .size(12.5)
                        .color(theme.ink_secondary),
                );
                let te = ui.add_sized(
                    [58.0, 26.0],
                    egui::TextEdit::singleline(&mut app.d.lesson_plan.start_time)
                        .margin(egui::Margin::symmetric(8, 4)),
                );
                if te.changed() && time(&app.d.lesson_plan.start_time).is_some() {
                    changed = true;
                }
                for preset in ["08:00", "08:30", "09:00"] {
                    let active = app.d.lesson_plan.start_time.trim() == preset;
                    if preset_chip(
                        ui,
                        egui::Id::new(("start_pre", preset)),
                        preset,
                        active,
                        theme,
                    ) {
                        app.push_undo();
                        app.d.lesson_plan.start_time = preset.to_owned();
                        changed = true;
                    }
                }
            });
            changed
        };

        let draw_home_group = |app: &mut PlannerApp, ui: &mut egui::Ui| -> bool {
            let mut changed = false;
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 6.0;
                ui.label(
                    RichText::new("Во сколько домой")
                        .size(12.5)
                        .color(theme.ink_secondary),
                );
                let te = ui.add_sized(
                    [58.0, 26.0],
                    egui::TextEdit::singleline(&mut app.d.lesson_plan.home_time)
                        .margin(egui::Margin::symmetric(8, 4)),
                );
                if te.changed() && time(&app.d.lesson_plan.home_time).is_some() {
                    changed = true;
                }
                for preset in ["13:30", "14:15", "15:00"] {
                    let active = app.d.lesson_plan.home_time.trim() == preset;
                    if preset_chip(
                        ui,
                        egui::Id::new(("home_pre", preset)),
                        preset,
                        active,
                        theme,
                    ) {
                        app.push_undo();
                        app.d.lesson_plan.home_time = preset.to_owned();
                        changed = true;
                    }
                }
            });
            changed
        };

        if inner_w >= 675.0 {
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 18.0;
                if draw_start_group(app, ui) {
                    times_changed = true;
                }
                if draw_home_group(app, ui) {
                    times_changed = true;
                }
            });
        } else {
            if draw_start_group(app, ui) {
                times_changed = true;
            }
            ui.add_space(6.0);
            if draw_home_group(app, ui) {
                times_changed = true;
            }
        }

        ui.add_space(7.0);

        // Bell lengths & morning travel
        if inner_w >= 415.0 {
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 6.0;
                ui.label(RichText::new("Урок").size(12.5).color(theme.ink_secondary));
                if ui
                    .add_sized(
                        [56.0, 26.0],
                        egui::DragValue::new(&mut app.d.lesson_plan.lesson_mins)
                            .range(15..=90)
                            .suffix(" м"),
                    )
                    .changed()
                {
                    times_changed = true;
                }

                ui.add_space(8.0);
                ui.label(RichText::new("Перемена").size(12.5).color(theme.ink_secondary));
                if ui
                    .add_sized(
                        [56.0, 26.0],
                        egui::DragValue::new(&mut app.d.lesson_plan.break_mins)
                            .range(0..=45)
                            .suffix(" м"),
                    )
                    .changed()
                {
                    times_changed = true;
                }

                ui.add_space(8.0);
                ui.label(RichText::new("Дорога утром").size(12.5).color(theme.ink_secondary));
                if ui
                    .add_sized(
                        [58.0, 26.0],
                        egui::DragValue::new(&mut app.d.lesson_plan.travel_mins)
                            .range(0..=120)
                            .suffix(" м"),
                    )
                    .changed()
                {
                    times_changed = true;
                }
            });
        } else {
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 6.0;
                ui.label(RichText::new("Урок").size(12.5).color(theme.ink_secondary));
                if ui
                    .add_sized(
                        [56.0, 26.0],
                        egui::DragValue::new(&mut app.d.lesson_plan.lesson_mins)
                            .range(15..=90)
                            .suffix(" м"),
                    )
                    .changed()
                {
                    times_changed = true;
                }
                ui.add_space(8.0);
                ui.label(RichText::new("Перемена").size(12.5).color(theme.ink_secondary));
                if ui
                    .add_sized(
                        [56.0, 26.0],
                        egui::DragValue::new(&mut app.d.lesson_plan.break_mins)
                            .range(0..=45)
                            .suffix(" м"),
                    )
                    .changed()
                {
                    times_changed = true;
                }
            });
            ui.add_space(6.0);
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 6.0;
                ui.label(RichText::new("Дорога утром").size(12.5).color(theme.ink_secondary));
                if ui
                    .add_sized(
                        [58.0, 26.0],
                        egui::DragValue::new(&mut app.d.lesson_plan.travel_mins)
                            .range(0..=120)
                            .suffix(" м"),
                    )
                    .changed()
                {
                    times_changed = true;
                }
            });
        }

        ui.add_space(7.0);
        let valid_start = time(&app.d.lesson_plan.start_time).is_some();
        let valid_home = time(&app.d.lesson_plan.home_time).is_some();
        if !valid_start || !valid_home {
            ui.label(
                RichText::new("Укажите время в формате ЧЧ:ММ (например 08:30 и 14:15)")
                    .size(12.0)
                    .color(theme.danger),
            );
        } else {
            let day_lessons = app.d.lesson_plan.days[day].len();
            let preview_slots = if day_lessons > 0 { day_lessons } else { 4 };
            let mut bell_parts = Vec::with_capacity(preview_slots);
            for i in 0..preview_slots {
                if let Some((st, en)) = app.d.lesson_plan.slot_times(i) {
                    bell_parts.push(format!("{}) {}–{}", i + 1, st.format("%H:%M"), en.format("%H:%M")));
                }
            }
            let bell_prefix = if day_lessons > 0 {
                format!("Звонки ({day_lessons} ур.):")
            } else {
                "Сетка звонков:".to_owned()
            };
            ui.add(
                egui::Label::new(
                    RichText::new(format!(
                        "{bell_prefix} {} · Домой: {}",
                        bell_parts.join("  ·  "),
                        app.d.lesson_plan.home_time.trim()
                    ))
                    .size(12.0)
                    .color(theme.ink_muted),
                )
                .wrap(),
            );
        }
    });

    if times_changed {
        app.d.sync_lessons_to_events();
    }

    ui.add_space(8.0);

    // 2. Day tabs (with lesson counts) + quick navigation across the week
    let tab_strings: Vec<String> = (0..7)
        .map(|i| {
            let cnt = app.d.lesson_plan.days[i].len();
            if cnt > 0 {
                format!("{} ({cnt})", DAYS_SHORT[i])
            } else {
                DAYS_SHORT[i].to_owned()
            }
        })
        .collect();
    let tab_refs: Vec<&str> = tab_strings.iter().map(String::as_str).collect();
    segmented_control(
        ui,
        &mut app.motion,
        egui::Id::new("lessons_day_tabs"),
        &mut app.lesson_day,
        &tab_refs,
        theme,
    );
    let day = app.lesson_day.min(6);

    ui.add_space(6.0);
    let day_hdr_w = ui.available_width();
    if day_hdr_w >= 600.0 {
        ui.horizontal(|ui| {
            ui.label(semibold(
                format!("{}: уроков {}", DAYS[day], app.d.lesson_plan.days[day].len()),
                15.5,
                theme.ink,
            ));
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                draw_day_actions(app, ui, theme, day);
            });
        });
    } else {
        ui.label(semibold(
            format!("{}: уроков {}", DAYS[day], app.d.lesson_plan.days[day].len()),
            15.5,
            theme.ink,
        ));
        ui.add_space(4.0);
        ui.horizontal_wrapped(|ui| {
            draw_day_actions(app, ui, theme, day);
        });
    }

    ui.add_space(6.0);

    // 3. Two-column checklist (left) + ordered lesson sequence (right)
    let avail_w = ui.available_width();
    if avail_w >= 660.0 {
        let gap = 14.0;
        let left_w = ((avail_w - gap) * 0.55).max(320.0);
        let right_w = (avail_w - gap - left_w).max(240.0);
        ui.horizontal_top(|ui| {
            ui.allocate_ui_with_layout(
                Vec2::new(left_w, 0.0),
                egui::Layout::top_down(egui::Align::Min),
                |ui| show_subject_checklist(app, ui, theme, day),
            );
            ui.add_space(gap);
            ui.allocate_ui_with_layout(
                Vec2::new(right_w, 0.0),
                egui::Layout::top_down(egui::Align::Min),
                |ui| show_day_sequence(app, ui, theme, day),
            );
        });
    } else {
        show_subject_checklist(app, ui, theme, day);
        ui.add_space(12.0);
        show_day_sequence(app, ui, theme, day);
    }
}

fn draw_week_status_control(
    app: &mut PlannerApp,
    ui: &mut egui::Ui,
    theme: &Theme,
    needs_check: bool,
    week_range: &str,
    today_d: chrono::NaiveDate,
) {
    if needs_check {
        if glass_button(
            ui,
            &mut app.motion,
            egui::Id::new("lessons_confirm_week_top"),
            "Подтвердить неделю",
            Some(Icon::Check),
            true,
            theme,
        )
        .clicked()
        {
            confirm_current_week(app, today_d);
        }
    } else {
        let badge_txt = format!("Неделя подтверждена · {week_range}");
        let g = ui.painter().layout_no_wrap(
            badge_txt,
            egui::FontId::new(12.5, egui::FontFamily::Name("Semibold".into())),
            theme.accent,
        );
        let (rect, _) = ui.allocate_exact_size(Vec2::new(g.size().x + 22.0, 30.0), Sense::hover());
        ui.painter()
            .rect_filled(rect, CornerRadius::same(9), theme.accent_soft);
        ui.painter()
            .galley(rect.center() - g.size() * 0.5, g, theme.accent);
    }
}

fn draw_day_actions(app: &mut PlannerApp, ui: &mut egui::Ui, theme: &Theme, day: usize) {
    if day < 6
        && glass_button(
            ui,
            &mut app.motion,
            egui::Id::new("lessons_next_day_btn"),
            &format!("Дальше: {}", DAYS_SHORT[day + 1]),
            Some(Icon::ChevronRight),
            false,
            theme,
        )
        .clicked()
    {
        app.lesson_day = day + 1;
    }
    if day > 0
        && !app.d.lesson_plan.days[day - 1].is_empty()
        && glass_button(
            ui,
            &mut app.motion,
            egui::Id::new("lessons_copy_prev_btn"),
            &format!("Скопировать с {}", DAYS_SHORT[day - 1]),
            None,
            false,
            theme,
        )
        .clicked()
    {
        app.push_undo();
        let src = app.d.lesson_plan.days[day - 1].clone();
        app.d.lesson_plan.days[day] = src;
        app.d.sync_lessons_to_events();
        app.notify(format!("Уроки скопированы в {}", DAYS[day].to_lowercase()));
    }
    if !app.d.lesson_plan.days[day].is_empty()
        && ui
            .add(
                egui::Button::new(
                    RichText::new("Очистить день")
                        .size(12.5)
                        .color(theme.ink_secondary),
                )
                .frame(false),
            )
            .clicked()
    {
        app.push_undo();
        app.d.lesson_plan.days[day].clear();
        app.d.sync_lessons_to_events();
    }
}

fn confirm_current_week(app: &mut PlannerApp, today_d: chrono::NaiveDate) {
    app.push_undo();
    app.d.lesson_plan.confirmed_week = iso_week_key(today_d);
    app.d.sync_lessons_to_events();
    app.lesson_wizard_dismissed = true;
    app.notify("Расписание уроков на неделю подтверждено");
}

fn show_subject_checklist(
    app: &mut PlannerApp,
    ui: &mut egui::Ui,
    theme: &Theme,
    day: usize,
) {
    let w = ui.available_width();
    glass_panel(ui, theme, 16, false, 12, |ui| {
        ui.set_min_width((w - 24.0).max(200.0));
        ui.label(semibold("Все предметы — выберите галочками", 15.0, theme.ink));
        ui.add(
            egui::Label::new(
                RichText::new(
                    "Отмеченный предмет сразу добавляется в очередь дня справа. «+» — второй урок.",
                )
                .size(12.0)
                .color(theme.ink_secondary),
            )
            .wrap(),
        );
        ui.add_space(6.0);

        let catalog = app.d.lesson_plan.subjects_catalog.clone();
        let inner_w = ui.available_width();
        let cols = if inner_w >= 340.0 { 2 } else { 1 };
        let gap_x = 10.0;
        let col_w = ((inner_w - gap_x * (cols as f32 - 1.0)) / cols as f32).max(140.0);

        let mut add_subject: Option<String> = None;
        let mut remove_subject: Option<String> = None;

        for row_items in catalog.chunks(cols) {
            ui.horizontal_top(|ui| {
                ui.spacing_mut().item_spacing.x = gap_x;
                for subj in row_items {
                    ui.allocate_ui_with_layout(
                        Vec2::new(col_w, 24.0),
                        egui::Layout::left_to_right(egui::Align::Center),
                        |ui| {
                            ui.spacing_mut().item_spacing.x = 4.0;
                            let count = app.d.lesson_plan.days[day]
                                .iter()
                                .filter(|s| s.subject == *subj)
                                .count();
                            let mut checked = count > 0;
                            if reminders_checkbox(
                                ui,
                                &mut app.motion,
                                egui::Id::new(("lesson_chk", day, subj.as_str())),
                                &mut checked,
                                theme,
                            ) {
                                if checked {
                                    add_subject = Some(subj.clone());
                                } else {
                                    remove_subject = Some(subj.clone());
                                }
                            }

                            let extra_w = if count > 1 {
                                42.0
                            } else if count == 1 {
                                32.0
                            } else {
                                2.0
                            };
                            let label_w = (col_w - 30.0 - extra_w).max(60.0);
                            let label_col = if count > 0 { theme.ink } else { theme.ink_secondary };
                            let _ = ui.allocate_ui_with_layout(
                                Vec2::new(label_w, 22.0),
                                egui::Layout::left_to_right(egui::Align::Center),
                                |ui| {
                                    let lbl = ui.add(
                                        egui::Label::new(
                                            RichText::new(subj.as_str())
                                                .size(13.0)
                                                .color(label_col),
                                        )
                                        .truncate()
                                        .sense(Sense::click()),
                                    );
                                    if lbl.clicked() {
                                        if count > 0 {
                                            remove_subject = Some(subj.clone());
                                        } else {
                                            add_subject = Some(subj.clone());
                                        }
                                    }
                                },
                            );

                            if count > 0 {
                                let chip_txt = if count > 1 {
                                    format!("{count}×+")
                                } else {
                                    "+".to_owned()
                                };
                                if preset_chip(
                                    ui,
                                    egui::Id::new(("lesson_add_more", day, subj.as_str())),
                                    &chip_txt,
                                    false,
                                    theme,
                                ) {
                                    add_subject = Some(subj.clone());
                                }
                            }
                        },
                    );
                }
            });
            ui.add_space(1.0);
        }

        if let Some(subj) = add_subject {
            app.push_undo();
            app.d.lesson_plan.days[day].push(LessonSlot {
                subject: subj,
                place: String::new(),
            });
            app.d.sync_lessons_to_events();
        } else if let Some(subj) = remove_subject {
            app.push_undo();
            app.d.lesson_plan.days[day].retain(|s| s.subject != subj);
            app.d.sync_lessons_to_events();
        }

        ui.add_space(8.0);
        let line_y = ui.cursor().min.y;
        ui.painter().line_segment(
            [
                Pos2::new(ui.cursor().min.x, line_y),
                Pos2::new(ui.cursor().min.x + ui.available_width(), line_y),
            ],
            Stroke::new(1.0_f32, theme.divider),
        );
        ui.add_space(8.0);

        ui.horizontal(|ui| {
            let input_w = (ui.available_width() - 126.0).max(100.0);
            let te = ui.add_sized(
                [input_w, 28.0],
                egui::TextEdit::singleline(&mut app.custom_subject_input)
                    .hint_text("Свой предмет (если нет в списке)...")
                    .margin(egui::Margin::symmetric(8, 4)),
            );
            let enter = te.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));
            let clicked = glass_button(
                ui,
                &mut app.motion,
                egui::Id::new("lesson_add_custom_btn"),
                "Добавить",
                Some(Icon::Plus),
                false,
                theme,
            )
            .clicked();

            if (enter || clicked) && !app.custom_subject_input.trim().is_empty() {
                let name = app.custom_subject_input.trim().to_owned();
                app.push_undo();
                if !app
                    .d
                    .lesson_plan
                    .subjects_catalog
                    .iter()
                    .any(|s| s.eq_ignore_ascii_case(&name))
                {
                    app.d.lesson_plan.subjects_catalog.push(name.clone());
                }
                app.d.lesson_plan.days[day].push(LessonSlot {
                    subject: name,
                    place: String::new(),
                });
                app.custom_subject_input.clear();
                app.d.sync_lessons_to_events();
            }
        });
    });
}

fn show_day_sequence(app: &mut PlannerApp, ui: &mut egui::Ui, theme: &Theme, day: usize) {
    let w = ui.available_width();
    glass_panel(ui, theme, 16, true, 12, |ui| {
        ui.set_min_width((w - 24.0).max(200.0));
        ui.label(semibold(
            format!("По очереди · {}", DAYS[day]),
            15.0,
            theme.ink,
        ));
        ui.add(
            egui::Label::new(
                RichText::new("Порядок задаётся кнопками ↑ / ↓, время считается автоматически.")
                    .size(12.0)
                    .color(theme.ink_secondary),
            )
            .wrap(),
        );
        ui.add_space(6.0);

        let len = app.d.lesson_plan.days[day].len();
        if len == 0 {
            ui.add(
                egui::Label::new(
                    RichText::new(
                        "На этот день пока нет уроков.\nПоставьте галочки слева — предметы сразу появятся здесь по порядку 1, 2, 3…",
                    )
                    .size(13.0)
                    .color(theme.ink_secondary),
                )
                .wrap(),
            );
            return;
        }

        let mut swap_pair: Option<(usize, usize)> = None;
        let mut remove_idx: Option<usize> = None;

        for idx in 0..len {
            let line_y = ui.cursor().min.y;
            ui.painter().line_segment(
                [
                    Pos2::new(ui.cursor().min.x, line_y),
                    Pos2::new(ui.cursor().min.x + ui.available_width(), line_y),
                ],
                Stroke::new(1.0_f32, theme.divider),
            );
            ui.add_space(5.0);

            let subject = app.d.lesson_plan.days[day][idx].subject.clone();
            let place = app.d.lesson_plan.days[day][idx].place.clone();
            let slot_time = app
                .d
                .lesson_plan
                .slot_times(idx)
                .map(|(st, en)| format!("{}–{}", st.format("%H:%M"), en.format("%H:%M")))
                .unwrap_or_else(|| "—".to_owned());
            let slot_sub = if place.trim().is_empty() {
                slot_time
            } else {
                format!("{slot_time} · {}", place.trim())
            };

            ui.horizontal_top(|ui| {
                // Slot ordinal badge
                let (num_r, _) = ui.allocate_exact_size(Vec2::splat(22.0), Sense::hover());
                ui.painter()
                    .rect_filled(num_r, CornerRadius::same(7), theme.accent_soft);
                let num_g = ui.painter().layout_no_wrap(
                    format!("{}", idx + 1),
                    egui::FontId::new(12.0, egui::FontFamily::Name("Semibold".into())),
                    theme.accent,
                );
                ui.painter()
                    .galley(num_r.center() - num_g.size() * 0.5, num_g, theme.accent);

                let text_w = (ui.available_width() - 88.0).max(80.0);
                let text_resp = ui.allocate_ui_with_layout(
                    Vec2::new(text_w, 0.0),
                    egui::Layout::top_down(egui::Align::Min),
                    |ui| {
                        ui.spacing_mut().item_spacing.y = 1.0;
                        ui.add(
                            egui::Label::new(semibold(subject, 13.5, theme.ink)).truncate(),
                        );
                        ui.add(
                            egui::Label::new(
                                RichText::new(slot_sub)
                                    .size(11.5)
                                    .color(theme.ink_secondary),
                            )
                            .truncate(),
                        );
                    },
                );

                let row_h = text_resp.response.rect.height().max(24.0);
                let rem_w = ui.available_width().max(78.0);
                ui.allocate_ui_with_layout(
                    Vec2::new(rem_w, row_h),
                    egui::Layout::right_to_left(egui::Align::Center),
                    |ui| {
                        ui.spacing_mut().item_spacing.x = 2.0;
                        if mini_icon_btn(
                            ui,
                            egui::Id::new(("slot_del", day, idx)),
                            Some(Icon::Trash),
                            None,
                            true,
                            "Убрать урок",
                            theme,
                        ) {
                            remove_idx = Some(idx);
                        }
                        if mini_icon_btn(
                            ui,
                            egui::Id::new(("slot_down", day, idx)),
                            Some(Icon::ArrowDown),
                            None,
                            idx + 1 < len,
                            "Ниже по очереди",
                            theme,
                        ) && idx + 1 < len
                        {
                            swap_pair = Some((idx, idx + 1));
                        }
                        if mini_icon_btn(
                            ui,
                            egui::Id::new(("slot_up", day, idx)),
                            Some(Icon::ArrowUp),
                            None,
                            idx > 0,
                            "Выше по очереди",
                            theme,
                        ) && idx > 0
                        {
                            swap_pair = Some((idx, idx - 1));
                        }
                    },
                );
            });
            ui.add_space(5.0);
        }

        if let Some((a, b)) = swap_pair {
            app.push_undo();
            app.d.lesson_plan.days[day].swap(a, b);
            app.d.sync_lessons_to_events();
        } else if let Some(idx) = remove_idx {
            app.push_undo();
            app.d.lesson_plan.days[day].remove(idx);
            app.d.sync_lessons_to_events();
        }

        // Day summary: last lesson end vs target home_time
        let line_y = ui.cursor().min.y;
        ui.painter().line_segment(
            [
                Pos2::new(ui.cursor().min.x, line_y),
                Pos2::new(ui.cursor().min.x + ui.available_width(), line_y),
            ],
            Stroke::new(1.0_f32, theme.divider),
        );
        ui.add_space(8.0);

        if let Some(last_end) = app.d.lesson_plan.day_last_end(day) {
            let home_t = time(&app.d.lesson_plan.home_time);
            let home_str = home_t
                .map(|t| t.format("%H:%M").to_string())
                .unwrap_or_else(|| app.d.lesson_plan.home_time.clone());
            ui.add(
                egui::Label::new(semibold(
                    format!("Последний урок до {} · домой в {}", last_end.format("%H:%M"), home_str),
                    13.0,
                    theme.ink,
                ))
                .wrap(),
            );
            if let Some(ht) = home_t {
                let diff = (ht - last_end).num_minutes();
                if diff < 0 {
                    ui.add(
                        egui::Label::new(
                            RichText::new(format!(
                                "Уроки заканчиваются на {} мин позже выбранного «домой».",
                                -diff
                            ))
                            .size(12.0)
                            .color(theme.danger),
                        )
                        .wrap(),
                    );
                } else {
                    ui.add(
                        egui::Label::new(
                            RichText::new(format!(
                                "Время в школе и на дорогу домой учтено в проверке свободных окон (запас {} мин).",
                                diff
                            ))
                            .size(12.0)
                            .color(theme.ink_muted),
                        )
                        .wrap(),
                    );
                }
            }
        }
    });
}

fn preset_chip(
    ui: &mut egui::Ui,
    id: egui::Id,
    label: &str,
    active: bool,
    theme: &Theme,
) -> bool {
    let font = egui::FontId::new(11.5, egui::FontFamily::Name("Semibold".into()));
    let measure = ui
        .painter()
        .layout_no_wrap(label.to_owned(), font.clone(), Color32::PLACEHOLDER);
    let w = (measure.size().x + 14.0).max(24.0);
    let (rect, resp) = ui.allocate_exact_size(Vec2::new(w, 24.0), Sense::click());
    if ui.is_rect_visible(rect) {
        let fill = if active {
            theme.accent_soft
        } else if resp.hovered() {
            theme.pill_active
        } else {
            theme.pill_track
        };
        let fg = if active || resp.hovered() {
            theme.accent
        } else {
            theme.ink_secondary
        };
        let g = ui.painter().layout_no_wrap(label.to_owned(), font, fg);
        ui.painter().rect_filled(rect, CornerRadius::same(7), fill);
        ui.painter().galley(rect.center() - g.size() * 0.5, g, fg);
    }
    let _ = id;
    resp.clicked()
}

fn mini_icon_btn(
    ui: &mut egui::Ui,
    id: egui::Id,
    icon: Option<Icon>,
    text: Option<&str>,
    enabled: bool,
    tooltip: &str,
    theme: &Theme,
) -> bool {
    let sense = if enabled { Sense::click() } else { Sense::hover() };
    let (rect, resp) = ui.allocate_exact_size(Vec2::splat(24.0), sense);
    if ui.is_rect_visible(rect) {
        if enabled && resp.hovered() {
            ui.painter()
                .rect_filled(rect, CornerRadius::same(6), theme.pill_track);
        }
        let col = if !enabled {
            theme.ink_muted.gamma_multiply(0.35)
        } else if resp.hovered() {
            theme.ink
        } else {
            theme.ink_secondary
        };
        if let Some(ic) = icon {
            draw_icon(ui.painter(), rect.center(), 12.0, ic, col, 1.45);
        } else if let Some(t) = text {
            let g = ui
                .painter()
                .layout_no_wrap(t.to_owned(), egui::FontId::proportional(13.0), col);
            ui.painter()
                .galley(rect.center() - g.size() * 0.5, g, col);
        }
    }
    let _ = id;
    if enabled {
        resp.on_hover_text(tooltip).clicked()
    } else {
        false
    }
}
