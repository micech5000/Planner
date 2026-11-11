use crate::glass::{
    GlassSurfaceSpec, Icon, Theme, draw_glass_surface, glass_button, icon_button,
    segmented_control, semibold, truncate_to_width,
};
use crate::model::{CATEGORIES, DAYS, DAYS_SHORT, Event, date, now_time, time, today};
use crate::ui::{PlannerApp, form_columns, labeled_input};
use chrono::{Datelike, Duration, Timelike};
use eframe::egui::{
    self, Color32, CornerRadius, FontFamily, FontId, Pos2, Rect, RichText, Sense, Stroke,
    StrokeKind, Vec2,
};

pub fn show(app: &mut PlannerApp, ui: &mut egui::Ui, theme: &Theme) {
    let today_d = today();

    let avail = ui.available_width();
    if avail >= 660.0 {
        let ctrl_est_w = 344.0;
        let left_w = (avail - ctrl_est_w - 14.0).max(200.0);
        ui.horizontal_top(|ui| {
            let left_resp = ui.allocate_ui_with_layout(
                Vec2::new(left_w, 0.0),
                egui::Layout::top_down(egui::Align::Min),
                |ui| {
                    ui.label(semibold("Расписание", 28.0, theme.ink));
                    ui.add(
                        egui::Label::new(
                            RichText::new("Повторяющаяся неделя, разовые даты и время на дорогу")
                                .size(13.5)
                                .color(theme.ink_secondary),
                        )
                        .wrap(),
                    );
                },
            );
            let row_h = left_resp.response.rect.height().max(34.0);
            let rem_w = ui.available_width().max(ctrl_est_w);
            ui.allocate_ui_with_layout(
                Vec2::new(rem_w, row_h),
                egui::Layout::right_to_left(egui::Align::Center),
                |ui| {
                    if glass_button(
                        ui,
                        &mut app.motion,
                        egui::Id::new("sched_toggle_form"),
                        if app.show_event_form || app.edit_event.is_some() {
                            "Скрыть форму"
                        } else {
                            "Новое занятие"
                        },
                        Some(Icon::Plus),
                        true,
                        theme,
                    )
                    .clicked()
                    {
                        if app.show_event_form || app.edit_event.is_some() {
                            app.show_event_form = false;
                            app.edit_event = None;
                            app.event = Event {
                                day: app.selected_day,
                                ..Default::default()
                            };
                        } else {
                            app.show_event_form = true;
                            app.event = Event {
                                day: app.selected_day,
                                ..Default::default()
                            };
                        }
                    }

                    let mut view_idx = if app.d.schedule_grid { 0 } else { 1 };
                    if segmented_control(
                        ui,
                        &mut app.motion,
                        egui::Id::new("sched_view_mode"),
                        &mut view_idx,
                        &["Сетка", "Список"],
                        theme,
                    ) {
                        app.d.schedule_grid = view_idx == 0;
                    }
                },
            );
        });
    } else {
        ui.label(semibold("Расписание", 28.0, theme.ink));
        ui.add(
            egui::Label::new(
                RichText::new("Повторяющаяся неделя, разовые даты и время на дорогу")
                    .size(13.5)
                    .color(theme.ink_secondary),
            )
            .wrap(),
        );
        ui.add_space(8.0);
        ui.horizontal_wrapped(|ui| {
            let mut view_idx = if app.d.schedule_grid { 0 } else { 1 };
            if segmented_control(
                ui,
                &mut app.motion,
                egui::Id::new("sched_view_mode"),
                &mut view_idx,
                &["Сетка", "Список"],
                theme,
            ) {
                app.d.schedule_grid = view_idx == 0;
            }
            if glass_button(
                ui,
                &mut app.motion,
                egui::Id::new("sched_toggle_form"),
                if app.show_event_form || app.edit_event.is_some() {
                    "Скрыть форму"
                } else {
                    "Новое занятие"
                },
                Some(Icon::Plus),
                true,
                theme,
            )
            .clicked()
            {
                if app.show_event_form || app.edit_event.is_some() {
                    app.show_event_form = false;
                    app.edit_event = None;
                    app.event = Event {
                        day: app.selected_day,
                        ..Default::default()
                    };
                } else {
                    app.show_event_form = true;
                    app.event = Event {
                        day: app.selected_day,
                        ..Default::default()
                    };
                }
            }
        });
    }

    ui.add_space(12.0);

    // Legend with text labels (never color-only coding)
    ui.horizontal_wrapped(|ui| {
        ui.spacing_mut().item_spacing.x = 14.0;
        for cat in CATEGORIES {
            let col = theme.category_color(cat);
            let (r, _) = ui.allocate_exact_size(Vec2::new(10.0, 14.0), Sense::hover());
            ui.painter().circle_filled(r.center(), 4.0, col);
            ui.label(RichText::new(cat).size(12.5).color(theme.ink_secondary));
        }
    });

    ui.add_space(10.0);

    if app.d.schedule_grid {
        show_weekly_grid(app, ui, theme, today_d);
    } else {
        show_day_list(app, ui, theme, today_d);
    }

    if app.show_event_form || app.edit_event.is_some() {
        ui.add_space(18.0);
        show_event_editor(app, ui, theme, today_d);
    }
}

fn show_weekly_grid(
    app: &mut PlannerApp,
    ui: &mut egui::Ui,
    theme: &Theme,
    today_d: chrono::NaiveDate,
) {
    let wd_offset = today_d.weekday().num_days_from_monday() as i64;
    let monday = today_d - Duration::days(wd_offset);

    if app.d.events.is_empty() {
        ui.add_space(8.0);
        ui.label(
            RichText::new("Пока нет занятий. Нажми «Новое занятие», чтобы заполнить неделю.")
                .size(14.0)
                .color(theme.ink_secondary),
        );
        ui.add_space(8.0);
    }

    let draw_grid_body = |app: &mut PlannerApp, ui: &mut egui::Ui| {
        let start_hour = 7;
        let end_hour = 22;
        let hour_h = 32.5;
        let header_h = 36.0;
        let gutter_w = 48.0;
        let total_w = ui.available_width().max(520.0);
        let col_w = ((total_w - gutter_w) / 7.0).max(60.0);
        let body_h = (end_hour - start_hour) as f32 * hour_h;

        let (grid_rect, _) =
            ui.allocate_exact_size(Vec2::new(gutter_w + col_w * 7.0, header_h + body_h), Sense::hover());

        draw_glass_surface(
            ui.painter(),
            grid_rect,
            GlassSurfaceSpec {
                radius: 14,
                fill: theme.surface,
                elevated: false,
                hover_t: 0.0,
            },
            theme,
        );

        let p = ui.painter();

        // Header row + column highlights
        for c in 0..7 {
            let col_date = monday + Duration::days(c as i64);
            let is_today = col_date == today_d;
            let x0 = grid_rect.min.x + gutter_w + c as f32 * col_w;
            let col_rect = Rect::from_min_size(
                Pos2::new(x0, grid_rect.min.y + header_h),
                Vec2::new(col_w, body_h),
            );

            if is_today {
                p.rect_filled(col_rect, CornerRadius::ZERO, theme.accent_soft.gamma_multiply(0.35));
            }

            // Column separator
            p.line_segment(
                [
                    Pos2::new(x0, grid_rect.min.y + 4.0),
                    Pos2::new(x0, grid_rect.max.y - 4.0),
                ],
                Stroke::new(1.0_f32, theme.divider),
            );

            let head_rect = Rect::from_min_size(Pos2::new(x0, grid_rect.min.y), Vec2::new(col_w, header_h));
            let label = format!("{} {}", DAYS_SHORT[c], col_date.day());
            if is_today {
                let pill = Rect::from_center_size(head_rect.center(), Vec2::new((col_w - 10.0).min(66.0), 24.0));
                p.rect_filled(pill, CornerRadius::same(8), theme.accent);
                let fg = if theme.dark {
                    Color32::from_rgb(16, 20, 28)
                } else {
                    Color32::WHITE
                };
                let g = p.layout_no_wrap(label, FontId::new(12.5, FontFamily::Name("Semibold".into())), fg);
                p.galley(pill.center() - g.size() * 0.5, g, fg);
            } else {
                let g = p.layout_no_wrap(label, FontId::proportional(12.5), theme.ink_secondary);
                p.galley(head_rect.center() - g.size() * 0.5, g, theme.ink_secondary);
            }
        }

        // Hour horizontal lines and labels
        for h in start_hour..=end_hour {
            let y = grid_rect.min.y + header_h + (h - start_hour) as f32 * hour_h;
            p.line_segment(
                [
                    Pos2::new(grid_rect.min.x + gutter_w, y),
                    Pos2::new(grid_rect.max.x, y),
                ],
                Stroke::new(1.0_f32, theme.divider),
            );
            if h < end_hour {
                let lbl = format!("{h:02}:00");
                let g = p.layout_no_wrap(lbl, FontId::proportional(11.5), theme.ink_muted);
                p.galley(
                    Pos2::new(grid_rect.min.x + 8.0, y - g.size().y * 0.45),
                    g,
                    theme.ink_muted,
                );
            }
        }

        // Current time indicator line on today's column
        let now_t = now_time();
        let now_mins = now_t.num_seconds_from_midnight() as f32 / 60.0;
        let min_grid = (start_hour * 60) as f32;
        let max_grid = (end_hour * 60) as f32;
        if now_mins >= min_grid && now_mins <= max_grid {
            let y = grid_rect.min.y + header_h + ((now_mins - min_grid) / 60.0) * hour_h;
            let x0 = grid_rect.min.x + gutter_w + wd_offset as f32 * col_w;
            p.line_segment(
                [Pos2::new(x0, y), Pos2::new(x0 + col_w, y)],
                Stroke::new(1.6_f32, theme.danger),
            );
            p.circle_filled(Pos2::new(x0 + 3.0, y), 3.5, theme.danger);
        }

        // Render event blocks
        let mut clicked_event: Option<(usize, chrono::NaiveDate)> = None;
        for c in 0..7 {
            let col_date = monday + Duration::days(c as i64);
            let x0 = grid_rect.min.x + gutter_w + c as f32 * col_w;

            for (ev_idx, ev) in app.d.events.iter().enumerate() {
                if !ev.occurs(col_date) {
                    continue;
                }
                let (Some(st), Some(en)) = (time(&ev.start), time(&ev.end)) else {
                    continue;
                };
                let sm = (st.num_seconds_from_midnight() as f32 / 60.0).clamp(min_grid, max_grid);
                let em = (en.num_seconds_from_midnight() as f32 / 60.0).clamp(min_grid, max_grid);
                if em <= sm {
                    continue;
                }
                let y0 = grid_rect.min.y + header_h + ((sm - min_grid) / 60.0) * hour_h + 1.0;
                let h = (((em - sm) / 60.0) * hour_h - 2.0).max(20.0);
                let ev_rect = Rect::from_min_size(Pos2::new(x0 + 3.0, y0), Vec2::new(col_w - 6.0, h));

                let cat_col = theme.category_color(ev.effective_category());
                let selected = app.edit_event == Some(ev_idx);
                let resp = ui.interact(ev_rect, egui::Id::new(("grid_ev", c, ev_idx)), Sense::click());
                if resp.clicked() {
                    clicked_event = Some((ev_idx, col_date));
                }

                let fill_alpha = if selected {
                    0.28
                } else if resp.hovered() {
                    0.22
                } else {
                    0.15
                };
                let ev_p = ui.painter_at(ev_rect);
                ev_p.rect_filled(ev_rect, CornerRadius::same(7), cat_col.gamma_multiply(fill_alpha));
                ev_p.rect_stroke(
                    ev_rect,
                    CornerRadius::same(7),
                    Stroke::new(if selected { 1.4_f32 } else { 0.8_f32 }, cat_col.gamma_multiply(0.55)),
                    StrokeKind::Inside,
                );

                // Left accent strip inside rounded container
                let bar = Rect::from_min_size(
                    Pos2::new(ev_rect.min.x + 3.0, ev_rect.min.y + 4.0),
                    Vec2::new(2.5, (ev_rect.height() - 8.0).max(4.0)),
                );
                ev_p.rect_filled(bar, CornerRadius::same(2), cat_col);

                let max_txt_w = (ev_rect.width() - 13.0).max(20.0);
                let title_font = FontId::new(12.0, FontFamily::Name("Semibold".into()));
                let show_sub = ev_rect.height() >= 31.0;

                if ev_rect.height() >= 50.0 {
                    let title_g = ev_p.layout(ev.name.clone(), title_font, theme.ink, max_txt_w);
                    ev_p.galley(
                        Pos2::new(ev_rect.min.x + 9.0, ev_rect.min.y + 4.0),
                        title_g,
                        theme.ink,
                    );
                } else {
                    let title_txt = truncate_to_width(&ev_p, &ev.name, &title_font, max_txt_w);
                    let title_g = ev_p.layout_no_wrap(title_txt, title_font, theme.ink);
                    let title_y = if show_sub {
                        ev_rect.min.y + 2.5
                    } else {
                        (ev_rect.center().y - title_g.size().y * 0.5).max(ev_rect.min.y + 2.0)
                    };
                    ev_p.galley(Pos2::new(ev_rect.min.x + 9.0, title_y), title_g, theme.ink);
                }

                if show_sub {
                    let sub = if ev.place.trim().is_empty() {
                        format!("{}–{}", ev.start, ev.end)
                    } else {
                        format!("{} · {}", ev.start, ev.place.trim())
                    };
                    let sub_font = FontId::proportional(10.5);
                    let sub_txt = truncate_to_width(&ev_p, &sub, &sub_font, max_txt_w);
                    let sub_g = ev_p.layout_no_wrap(sub_txt, sub_font, theme.ink_secondary);
                    ev_p.galley(
                        Pos2::new(ev_rect.min.x + 9.0, ev_rect.min.y + 17.5),
                        sub_g,
                        theme.ink_secondary,
                    );
                }

                let mut tip = format!("{} ({})\n{}–{}", ev.name, ev.effective_category(), ev.start, ev.end);
                if !ev.place.trim().is_empty() {
                    tip.push_str(&format!(" · {}", ev.place.trim()));
                }
                if ev.travel > 0 {
                    tip.push_str(&format!("\nДорога: {} мин", ev.travel));
                }
                resp.on_hover_text(tip);
            }
        }

        if let Some((ev_idx, col_date)) = clicked_event {
            app.event = app.d.events[ev_idx].clone();
            app.edit_event = Some(ev_idx);
            app.show_event_form = true;
            app.override_date_input = col_date.to_string();
        }
    };

    if ui.available_width() < 520.0 {
        egui::ScrollArea::horizontal()
            .id_salt("sched_grid_hscroll")
            .show(ui, |ui| {
                draw_grid_body(app, ui);
            });
    } else {
        draw_grid_body(app, ui);
    }

    // Any one-off events outside this week
    let sunday = monday + Duration::days(6);
    let outside: Vec<(usize, &Event)> = app
        .d
        .events
        .iter()
        .enumerate()
        .filter(|(_, e)| {
            !e.on_date.trim().is_empty()
                && date(&e.on_date).is_some_and(|d| d < monday || d > sunday)
        })
        .collect();
    if !outside.is_empty() {
        ui.add_space(14.0);
        ui.label(semibold("Отдельные даты вне текущей недели", 14.5, theme.ink));
        ui.add_space(6.0);
        let mut to_edit = None;
        for (idx, ev) in outside {
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new(format!("{} · {}–{} {}", ev.on_date, ev.start, ev.end, ev.name))
                        .size(13.5)
                        .color(theme.ink_secondary),
                );
                if ui.small_button("Открыть").clicked() {
                    to_edit = Some(idx);
                }
            });
        }
        if let Some(idx) = to_edit {
            app.event = app.d.events[idx].clone();
            app.edit_event = Some(idx);
            app.show_event_form = true;
        }
    }
}

fn show_day_list(
    app: &mut PlannerApp,
    ui: &mut egui::Ui,
    theme: &Theme,
    today_d: chrono::NaiveDate,
) {
    segmented_control(
        ui,
        &mut app.motion,
        egui::Id::new("sched_day_tabs"),
        &mut app.selected_day,
        &DAYS_SHORT,
        theme,
    );
    ui.add_space(14.0);

    let mut indices: Vec<usize> = app
        .d
        .events
        .iter()
        .enumerate()
        .filter(|(_, e)| {
            if e.on_date.trim().is_empty() {
                e.day == app.selected_day
            } else {
                date(&e.on_date).is_some_and(|d| d.weekday().num_days_from_monday() as usize == app.selected_day)
            }
        })
        .map(|(i, _)| i)
        .collect();
    indices.sort_by_key(|&i| time(&app.d.events[i].start));

    if indices.is_empty() {
        ui.label(
            RichText::new(format!("В {} занятий нет.", DAYS[app.selected_day].to_lowercase()))
                .size(14.0)
                .color(theme.ink_secondary),
        );
    }

    let mut edit = None;
    let mut remove = None;
    for idx in indices {
        let ev = &app.d.events[idx];
        let line_y = ui.cursor().min.y;
        ui.painter().line_segment(
            [
                Pos2::new(ui.cursor().min.x, line_y),
                Pos2::new(ui.cursor().min.x + ui.available_width(), line_y),
            ],
            Stroke::new(1.0_f32, theme.divider),
        );
        ui.add_space(8.0);
        ui.horizontal_top(|ui| {
            let cat_col = theme.category_color(ev.effective_category());
            let (dot_r, _) = ui.allocate_exact_size(Vec2::new(10.0, 20.0), Sense::hover());
            ui.painter().circle_filled(dot_r.center(), 4.0, cat_col);

            let text_w = (ui.available_width() - 88.0).max(120.0);
            let left_resp = ui.allocate_ui_with_layout(
                Vec2::new(text_w, 0.0),
                egui::Layout::top_down(egui::Align::Min),
                |ui| {
                    ui.horizontal_wrapped(|ui| {
                        ui.label(semibold(
                            format!("{}–{}  {}", ev.start, ev.end, ev.name),
                            14.5,
                            theme.ink,
                        ));
                        ui.label(
                            RichText::new(ev.effective_category())
                                .size(12.0)
                                .color(theme.ink_muted),
                        );
                    });
                    let mut meta = Vec::new();
                    if !ev.place.trim().is_empty() {
                        meta.push(ev.place.clone());
                    }
                    if ev.travel > 0 {
                        meta.push(format!("дорога {} мин", ev.travel));
                    }
                    if !ev.on_date.trim().is_empty() {
                        meta.push(format!("только {}", ev.on_date));
                    }
                    if !ev.skip_dates.trim().is_empty() {
                        meta.push(format!("кроме: {}", ev.skip_dates));
                    }
                    if !meta.is_empty() {
                        ui.add(
                            egui::Label::new(
                                RichText::new(meta.join(" · "))
                                    .size(12.5)
                                    .color(theme.ink_secondary),
                            )
                            .wrap(),
                        );
                    }
                },
            );

            let row_h = left_resp.response.rect.height().max(32.0);
            let rem_w = ui.available_width().max(72.0);
            ui.allocate_ui_with_layout(
                Vec2::new(rem_w, row_h),
                egui::Layout::right_to_left(egui::Align::Center),
                |ui| {
                    if icon_button(
                        ui,
                        &mut app.motion,
                        egui::Id::new(("del_ev", idx)),
                        Icon::Trash,
                        false,
                        "Удалить занятие",
                        theme,
                    )
                    .clicked()
                    {
                        remove = Some(idx);
                    }
                    if icon_button(
                        ui,
                        &mut app.motion,
                        egui::Id::new(("edit_ev", idx)),
                        Icon::Edit,
                        app.edit_event == Some(idx),
                        "Изменить занятие",
                        theme,
                    )
                    .clicked()
                    {
                        edit = Some(idx);
                    }
                },
            );
        });
        ui.add_space(8.0);
    }

    if let Some(i) = edit {
        app.event = app.d.events[i].clone();
        app.edit_event = Some(i);
        app.show_event_form = true;
        // Suggest next date of selected_day
        let cur_wd = today_d.weekday().num_days_from_monday() as i64;
        let diff = (app.selected_day as i64 - cur_wd + 7) % 7;
        app.override_date_input = (today_d + Duration::days(diff)).to_string();
    }
    if let Some(i) = remove {
        app.push_undo();
        app.d.events.remove(i);
        app.edit_event = None;
        app.notify("Занятие удалено");
    }
}

fn show_event_editor(
    app: &mut PlannerApp,
    ui: &mut egui::Ui,
    theme: &Theme,
    today_d: chrono::NaiveDate,
) {
    let w = ui.available_width();
    crate::glass::glass_panel(ui, theme, 16, true, 18, |ui| {
        ui.set_min_width((w - 36.0).max(200.0));
        ui.horizontal(|ui| {
            ui.label(semibold(
                if app.edit_event.is_some() {
                    "Редактировать занятие"
                } else {
                    "Новое занятие"
                },
                16.0,
                theme.ink,
            ));
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui
                    .add(
                        egui::Button::new(RichText::new("Закрыть").size(12.5).color(theme.ink_secondary))
                            .frame(false),
                    )
                    .clicked()
                {
                    app.show_event_form = false;
                    app.edit_event = None;
                }
            });
        });
        ui.add_space(10.0);

        let form_w = ui.available_width();
        let gap_x = 12.0;
        let (cols1, cw1) = form_columns(form_w, 3, 170.0, gap_x);
        if cols1 >= 3 {
            ui.horizontal_top(|ui| {
                ui.spacing_mut().item_spacing.x = gap_x;
                labeled_input(ui, "Название / предмет", &mut app.event.name, cw1, theme);
                labeled_input(ui, "Кабинет или место", &mut app.event.place, cw1, theme);
                ui.allocate_ui_with_layout(
                    Vec2::new(cw1, 0.0),
                    egui::Layout::top_down(egui::Align::Min),
                    |ui| {
                        ui.spacing_mut().item_spacing.y = 4.0;
                        ui.spacing_mut().interact_size.y = 32.0;
                        ui.label(RichText::new("Категория").size(12.5).color(theme.ink_secondary));
                        egui::ComboBox::from_id_salt("ev_cat_combo")
                            .width(cw1)
                            .selected_text(app.event.effective_category())
                            .show_ui(ui, |ui| {
                                for cat in CATEGORIES {
                                    ui.selectable_value(&mut app.event.category, cat.into(), cat);
                                }
                            });
                    },
                );
            });
        } else {
            let (_, cw2) = form_columns(form_w, 2, 160.0, gap_x);
            ui.horizontal_top(|ui| {
                ui.spacing_mut().item_spacing.x = gap_x;
                labeled_input(ui, "Название / предмет", &mut app.event.name, cw2, theme);
                labeled_input(ui, "Кабинет или место", &mut app.event.place, cw2, theme);
            });
            ui.add_space(8.0);
            ui.allocate_ui_with_layout(
                Vec2::new(cw2, 0.0),
                egui::Layout::top_down(egui::Align::Min),
                |ui| {
                    ui.spacing_mut().item_spacing.y = 4.0;
                    ui.label(RichText::new("Категория").size(12.5).color(theme.ink_secondary));
                    egui::ComboBox::from_id_salt("ev_cat_combo")
                        .width(cw2)
                        .selected_text(app.event.effective_category())
                        .show_ui(ui, |ui| {
                            for cat in CATEGORIES {
                                ui.selectable_value(&mut app.event.category, cat.into(), cat);
                            }
                        });
                },
            );
        }

        ui.add_space(10.0);
        let (cols2, cw2) = form_columns(form_w, 4, 125.0, gap_x);
        if cols2 >= 4 {
            ui.horizontal_top(|ui| {
                ui.spacing_mut().item_spacing.x = gap_x;
                ui.allocate_ui_with_layout(
                    Vec2::new(cw2, 0.0),
                    egui::Layout::top_down(egui::Align::Min),
                    |ui| {
                        ui.spacing_mut().item_spacing.y = 4.0;
                        ui.label(RichText::new("День недели").size(12.5).color(theme.ink_secondary));
                        egui::ComboBox::from_id_salt("ev_day_combo")
                            .width(cw2)
                            .selected_text(DAYS[app.event.day.min(6)])
                            .show_ui(ui, |ui| {
                                for (i, n) in DAYS.iter().enumerate() {
                                    ui.selectable_value(&mut app.event.day, i, *n);
                                }
                            });
                    },
                );
                labeled_input(ui, "Начало (ЧЧ:ММ)", &mut app.event.start, cw2, theme);
                labeled_input(ui, "Конец (ЧЧ:ММ)", &mut app.event.end, cw2, theme);
                ui.allocate_ui_with_layout(
                    Vec2::new(cw2, 0.0),
                    egui::Layout::top_down(egui::Align::Min),
                    |ui| {
                        ui.spacing_mut().item_spacing.y = 4.0;
                        ui.label(RichText::new("Дорога, мин").size(12.5).color(theme.ink_secondary));
                        ui.add_sized([cw2, 32.0], egui::DragValue::new(&mut app.event.travel).range(0..=240));
                    },
                );
            });
        } else {
            let (_, hw) = form_columns(form_w, 2, 140.0, gap_x);
            ui.horizontal_top(|ui| {
                ui.spacing_mut().item_spacing.x = gap_x;
                ui.allocate_ui_with_layout(
                    Vec2::new(hw, 0.0),
                    egui::Layout::top_down(egui::Align::Min),
                    |ui| {
                        ui.spacing_mut().item_spacing.y = 4.0;
                        ui.label(RichText::new("День недели").size(12.5).color(theme.ink_secondary));
                        egui::ComboBox::from_id_salt("ev_day_combo")
                            .width(hw)
                            .selected_text(DAYS[app.event.day.min(6)])
                            .show_ui(ui, |ui| {
                                for (i, n) in DAYS.iter().enumerate() {
                                    ui.selectable_value(&mut app.event.day, i, *n);
                                }
                            });
                    },
                );
                ui.allocate_ui_with_layout(
                    Vec2::new(hw, 0.0),
                    egui::Layout::top_down(egui::Align::Min),
                    |ui| {
                        ui.spacing_mut().item_spacing.y = 4.0;
                        ui.label(RichText::new("Дорога, мин").size(12.5).color(theme.ink_secondary));
                        ui.add_sized([hw, 32.0], egui::DragValue::new(&mut app.event.travel).range(0..=240));
                    },
                );
            });
            ui.add_space(8.0);
            ui.horizontal_top(|ui| {
                ui.spacing_mut().item_spacing.x = gap_x;
                labeled_input(ui, "Начало (ЧЧ:ММ)", &mut app.event.start, hw, theme);
                labeled_input(ui, "Конец (ЧЧ:ММ)", &mut app.event.end, hw, theme);
            });
        }

        ui.add_space(10.0);
        ui.collapsing("Отдельная дата и исключения вручную", |ui| {
            let (_, ecw) = form_columns(ui.available_width(), 2, 180.0, gap_x);
            ui.horizontal_top(|ui| {
                ui.spacing_mut().item_spacing.x = gap_x;
                labeled_input(
                    ui,
                    "Только дата (ГГГГ-ММ-ДД)",
                    &mut app.event.on_date,
                    ecw,
                    theme,
                );
                labeled_input(
                    ui,
                    "Пропустить даты (через запятую)",
                    &mut app.event.skip_dates,
                    ecw,
                    theme,
                );
            });
        });

        ui.add_space(12.0);

        let is_valid = {
            let e = &app.event;
            !e.name.trim().is_empty()
                && time(&e.start).zip(time(&e.end)).is_some_and(|(a, b)| a < b)
                && (e.on_date.trim().is_empty() || date(&e.on_date).is_some())
                && e.skip_dates
                    .split(',')
                    .all(|s| s.trim().is_empty() || date(s.trim()).is_some())
        };

        let is_editing_series = app
            .edit_event
            .is_some_and(|idx| app.d.events[idx].on_date.trim().is_empty());

        ui.horizontal_wrapped(|ui| {
            let save_label = if is_editing_series {
                "Сохранить всю серию"
            } else {
                "Сохранить занятие"
            };
            if glass_button(
                ui,
                &mut app.motion,
                egui::Id::new("ev_save_btn"),
                save_label,
                Some(Icon::Check),
                true,
                theme,
            )
            .clicked()
            {
                if is_valid {
                    app.push_undo();
                    if let Some(i) = app.edit_event.take() {
                        app.d.events[i] = app.event.clone();
                    } else {
                        app.d.events.push(app.event.clone());
                    }
                    app.event = Event {
                        day: today_d.weekday().num_days_from_monday() as usize,
                        ..Default::default()
                    };
                    app.show_event_form = false;
                    app.notify("Занятие сохранено");
                } else {
                    app.notify_error(
                        "Проверь название, даты и время ЧЧ:ММ (конец должен быть позже начала).",
                    );
                }
            }

            if is_editing_series {
                ui.add_space(8.0);
                ui.add_sized(
                    [115.0, 32.0],
                    egui::TextEdit::singleline(&mut app.override_date_input)
                        .hint_text("ГГГГ-ММ-ДД"),
                );
                if glass_button(
                    ui,
                    &mut app.motion,
                    egui::Id::new("ev_override_date_btn"),
                    "Только эту дату",
                    None,
                    false,
                    theme,
                )
                .clicked()
                {
                    if is_valid {
                        if let Some(od) = date(&app.override_date_input) {
                            if let Some(i) = app.edit_event.take() {
                                app.push_undo();
                                app.d.events[i].add_skip_date(od);
                                let mut single = app.event.clone();
                                single.on_date = od.to_string();
                                single.skip_dates.clear();
                                app.d.events.push(single);
                                app.show_event_form = false;
                                app.notify(format!("Изменено только для {}", od));
                            }
                        } else {
                            app.notify_error("Укажи дату в формате ГГГГ-ММ-ДД");
                        }
                    } else {
                        app.notify_error("Проверь название и время занятия.");
                    }
                }
            }

            if let Some(i) = app.edit_event {
                if glass_button(
                    ui,
                    &mut app.motion,
                    egui::Id::new("ev_delete_btn"),
                    "Удалить",
                    Some(Icon::Trash),
                    false,
                    theme,
                )
                .clicked()
                {
                    app.push_undo();
                    app.d.events.remove(i);
                    app.edit_event = None;
                    app.show_event_form = false;
                    app.notify("Занятие удалено");
                }
            }
        });
    });
}
