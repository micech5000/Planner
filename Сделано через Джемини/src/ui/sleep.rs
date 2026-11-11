use crate::glass::{
    Icon, Theme, glass_button, glass_panel, icon_button, segmented_control, semibold,
};
use crate::model::{Night, date, plan, ru_date_day_month, time, today};
use crate::ui::{PlannerApp, form_columns, labeled_input};
use chrono::{Duration, NaiveDate};
use eframe::egui::{self, CornerRadius, FontId, Pos2, Rect, RichText, Sense, Stroke, Vec2};

pub fn show(app: &mut PlannerApp, ui: &mut egui::Ui, theme: &Theme) {
    let avail = ui.available_width();
    if avail >= 640.0 {
        let ctrl_est_w = 224.0;
        let left_w = (avail - ctrl_est_w - 14.0).max(200.0);
        ui.horizontal_top(|ui| {
            let left_resp = ui.allocate_ui_with_layout(
                Vec2::new(left_w, 0.0),
                egui::Layout::top_down(egui::Align::Min),
                |ui| {
                    ui.label(semibold("Сон", 28.0, theme.ink));
                    ui.add(
                        egui::Label::new(
                            RichText::new("Обратный план на следующую ночь и личный дневник наблюдений")
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
                    segmented_control(
                        ui,
                        &mut app.motion,
                        egui::Id::new("sleep_subtab"),
                        &mut app.d.sleep_tab,
                        &["План и обзор", "Дневник"],
                        theme,
                    );
                },
            );
        });
    } else {
        ui.label(semibold("Сон", 28.0, theme.ink));
        ui.add(
            egui::Label::new(
                RichText::new("Обратный план на следующую ночь и личный дневник наблюдений")
                    .size(13.5)
                    .color(theme.ink_secondary),
            )
            .wrap(),
        );
        ui.add_space(8.0);
        segmented_control(
            ui,
            &mut app.motion,
            egui::Id::new("sleep_subtab"),
            &mut app.d.sleep_tab,
            &["План и обзор", "Дневник"],
            theme,
        );
    }

    ui.add_space(12.0);

    if app.d.sleep_tab == 0 {
        show_overview(app, ui, theme);
    } else {
        show_diary(app, ui, theme);
    }
}

fn show_overview(app: &mut PlannerApp, ui: &mut egui::Ui, theme: &Theme) {
    let today_d = today();
    let tomorrow = today_d + Duration::days(1);
    let p = plan(&app.d, tomorrow);

    // 1. Planned sleep chain
    let w = ui.available_width();
    glass_panel(ui, theme, 16, true, 16, |ui| {
        ui.set_min_width((w - 32.0).max(200.0));
        ui.horizontal_wrapped(|ui| {
            ui.label(semibold(
                format!(
                    "План на ночь с {} на {}",
                    ru_date_day_month(today_d),
                    ru_date_day_month(tomorrow)
                ),
                16.0,
                theme.ink,
            ));
            let anchor = match (&p.first_event_name, p.first_event_time) {
                (Some(n), Some(t)) => format!("· от занятия «{n}» в {}", t.format("%H:%M")),
                _ => "· свободный день (ручной подъём)".to_owned(),
            };
            ui.label(RichText::new(anchor).size(13.0).color(theme.ink_secondary));
        });

        ui.add_space(10.0);

        let mut steps: Vec<(&str, String, String, bool)> = vec![
            (
                "Завершить дела",
                p.finish.format("%H:%M").to_string(),
                p.finish.format("%d.%m").to_string(),
                true,
            ),
            (
                "Лечь",
                p.bed.format("%H:%M").to_string(),
                p.bed.format("%d.%m").to_string(),
                false,
            ),
            (
                "Уснуть",
                p.asleep.format("%H:%M").to_string(),
                p.asleep.format("%d.%m").to_string(),
                false,
            ),
            (
                "Подъём",
                p.wake.format("%H:%M").to_string(),
                p.wake.format("%d.%m").to_string(),
                true,
            ),
        ];
        if let Some(leave) = p.leave {
            steps.push((
                "Выход",
                leave.format("%H:%M").to_string(),
                leave.format("%d.%m").to_string(),
                false,
            ));
        }

        ui.horizontal_wrapped(|ui| {
            ui.spacing_mut().item_spacing = Vec2::new(22.0, 10.0);
            for (title, clock, date_str, highlight) in steps {
                ui.vertical(|ui| {
                    ui.set_min_width(100.0);
                    ui.spacing_mut().item_spacing.y = 2.0;
                    ui.label(RichText::new(title).size(12.5).color(theme.ink_secondary));
                    ui.label(semibold(
                        clock,
                        22.0,
                        if highlight { theme.accent } else { theme.ink },
                    ));
                    ui.label(RichText::new(date_str).size(12.0).color(theme.ink_muted));
                });
            }
        });

        ui.add_space(8.0);
        ui.collapsing("Настроить расчёт сна и дороги", |ui| {
            ui.add_space(6.0);
            let form_w = ui.available_width();
            let gap_x = 12.0;
            let (cols, cw) = form_columns(form_w, 3, 165.0, gap_x);

            enum ParamKind {
                SleepHours,
                Falling,
                Morning,
                Evening,
                Buffer,
                FreeWake,
            }
            let items = [
                ParamKind::SleepHours,
                ParamKind::Falling,
                ParamKind::Morning,
                ParamKind::Evening,
                ParamKind::Buffer,
                ParamKind::FreeWake,
            ];
            for row in items.chunks(cols) {
                ui.horizontal_top(|ui| {
                    ui.spacing_mut().item_spacing.x = gap_x;
                    for kind in row {
                        match kind {
                            ParamKind::SleepHours => {
                                ui.allocate_ui_with_layout(
                                    Vec2::new(cw, 0.0),
                                    egui::Layout::top_down(egui::Align::Min),
                                    |ui| {
                                        ui.spacing_mut().item_spacing.y = 4.0;
                                        ui.label(
                                            RichText::new("Целевой сон, ч")
                                                .size(12.5)
                                                .color(theme.ink_secondary),
                                        );
                                        ui.add_sized(
                                            [cw, 32.0],
                                            egui::DragValue::new(&mut app.d.sleep_hours)
                                                .range(1.0..=16.0)
                                                .speed(0.25),
                                        );
                                    },
                                );
                            }
                            ParamKind::Falling => {
                                ui.allocate_ui_with_layout(
                                    Vec2::new(cw, 0.0),
                                    egui::Layout::top_down(egui::Align::Min),
                                    |ui| {
                                        ui.spacing_mut().item_spacing.y = 4.0;
                                        ui.label(
                                            RichText::new("Засыпание, мин")
                                                .size(12.5)
                                                .color(theme.ink_secondary),
                                        );
                                        ui.add_sized(
                                            [cw, 32.0],
                                            egui::DragValue::new(&mut app.d.falling).range(0..=180),
                                        );
                                    },
                                );
                            }
                            ParamKind::Morning => {
                                ui.allocate_ui_with_layout(
                                    Vec2::new(cw, 0.0),
                                    egui::Layout::top_down(egui::Align::Min),
                                    |ui| {
                                        ui.spacing_mut().item_spacing.y = 4.0;
                                        ui.label(
                                            RichText::new("Утренние сборы, мин")
                                                .size(12.5)
                                                .color(theme.ink_secondary),
                                        );
                                        ui.add_sized(
                                            [cw, 32.0],
                                            egui::DragValue::new(&mut app.d.morning).range(0..=240),
                                        );
                                    },
                                );
                            }
                            ParamKind::Evening => {
                                ui.allocate_ui_with_layout(
                                    Vec2::new(cw, 0.0),
                                    egui::Layout::top_down(egui::Align::Min),
                                    |ui| {
                                        ui.spacing_mut().item_spacing.y = 4.0;
                                        ui.label(
                                            RichText::new("Вечерняя подготовка, мин")
                                                .size(12.5)
                                                .color(theme.ink_secondary),
                                        );
                                        ui.add_sized(
                                            [cw, 32.0],
                                            egui::DragValue::new(&mut app.d.evening).range(0..=240),
                                        );
                                    },
                                );
                            }
                            ParamKind::Buffer => {
                                ui.allocate_ui_with_layout(
                                    Vec2::new(cw, 0.0),
                                    egui::Layout::top_down(egui::Align::Min),
                                    |ui| {
                                        ui.spacing_mut().item_spacing.y = 4.0;
                                        ui.label(
                                            RichText::new("Запас на дорогу, мин")
                                                .size(12.5)
                                                .color(theme.ink_secondary),
                                        );
                                        ui.add_sized(
                                            [cw, 32.0],
                                            egui::DragValue::new(&mut app.d.buffer).range(0..=120),
                                        );
                                    },
                                );
                            }
                            ParamKind::FreeWake => {
                                labeled_input(
                                    ui,
                                    "Подъём в свободный день",
                                    &mut app.d.free_wake,
                                    cw,
                                    theme,
                                );
                            }
                        }
                    }
                });
                ui.add_space(8.0);
            }
            if time(&app.d.free_wake).is_none() {
                ui.label(
                    RichText::new("Некорректное время ЧЧ:ММ; временно используется 08:00")
                        .size(12.5)
                        .color(theme.danger),
                );
            }
        });
    });

    ui.add_space(10.0);

    // 2. Observed sleep chart (strictly separated from planned sleep!)
    ui.horizontal(|ui| {
        ui.label(semibold("Фактический сон", 17.0, theme.ink));
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if glass_button(
                ui,
                &mut app.motion,
                egui::Id::new("sleep_open_diary_btn"),
                "Записать сон",
                Some(Icon::Plus),
                false,
                theme,
            )
            .clicked()
            {
                app.d.sleep_tab = 1;
            }
        });
    });
    ui.add_space(4.0);

    let mut values: Vec<(NaiveDate, f32, &Night)> = app
        .d
        .nights
        .iter()
        .filter_map(|n| Some((date(&n.date)?, n.hours.trim().parse::<f32>().ok()?, n)))
        .collect();
    values.sort_by_key(|(d, _, _)| *d);

    if values.is_empty() {
        glass_panel(ui, theme, 14, false, 16, |ui| {
            ui.set_min_width((w - 32.0).max(200.0));
            ui.add(
                egui::Label::new(
                    RichText::new(
                        "Записей фактического сна пока нет. Пропущенные дни не считаются за ноль — добавь первую ночь в разделе «Дневник».",
                    )
                    .size(14.0)
                    .color(theme.ink_secondary),
                )
                .wrap(),
            );
        });
    } else {
        let avg = values.iter().map(|v| v.1).sum::<f32>() / values.len() as f32;
        ui.add(
            egui::Label::new(
                RichText::new(format!(
                    "Средняя длительность: {avg:.1} ч · учтено записей: {} (дни без записей пропущены)",
                    values.len()
                ))
                .size(13.0)
                .color(theme.ink_secondary),
            )
            .wrap(),
        );
        ui.add_space(6.0);

        glass_panel(ui, theme, 16, false, 12, |ui| {
            ui.set_min_width((w - 24.0).max(200.0));
            draw_sleep_duration_chart(ui, theme, &values, app.d.sleep_hours);
        });
    }

    ui.add_space(10.0);

    // 3. Separate sections for Phases and Vitals (missing measurements stay omitted, no fabricated scores)
    let rem_avg = avg_field(&app.d.nights, |n| &n.rem);
    let core_avg = avg_field(&app.d.nights, |n| &n.core);
    let deep_avg = avg_field(&app.d.nights, |n| &n.deep);
    let awake_avg = avg_field(&app.d.nights, |n| &n.awake);

    let pulse_avg = avg_field(&app.d.nights, |n| &n.pulse);
    let breath_avg = avg_field(&app.d.nights, |n| &n.breathing);
    let awaken_avg = avg_field(&app.d.nights, |n| &n.awakenings);

    let avail = ui.available_width();
    if avail >= 580.0 {
        let gap = 16.0;
        let half = (avail - gap) * 0.5;
        let mut left_inner_h = 0.0_f32;
        ui.horizontal_top(|ui| {
            ui.allocate_ui_with_layout(
                Vec2::new(half, 0.0),
                egui::Layout::top_down(egui::Align::Min),
                |ui| {
                    let r = show_metric_block(
                        ui,
                        theme,
                        "Фазы сна (среднее по записям)",
                        &[
                            ("Быстрый (REM)", fmt_metric(rem_avg, "мин")),
                            ("Базовый", fmt_metric(core_avg, "мин")),
                            ("Глубокий", fmt_metric(deep_avg, "мин")),
                            ("Бодрствование", fmt_metric(awake_avg, "мин")),
                        ],
                        0.0,
                    );
                    left_inner_h = (r.response.rect.height() - 24.0).max(0.0);
                },
            );
            ui.add_space(gap);
            ui.allocate_ui_with_layout(
                Vec2::new(half, 0.0),
                egui::Layout::top_down(egui::Align::Min),
                |ui| {
                    show_metric_block(
                        ui,
                        theme,
                        "Показатели организма (по записям)",
                        &[
                            ("Средний пульс", fmt_metric(pulse_avg, "уд/мин")),
                            ("Дыхание", fmt_metric(breath_avg, "/мин")),
                            ("Пробуждения", fmt_metric(awaken_avg, "раз")),
                        ],
                        left_inner_h,
                    );
                },
            );
        });
    } else {
        show_metric_block(
            ui,
            theme,
            "Фазы сна (среднее по записям)",
            &[
                ("Быстрый (REM)", fmt_metric(rem_avg, "мин")),
                ("Базовый", fmt_metric(core_avg, "мин")),
                ("Глубокий", fmt_metric(deep_avg, "мин")),
                ("Бодрствование", fmt_metric(awake_avg, "мин")),
            ],
            0.0,
        );
        ui.add_space(12.0);
        show_metric_block(
            ui,
            theme,
            "Показатели организма (по записям)",
            &[
                ("Средний пульс", fmt_metric(pulse_avg, "уд/мин")),
                ("Дыхание", fmt_metric(breath_avg, "/мин")),
                ("Пробуждения", fmt_metric(awaken_avg, "раз")),
            ],
            0.0,
        );
    }
}

fn draw_sleep_duration_chart(
    ui: &mut egui::Ui,
    theme: &Theme,
    values: &[(NaiveDate, f32, &Night)],
    goal_hours: f32,
) {
    let first_d = values.first().expect("non-empty").0;
    let end_d = values.last().expect("non-empty").0;
    let start_d = first_d.max(end_d - Duration::days(13));
    let span_days = ((end_d - start_d).num_days() + 1).max(1) as usize;

    let max_val = values
        .iter()
        .map(|(_, h, _)| *h)
        .fold(0.0_f32, f32::max)
        .max(goal_hours);
    let max_y = if max_val <= 10.0 {
        10.0_f32
    } else if max_val <= 12.0 {
        12.0_f32
    } else {
        (max_val + 1.0).ceil().min(16.0)
    };

    let avail_w = ui.available_width().max(240.0);
    let chart_h = 114.0;
    let (rect, _) = ui.allocate_exact_size(Vec2::new(avail_w, chart_h), Sense::hover());

    let y_gutter = 34.0;
    let top_pad = 16.0;
    let bottom_pad = 22.0;
    let right_pad = 8.0;
    let plot_rect = Rect::from_min_max(
        Pos2::new(rect.min.x + y_gutter, rect.min.y + top_pad),
        Pos2::new(rect.max.x - right_pad, rect.max.y - bottom_pad),
    );
    if plot_rect.width() <= 20.0 || plot_rect.height() <= 20.0 {
        return;
    }

    let p = ui.painter();

    // Horizontal reference grid lines (0, 4, 8, max_y)
    let ticks = if max_y >= 12.0 {
        vec![0.0_f32, 4.0, 8.0, 12.0]
    } else {
        vec![0.0_f32, 4.0, 8.0, 10.0]
    };
    for tick in ticks {
        if tick > max_y + 0.1 {
            continue;
        }
        let y = plot_rect.max.y - (tick / max_y) * plot_rect.height();
        p.line_segment(
            [Pos2::new(plot_rect.min.x, y), Pos2::new(plot_rect.max.x, y)],
            Stroke::new(1.0_f32, theme.divider),
        );
        let lbl = format!("{tick:.0} ч");
        let g = p.layout_no_wrap(lbl, FontId::proportional(11.0), theme.ink_muted);
        p.galley(
            Pos2::new(plot_rect.min.x - g.size().x - 6.0, y - g.size().y * 0.5),
            g,
            theme.ink_muted,
        );
    }

    // Subtle dashed goal reference line
    if goal_hours > 0.5 && goal_hours <= max_y {
        let gy = plot_rect.max.y - (goal_hours / max_y) * plot_rect.height();
        let dash_col = theme.accent.gamma_multiply(0.45);
        let mut x = plot_rect.min.x;
        while x < plot_rect.max.x {
            let x2 = (x + 5.0).min(plot_rect.max.x);
            p.line_segment(
                [Pos2::new(x, gy), Pos2::new(x2, gy)],
                Stroke::new(1.1_f32, dash_col),
            );
            x += 9.0;
        }
        let goal_lbl = format!("Цель {goal_hours:.0} ч");
        let g = p.layout_no_wrap(goal_lbl, FontId::proportional(10.5), theme.accent.gamma_multiply(0.85));
        p.galley(
            Pos2::new(plot_rect.max.x - g.size().x, gy - g.size().y - 2.0),
            g,
            theme.accent,
        );
    }

    let col_w = plot_rect.width() / span_days as f32;
    let bar_w = (col_w * 0.42).clamp(16.0, 42.0);
    let label_step = if col_w < 38.0 { 2 } else { 1 };

    for day_idx in 0..span_days {
        let d = start_d + Duration::days(day_idx as i64);
        let cx = plot_rect.min.x + (day_idx as f32 + 0.5) * col_w;
        let hit_rect = Rect::from_min_max(
            Pos2::new(cx - col_w * 0.48, plot_rect.min.y - 12.0),
            Pos2::new(cx + col_w * 0.48, rect.max.y),
        );
        let resp = ui.interact(
            hit_rect,
            egui::Id::new(("sleep_bar_day", day_idx)),
            Sense::hover(),
        );

        let rec = values.iter().find(|(vd, _, _)| *vd == d);
        if let Some((_, hrs, night)) = rec {
            let clamped = hrs.clamp(0.0, max_y);
            let bar_h = ((clamped / max_y) * plot_rect.height()).max(4.0);
            let bar_rect = Rect::from_min_max(
                Pos2::new(cx - bar_w * 0.5, plot_rect.max.y - bar_h),
                Pos2::new(cx + bar_w * 0.5, plot_rect.max.y),
            );
            let fill_alpha = if resp.hovered() { 0.96 } else { 0.82 };
            p.rect_filled(
                bar_rect,
                CornerRadius {
                    nw: 6,
                    ne: 6,
                    sw: 0,
                    se: 0,
                },
                theme.accent.gamma_multiply(fill_alpha),
            );

            let val_g = p.layout_no_wrap(
                format!("{hrs:.1}"),
                FontId::new(11.5, egui::FontFamily::Name("Semibold".into())),
                if resp.hovered() {
                    theme.ink
                } else {
                    theme.ink_secondary
                },
            );
            p.galley(
                Pos2::new(cx - val_g.size().x * 0.5, bar_rect.min.y - val_g.size().y - 2.0),
                val_g,
                theme.ink_secondary,
            );

            let mut tip = format!("{} · Фактический сон: {hrs:.1} ч", d.format("%d.%m.%Y"));
            if !night.bed.trim().is_empty() || !night.wake.trim().is_empty() {
                tip.push_str(&format!("\n{} – {}", night.bed.trim(), night.wake.trim()));
            }
            if !night.mood.trim().is_empty() {
                tip.push_str(&format!("\nСамочувствие: {}", night.mood.trim()));
            }
            resp.on_hover_text(tip);
        } else {
            // Empty slot marker showing missing day is a gap, not zero
            let mark = Rect::from_center_size(
                Pos2::new(cx, plot_rect.max.y - 1.5),
                Vec2::new(10.0, 2.0),
            );
            p.rect_filled(
                mark,
                CornerRadius::same(1),
                theme.ink_muted.gamma_multiply(0.45),
            );
            resp.on_hover_text(format!(
                "{} · Записи нет (пропуск дня, не ноль)",
                d.format("%d.%m.%Y")
            ));
        }

        if day_idx % label_step == 0 || day_idx + 1 == span_days {
            let col = if rec.is_some() {
                theme.ink_secondary
            } else {
                theme.ink_muted
            };
            let dg = p.layout_no_wrap(d.format("%d.%m").to_string(), FontId::proportional(11.5), col);
            p.galley(
                Pos2::new(cx - dg.size().x * 0.5, plot_rect.max.y + 5.0),
                dg,
                col,
            );
        }
    }
}

fn show_metric_block(
    ui: &mut egui::Ui,
    theme: &Theme,
    title: &str,
    rows: &[(&str, String)],
    min_inner_h: f32,
) -> egui::InnerResponse<()> {
    let w = ui.available_width();
    glass_panel(ui, theme, 14, false, 12, |ui| {
        ui.set_min_width((w - 24.0).max(160.0));
        if min_inner_h > 0.0 {
            ui.set_min_height(min_inner_h);
        }
        ui.spacing_mut().item_spacing.y = 4.0;
        ui.label(semibold(title, 14.5, theme.ink));
        ui.add_space(2.0);
        for (name, val) in rows {
            ui.horizontal(|ui| {
                ui.label(RichText::new(*name).size(13.0).color(theme.ink_secondary));
                let rem = ui.available_width().max(60.0);
                ui.allocate_ui_with_layout(
                    Vec2::new(rem, 18.0),
                    egui::Layout::right_to_left(egui::Align::Center),
                    |ui| {
                        ui.label(RichText::new(val).size(13.0).color(theme.ink));
                    },
                );
            });
        }
    })
}

fn avg_field(nights: &[Night], getter: impl Fn(&Night) -> &str) -> Option<(f32, usize)> {
    let vals: Vec<f32> = nights
        .iter()
        .filter_map(|n| getter(n).trim().parse::<f32>().ok())
        .filter(|v| v.is_finite())
        .collect();
    if vals.is_empty() {
        None
    } else {
        Some((vals.iter().sum::<f32>() / vals.len() as f32, vals.len()))
    }
}

fn fmt_metric(avg: Option<(f32, usize)>, unit: &str) -> String {
    match avg {
        Some((v, count)) => format!("{v:.1} {unit} (зап.: {count})"),
        None => "—".to_owned(),
    }
}

fn show_diary(app: &mut PlannerApp, ui: &mut egui::Ui, theme: &Theme) {
    let w = ui.available_width();
    glass_panel(ui, theme, 16, false, 16, |ui| {
        ui.set_min_width((w - 32.0).max(200.0));
        ui.label(semibold(
            if app.edit_night.is_some() {
                "Редактировать запись ночи"
            } else {
                "Новая запись сна (по дате пробуждения)"
            },
            15.5,
            theme.ink,
        ));
        ui.label(
            RichText::new("Все медицинские и дополнительные показатели необязательны.")
                .size(12.5)
                .color(theme.ink_secondary),
        );
        ui.add_space(10.0);

        let form_w = ui.available_width();
        let gap_x = 12.0;
        let (cols, col_w) = form_columns(form_w, 4, 155.0, gap_x);

        let mut fields: [(&str, &mut String); 12] = [
            ("Дата (ГГГГ-ММ-ДД)", &mut app.night.date),
            ("Лёг (ЧЧ:ММ)", &mut app.night.bed),
            ("Проснулся (ЧЧ:ММ)", &mut app.night.wake),
            ("Фактический сон, ч", &mut app.night.hours),
            ("Быстрый сон, мин", &mut app.night.rem),
            ("Базовый сон, мин", &mut app.night.core),
            ("Глубокий сон, мин", &mut app.night.deep),
            ("Бодрствование, мин", &mut app.night.awake),
            ("Пробуждения", &mut app.night.awakenings),
            ("Пульс, уд/мин", &mut app.night.pulse),
            ("Дыхание, /мин", &mut app.night.breathing),
            ("Самочувствие", &mut app.night.mood),
        ];

        for row in fields.chunks_mut(cols) {
            ui.horizontal_top(|ui| {
                ui.spacing_mut().item_spacing.x = gap_x;
                for (label, val) in row.iter_mut() {
                    labeled_input(ui, label, val, col_w, theme);
                }
            });
            ui.add_space(8.0);
        }

        ui.add_space(2.0);
        ui.label(
            RichText::new("Сновидения и заметки")
                .size(12.5)
                .color(theme.ink_secondary),
        );
        ui.add_space(4.0);
        ui.add_sized(
            [ui.available_width().max(200.0), 64.0],
            egui::TextEdit::multiline(&mut app.night.notes),
        );
        ui.add_space(10.0);

        ui.horizontal(|ui| {
            if glass_button(
                ui,
                &mut app.motion,
                egui::Id::new("night_save_btn"),
                "Сохранить запись сна",
                Some(Icon::Check),
                true,
                theme,
            )
            .clicked()
            {
                let n = &app.night;
                let numeric = [
                    (&n.hours, 24.0),
                    (&n.rem, 1440.0),
                    (&n.core, 1440.0),
                    (&n.deep, 1440.0),
                    (&n.awake, 1440.0),
                    (&n.awakenings, 1000.0),
                    (&n.pulse, 300.0),
                    (&n.breathing, 100.0),
                ]
                .iter()
                .all(|(s, max)| {
                    s.trim().is_empty()
                        || s.trim()
                            .parse::<f32>()
                            .is_ok_and(|v| v.is_finite() && v >= 0.0 && v <= *max)
                });
                let duplicate = app
                    .d
                    .nights
                    .iter()
                    .enumerate()
                    .any(|(i, v)| v.date == n.date && Some(i) != app.edit_night);
                let times_ok = [&n.bed, &n.wake]
                    .iter()
                    .all(|s| s.trim().is_empty() || time(s).is_some());

                if date(&n.date).is_some() && numeric && !duplicate && times_ok {
                    app.push_undo();
                    if let Some(i) = app.edit_night.take() {
                        app.d.nights[i] = app.night.clone();
                    } else {
                        app.d.nights.push(app.night.clone());
                    }
                    app.night = Night::default();
                    app.notify("Запись сна сохранена");
                } else {
                    app.notify_error(
                        "Проверь дату (ГГГГ-ММ-ДД), время (ЧЧ:ММ) и числа через точку. На одну дату — одна запись.",
                    );
                }
            }

            if app.edit_night.is_some()
                && glass_button(
                    ui,
                    &mut app.motion,
                    egui::Id::new("night_cancel_btn"),
                    "Отмена",
                    None,
                    false,
                    theme,
                )
                .clicked()
            {
                app.edit_night = None;
                app.night = Night::default();
            }
        });
    });

    ui.add_space(18.0);
    ui.label(semibold("Записи дневника", 16.5, theme.ink));
    ui.add_space(8.0);

    if app.d.nights.is_empty() {
        ui.label(
            RichText::new("Записей пока нет.")
                .size(14.0)
                .color(theme.ink_secondary),
        );
        return;
    }

    let mut order: Vec<usize> = (0..app.d.nights.len()).collect();
    order.sort_by(|&a, &b| app.d.nights[b].date.cmp(&app.d.nights[a].date));

    let mut edit = None;
    let mut remove = None;

    for idx in order {
        let n = &app.d.nights[idx];
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
            let text_w = (ui.available_width() - 88.0).max(120.0);
            let left_resp = ui.allocate_ui_with_layout(
                Vec2::new(text_w, 0.0),
                egui::Layout::top_down(egui::Align::Min),
                |ui| {
                    let hours_txt = if n.hours.trim().is_empty() {
                        "—".to_owned()
                    } else {
                        format!("{} ч", n.hours.trim())
                    };
                    let pulse_txt = if n.pulse.trim().is_empty() {
                        "—"
                    } else {
                        n.pulse.trim()
                    };
                    let breath_txt = if n.breathing.trim().is_empty() {
                        "—"
                    } else {
                        n.breathing.trim()
                    };
                    ui.add(
                        egui::Label::new(semibold(
                            format!("{} · сон {hours_txt}", n.date),
                            14.5,
                            theme.ink,
                        ))
                        .wrap(),
                    );
                    let mut sub = format!("Пульс {pulse_txt} · Дыхание {breath_txt}");
                    if !n.mood.trim().is_empty() {
                        sub.push_str(&format!(" · {}", n.mood.trim()));
                    }
                    ui.add(
                        egui::Label::new(
                            RichText::new(sub).size(12.5).color(theme.ink_secondary),
                        )
                        .wrap(),
                    );
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
                        egui::Id::new(("night_del", idx)),
                        Icon::Trash,
                        false,
                        "Удалить запись",
                        theme,
                    )
                    .clicked()
                    {
                        remove = Some(idx);
                    }
                    if icon_button(
                        ui,
                        &mut app.motion,
                        egui::Id::new(("night_edit", idx)),
                        Icon::Edit,
                        app.edit_night == Some(idx),
                        "Открыть запись",
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
        app.night = app.d.nights[i].clone();
        app.edit_night = Some(i);
    }
    if let Some(i) = remove {
        app.push_undo();
        app.d.nights.remove(i);
        app.edit_night = None;
        app.notify("Запись сна удалена");
    }
}
