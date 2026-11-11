use crate::glass::{
    Icon, Theme, draw_icon, glass_button, glass_panel, reminders_checkbox, semibold,
};
use crate::model::{
    DAYS, Task, capacity_report, date, format_duration_ru, iso_week_key, now_time, plan,
    ru_date_long, time, today, week_range_label,
};
use crate::ui::{PlannerApp, SEC_LESSONS, SEC_SCHEDULE, SEC_SLEEP, SEC_TASKS};
use chrono::{Datelike, Duration};
use eframe::egui::{self, Pos2, RichText, Sense, Stroke, Vec2};

pub fn show(app: &mut PlannerApp, ui: &mut egui::Ui, theme: &Theme) {
    let today_d = today();
    let now_t = now_time();
    let wd = today_d.weekday().num_days_from_monday() as usize;

    // Header
    ui.horizontal(|ui| {
        ui.vertical(|ui| {
            ui.label(semibold("Сегодня", 28.0, theme.ink));
            ui.label(
                RichText::new(format!("{}, {}", DAYS[wd], ru_date_long(today_d)))
                    .size(13.5)
                    .color(theme.ink_secondary),
            );
        });
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if app.d.has_demo_data {
                if glass_button(
                    ui,
                    &mut app.motion,
                    egui::Id::new("today_clear_demo"),
                    "Очистить демо",
                    Some(Icon::Trash),
                    false,
                    theme,
                )
                .clicked()
                {
                    app.clear_demo();
                }
            } else if app.d.events.is_empty() && app.d.tasks.is_empty() {
                if glass_button(
                    ui,
                    &mut app.motion,
                    egui::Id::new("today_load_demo"),
                    "Загрузить пример дня",
                    Some(Icon::Sparkles),
                    false,
                    theme,
                )
                .clicked()
                {
                    app.load_demo();
                }
            }
        });
    });

    ui.add_space(14.0);

    if app.d.needs_weekly_lesson_check(today_d) && !app.lesson_wizard_dismissed {
        show_weekly_lesson_prompt(app, ui, theme, today_d);
        ui.add_space(14.0);
    }

    let avail_w = ui.available_width();
    if avail_w >= 640.0 {
        let gap = 28.0;
        let col_w = (avail_w - gap) * 0.5;
        ui.horizontal_top(|ui| {
            ui.allocate_ui_with_layout(
                Vec2::new(col_w, 0.0),
                egui::Layout::top_down(egui::Align::Min),
                |ui| show_events_column(app, ui, theme, today_d, now_t),
            );
            ui.add_space(gap);
            ui.allocate_ui_with_layout(
                Vec2::new(col_w, 0.0),
                egui::Layout::top_down(egui::Align::Min),
                |ui| show_tasks_column(app, ui, theme, today_d),
            );
        });
    } else {
        show_events_column(app, ui, theme, today_d, now_t);
        ui.add_space(18.0);
        show_tasks_column(app, ui, theme, today_d);
    }

    ui.add_space(16.0);
    show_evening_strip(app, ui, theme, today_d, now_t);
}

fn show_weekly_lesson_prompt(
    app: &mut PlannerApp,
    ui: &mut egui::Ui,
    theme: &Theme,
    today_d: chrono::NaiveDate,
) {
    let full_w = ui.available_width();
    let inner_w = (full_w - 24.0).max(200.0);
    let has_prev = app.d.lesson_plan.has_any_lessons();
    let range = week_range_label(today_d);
    let subtitle = if let (Some(st), Some(ht)) = (
        time(&app.d.lesson_plan.start_time),
        time(&app.d.lesson_plan.home_time),
    ) {
        format!(
            "Начало в {}, домой в {}. Проверьте галочки уроков по порядку.",
            st.format("%H:%M"),
            ht.format("%H:%M")
        )
    } else {
        "Выберите предметы галочками по очереди и укажите время начала и домой.".to_owned()
    };

    glass_panel(ui, theme, 14, false, 12, |ui| {
        ui.set_min_width(inner_w);
        let inline_actions = inner_w >= 540.0;
        let actions_w = if has_prev { 256.0 } else { 156.0 };
        let text_w = if inline_actions {
            (inner_w - 38.0 - actions_w).max(180.0)
        } else {
            (inner_w - 38.0).max(160.0)
        };

        let mut open_lessons = false;
        let mut keep_prev = false;
        let mut dismiss = false;

        ui.horizontal_top(|ui| {
            let (icon_r, _) = ui.allocate_exact_size(Vec2::splat(26.0), Sense::hover());
            ui.painter()
                .circle_filled(icon_r.center(), 13.0, theme.accent_soft);
            draw_icon(
                ui.painter(),
                icon_r.center(),
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
                    ui.add(
                        egui::Label::new(semibold(
                            format!("Начало недели ({range}) — какие уроки на этой неделе?"),
                            14.0,
                            theme.ink,
                        ))
                        .wrap(),
                    );
                    ui.add(
                        egui::Label::new(
                            RichText::new(&subtitle)
                                .size(12.5)
                                .color(theme.ink_secondary),
                        )
                        .wrap(),
                    );
                },
            );

            if inline_actions {
                let row_h = text_resp.response.rect.height().max(34.0);
                let rem_w = ui.available_width().max(actions_w);
                ui.allocate_ui_with_layout(
                    Vec2::new(rem_w, row_h),
                    egui::Layout::right_to_left(egui::Align::Center),
                    |ui| {
                        ui.spacing_mut().item_spacing.x = 8.0;
                        if ui
                            .add(
                                egui::Button::new(
                                    RichText::new("Позже")
                                        .size(12.0)
                                        .color(theme.ink_muted),
                                )
                                .frame(false),
                            )
                            .clicked()
                        {
                            dismiss = true;
                        }
                        if has_prev
                            && glass_button(
                                ui,
                                &mut app.motion,
                                egui::Id::new("today_keep_lessons_btn"),
                                "Оставить",
                                None,
                                false,
                                theme,
                            )
                            .clicked()
                        {
                            keep_prev = true;
                        }
                        if glass_button(
                            ui,
                            &mut app.motion,
                            egui::Id::new("today_setup_lessons_btn"),
                            "Уроки",
                            Some(Icon::ChevronRight),
                            true,
                            theme,
                        )
                        .clicked()
                        {
                            open_lessons = true;
                        }
                    },
                );
            }
        });

        if !inline_actions {
            ui.add_space(8.0);
            ui.horizontal_wrapped(|ui| {
                ui.add_space(34.0);
                if glass_button(
                    ui,
                    &mut app.motion,
                    egui::Id::new("today_setup_lessons_btn_narrow"),
                    "Настроить уроки",
                    Some(Icon::ChevronRight),
                    true,
                    theme,
                )
                .clicked()
                {
                    open_lessons = true;
                }
                if has_prev
                    && glass_button(
                        ui,
                        &mut app.motion,
                        egui::Id::new("today_keep_lessons_btn_narrow"),
                        "Оставить",
                        None,
                        false,
                        theme,
                    )
                    .clicked()
                {
                    keep_prev = true;
                }
                if ui
                    .add(
                        egui::Button::new(
                            RichText::new("Позже")
                                .size(12.0)
                                .color(theme.ink_muted),
                        )
                        .frame(false),
                    )
                    .clicked()
                {
                    dismiss = true;
                }
            });
        }

        if open_lessons {
            app.lesson_day = 0;
            app.section = SEC_LESSONS;
        } else if keep_prev {
            app.push_undo();
            app.d.lesson_plan.confirmed_week = iso_week_key(today_d);
            app.d.sync_lessons_to_events();
            app.notify("Расписание уроков подтверждено на эту неделю");
        } else if dismiss {
            app.lesson_wizard_dismissed = true;
        }
    });
}

fn show_events_column(
    app: &mut PlannerApp,
    ui: &mut egui::Ui,
    theme: &Theme,
    today_d: chrono::NaiveDate,
    now_t: chrono::NaiveTime,
) {
    ui.horizontal(|ui| {
        ui.label(semibold("Занятия", 16.5, theme.ink));
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui
                .add(
                    egui::Button::new(RichText::new("+ В расписание").size(12.5).color(theme.accent))
                        .frame(false),
                )
                .clicked()
            {
                app.section = SEC_SCHEDULE;
                app.show_event_form = true;
            }
            if ui
                .add(
                    egui::Button::new(RichText::new("Уроки").size(12.5).color(theme.ink_secondary))
                        .frame(false),
                )
                .clicked()
            {
                app.lesson_day = today_d.weekday().num_days_from_monday() as usize;
                app.section = SEC_LESSONS;
            }
        });
    });
    ui.add_space(8.0);

    let mut events: Vec<_> = app
        .d
        .events
        .iter()
        .filter(|e| e.occurs(today_d))
        .cloned()
        .collect();
    events.sort_by_key(|e| time(&e.start));

    if events.is_empty() {
        ui.painter().line_segment(
            [
                ui.cursor().min,
                Pos2::new(ui.cursor().min.x + ui.available_width(), ui.cursor().min.y),
            ],
            Stroke::new(1.0_f32, theme.divider),
        );
        ui.add_space(14.0);
        ui.label(RichText::new("На сегодня занятий нет.").size(14.0).color(theme.ink_secondary));
        ui.add_space(8.0);
        if glass_button(
            ui,
            &mut app.motion,
            egui::Id::new("empty_add_event_btn"),
            "Добавить занятие",
            Some(Icon::Plus),
            false,
            theme,
        )
        .clicked()
        {
            app.section = SEC_SCHEDULE;
            app.show_event_form = true;
        }
        return;
    }

    // Highlight current or next upcoming event
    let next_idx = events
        .iter()
        .position(|e| time(&e.end).is_some_and(|end_t| end_t > now_t));

    for (idx, ev) in events.iter().enumerate() {
        let line_y = ui.cursor().min.y;
        ui.painter().line_segment(
            [
                Pos2::new(ui.cursor().min.x, line_y),
                Pos2::new(ui.cursor().min.x + ui.available_width(), line_y),
            ],
            Stroke::new(1.0_f32, theme.divider),
        );

        let is_next = next_idx == Some(idx);
        let is_past = next_idx.is_none() || next_idx.is_some_and(|n| idx < n);
        let cat_col = theme.category_color(ev.effective_category());

        ui.add_space(10.0);
        ui.horizontal_top(|ui| {
            // Fixed tabular-width time column
            ui.allocate_ui_with_layout(
                Vec2::new(52.0, 22.0),
                egui::Layout::left_to_right(egui::Align::Min),
                |ui| {
                    let t_col = if is_next {
                        theme.accent
                    } else if is_past {
                        theme.ink_muted
                    } else {
                        theme.ink_secondary
                    };
                    ui.label(RichText::new(&ev.start).size(13.0).color(t_col));
                },
            );

            let text_w = (ui.available_width() - 8.0).max(120.0);
            ui.allocate_ui_with_layout(
                Vec2::new(text_w, 0.0),
                egui::Layout::top_down(egui::Align::Min),
                |ui| {
                    ui.spacing_mut().item_spacing.y = 3.0;
                    ui.horizontal_wrapped(|ui| {
                        let title_col = if is_next {
                            theme.accent
                        } else if is_past {
                            theme.ink_secondary
                        } else {
                            theme.ink
                        };
                        ui.label(semibold(&ev.name, 15.0, title_col));

                        // Subtle category pill with explicit label
                        let (dot_rect, _) =
                            ui.allocate_exact_size(Vec2::new(8.0, 16.0), Sense::hover());
                        ui.painter().circle_filled(dot_rect.center(), 3.2, cat_col);
                        ui.label(
                            RichText::new(ev.effective_category())
                                .size(11.5)
                                .color(theme.ink_muted),
                        );
                    });

                    let mut meta = format!("До {}", ev.end);
                    if !ev.place.trim().is_empty() {
                        meta.push_str(&format!(" · {}", ev.place.trim()));
                    }
                    if ev.travel > 0 {
                        meta.push_str(&format!(" · дорога {} мин", ev.travel));
                    }
                    ui.add(
                        egui::Label::new(
                            RichText::new(meta).size(12.5).color(theme.ink_secondary),
                        )
                        .wrap(),
                    );
                },
            );
        });
        ui.add_space(10.0);
    }

    if events.iter().any(|e| e.from_lessons) {
        if let Some(home_t) = time(&app.d.lesson_plan.home_time) {
            ui.add_space(2.0);
            ui.label(
                RichText::new(format!("Домой после уроков: {}", home_t.format("%H:%M")))
                    .size(12.5)
                    .color(theme.ink_secondary),
            );
        }
    }
}

fn show_tasks_column(
    app: &mut PlannerApp,
    ui: &mut egui::Ui,
    theme: &Theme,
    today_d: chrono::NaiveDate,
) {
    ui.horizontal(|ui| {
        ui.label(semibold("Сделать сегодня", 16.5, theme.ink));
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui
                .add(
                    egui::Button::new(RichText::new("Все задачи →").size(12.5).color(theme.accent))
                        .frame(false),
                )
                .clicked()
            {
                app.section = SEC_TASKS;
            }
        });
    });
    ui.add_space(8.0);

    // Inline quick-add task strip
    ui.horizontal(|ui| {
        let input_w = (ui.available_width() - 58.0 - 34.0 - 20.0).max(110.0);
        let te = ui.add_sized(
            [input_w, 32.0],
            egui::TextEdit::singleline(&mut app.quick_task_title)
                .hint_text("Быстро добавить дело на сегодня...")
                .margin(egui::Margin::symmetric(10, 6)),
        );
        ui.add_sized(
            [58.0, 32.0],
            egui::DragValue::new(&mut app.quick_task_minutes)
                .range(1..=360)
                .suffix(" м"),
        );
        let enter_pressed = te.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));
        let clicked = glass_button(
            ui,
            &mut app.motion,
            egui::Id::new("quick_add_task_btn"),
            "",
            Some(Icon::Plus),
            true,
            theme,
        )
        .clicked();

        if (enter_pressed || clicked) && !app.quick_task_title.trim().is_empty() {
            app.push_undo();
            app.d.tasks.push(Task {
                title: app.quick_task_title.trim().to_owned(),
                subject: String::new(),
                due: today_d.to_string(),
                minutes: app.quick_task_minutes.max(1),
                done: false,
            });
            app.quick_task_title.clear();
            app.notify("Задача добавлена");
        }
    });

    ui.add_space(10.0);

    let mut visible_indices: Vec<usize> = app
        .d
        .tasks
        .iter()
        .enumerate()
        .filter(|(_, t)| t.due.trim().is_empty() || date(&t.due).is_some_and(|d| d <= today_d))
        .map(|(i, _)| i)
        .collect();

    // Sort active tasks first, completed after
    visible_indices.sort_by_key(|&i| app.d.tasks[i].done);

    if visible_indices.is_empty() {
        ui.painter().line_segment(
            [
                ui.cursor().min,
                Pos2::new(ui.cursor().min.x + ui.available_width(), ui.cursor().min.y),
            ],
            Stroke::new(1.0_f32, theme.divider),
        );
        ui.add_space(14.0);
        ui.label(
            RichText::new("На сегодня задач нет. Добавь дело выше или в разделе «Задачи».")
                .size(14.0)
                .color(theme.ink_secondary),
        );
        return;
    }

    let mut toggled_idx = None;
    for &idx in &visible_indices {
        let line_y = ui.cursor().min.y;
        ui.painter().line_segment(
            [
                Pos2::new(ui.cursor().min.x, line_y),
                Pos2::new(ui.cursor().min.x + ui.available_width(), line_y),
            ],
            Stroke::new(1.0_f32, theme.divider),
        );
        ui.add_space(9.0);
        let t = &mut app.d.tasks[idx];
        let mut checked = t.done;
        let title = t.title.clone();
        let subject = t.subject.clone();
        let minutes = t.minutes;
        let due = t.due.clone();

        ui.horizontal_top(|ui| {
            if reminders_checkbox(
                ui,
                &mut app.motion,
                egui::Id::new(("today_task_cb", idx)),
                &mut checked,
                theme,
            ) {
                toggled_idx = Some((idx, checked));
            }
            let text_w = (ui.available_width() - 6.0).max(110.0);
            ui.allocate_ui_with_layout(
                Vec2::new(text_w, 0.0),
                egui::Layout::top_down(egui::Align::Min),
                |ui| {
                    ui.spacing_mut().item_spacing.y = 2.0;
                    ui.add_space(2.0);
                    let rt = if checked {
                        RichText::new(&title)
                            .size(14.5)
                            .strikethrough()
                            .color(theme.ink_muted)
                    } else {
                        RichText::new(&title).size(14.5).color(theme.ink)
                    };
                    ui.add(egui::Label::new(rt).wrap());

                    let mut meta = Vec::new();
                    if !subject.trim().is_empty() {
                        meta.push(subject);
                    }
                    meta.push(format_duration_ru(minutes));
                    if let Some(d) = date(&due) {
                        if d < today_d {
                            meta.push(format!("срок просрочен ({})", d.format("%d.%m")));
                        }
                    }
                    ui.add(
                        egui::Label::new(
                            RichText::new(meta.join(" · "))
                                .size(12.5)
                                .color(theme.ink_secondary),
                        )
                        .wrap(),
                    );
                },
            );
        });
        ui.add_space(9.0);
    }

    if let Some((idx, new_val)) = toggled_idx {
        app.push_undo();
        app.d.tasks[idx].done = new_val;
    }

    let remaining_count = visible_indices
        .iter()
        .filter(|&&i| !app.d.tasks[i].done)
        .count();
    let remaining_mins: i64 = visible_indices
        .iter()
        .filter(|&&i| !app.d.tasks[i].done)
        .map(|&i| app.d.tasks[i].minutes.max(0))
        .sum();

    ui.add_space(6.0);
    let summary = if remaining_count == 0 {
        "Все дела на сегодня выполнены".to_owned()
    } else {
        format!(
            "Осталось дел: {remaining_count} · {}",
            format_duration_ru(remaining_mins)
        )
    };
    ui.label(RichText::new(summary).size(12.5).color(theme.ink_secondary));
}

fn show_evening_strip(
    app: &mut PlannerApp,
    ui: &mut egui::Ui,
    theme: &Theme,
    today_d: chrono::NaiveDate,
    now_t: chrono::NaiveTime,
) {
    let p = plan(&app.d, today_d + Duration::days(1));
    let cap = capacity_report(&app.d, today_d, now_t);

    let full_w = ui.available_width();
    let inner_w = (full_w - 32.0).max(200.0);
    let wake_reason = match (&p.first_event_name, p.first_event_time) {
        (Some(name), Some(t)) => format!("первое завтра: {} в {}", name, t.format("%H:%M")),
        _ => "свободный день".to_owned(),
    };
    let cap_info = if cap.task_count > 0 {
        let cap_text = if cap.fits {
            format!(
                "Свободные окна до {}: {} (макс. блок {}) — текущие задачи ({}) помещаются",
                p.finish.format("%H:%M"),
                format_duration_ru(cap.total_free_mins),
                format_duration_ru(cap.largest_window_mins),
                format_duration_ru(cap.task_mins),
            )
        } else {
            format!(
                "Не помещается задач в свободные окна: {} (свободно {}, макс. окно {})",
                cap.unfit_count,
                format_duration_ru(cap.total_free_mins),
                format_duration_ru(cap.largest_window_mins),
            )
        };
        let cap_col = if cap.fits {
            theme.ink_muted
        } else {
            theme.danger
        };
        Some((cap_text, cap_col))
    } else {
        None
    };

    glass_panel(ui, theme, 16, true, 16, |ui| {
        ui.set_min_width(inner_w);
        let stack_btn = inner_w < 430.0;
        let btn_w = 144.0;
        let text_w = if stack_btn {
            (inner_w - 40.0).max(140.0)
        } else {
            (inner_w - 28.0 - 10.0 - btn_w - 14.0).max(160.0)
        };

        let mut open_sleep = false;
        ui.horizontal_top(|ui| {
            let (icon_rect, _) = ui.allocate_exact_size(Vec2::splat(28.0), Sense::hover());
            ui.painter()
                .circle_filled(icon_rect.center(), 14.0, theme.accent_soft);
            draw_icon(
                ui.painter(),
                icon_rect.center(),
                15.0,
                Icon::Moon,
                theme.accent,
                1.6,
            );

            let left_resp = ui.allocate_ui_with_layout(
                Vec2::new(text_w, 0.0),
                egui::Layout::top_down(egui::Align::Min),
                |ui| {
                    ui.spacing_mut().item_spacing.y = 3.0;
                    ui.horizontal_wrapped(|ui| {
                        ui.label(semibold(
                            format!("Завершить дела до {}", p.finish.format("%H:%M")),
                            15.0,
                            theme.ink,
                        ));
                        ui.label(
                            RichText::new(format!("({})", p.finish.format("%d.%m")))
                                .size(12.5)
                                .color(theme.ink_muted),
                        );
                    });
                    ui.add(
                        egui::Label::new(
                            RichText::new(format!(
                                "Лечь {} ({}) · Подъём завтра {} ({wake_reason})",
                                p.bed.format("%H:%M"),
                                p.bed.format("%d.%m"),
                                p.wake.format("%H:%M"),
                            ))
                            .size(12.5)
                            .color(theme.ink_secondary),
                        )
                        .wrap(),
                    );
                    if let Some((ref cap_text, cap_col)) = cap_info {
                        ui.add(
                            egui::Label::new(RichText::new(cap_text).size(12.0).color(cap_col))
                                .wrap(),
                        );
                    }
                },
            );

            if !stack_btn {
                let row_h = left_resp.response.rect.height().max(34.0);
                let rem_w = ui.available_width().max(btn_w);
                ui.allocate_ui_with_layout(
                    Vec2::new(rem_w, row_h),
                    egui::Layout::right_to_left(egui::Align::Center),
                    |ui| {
                        if glass_button(
                            ui,
                            &mut app.motion,
                            egui::Id::new("today_to_sleep_btn"),
                            "План вечера",
                            Some(Icon::ChevronRight),
                            false,
                            theme,
                        )
                        .clicked()
                        {
                            open_sleep = true;
                        }
                    },
                );
            }
        });

        if stack_btn {
            ui.add_space(10.0);
            ui.horizontal(|ui| {
                ui.add_space(38.0);
                if glass_button(
                    ui,
                    &mut app.motion,
                    egui::Id::new("today_to_sleep_btn"),
                    "План вечера",
                    Some(Icon::ChevronRight),
                    false,
                    theme,
                )
                .clicked()
                {
                    open_sleep = true;
                }
            });
        }

        if open_sleep {
            app.section = SEC_SLEEP;
        }
    });
}
