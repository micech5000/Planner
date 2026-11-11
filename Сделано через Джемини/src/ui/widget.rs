use crate::glass::{
    GlassSurfaceSpec, Icon, Theme, draw_glass_surface, draw_icon, icon_button,
    reminders_checkbox, semibold,
};
use crate::model::{DAYS, date, now_time, plan, ru_date_day_month, time, today};
use crate::ui::PlannerApp;
use chrono::{Datelike, Duration};
use eframe::egui::{self, Pos2, Rect, RichText, Sense, Stroke, Vec2};

const WIDGET_TITLE: &str = "Мой день · виджет";

pub fn show(app: &mut PlannerApp, ctx: &egui::Context, theme: &Theme) {
    let mut visible = true;
    let mut toggle_top = false;
    let mut open_main = false;

    ctx.show_viewport_immediate(
        egui::ViewportId::from_hash_of("desktop_widget"),
        egui::ViewportBuilder::default()
            .with_title(WIDGET_TITLE)
            .with_inner_size([328.0, 392.0])
            .with_min_inner_size([276.0, 280.0])
            .with_decorations(false)
            .with_transparent(theme.glass)
            .with_active(false),
        |wctx, _| {
            wctx.send_viewport_cmd(egui::ViewportCommand::WindowLevel(if app.d.top {
                egui::WindowLevel::AlwaysOnTop
            } else {
                egui::WindowLevel::Normal
            }));

            if wctx.input(|i| i.viewport().close_requested()) {
                visible = false;
            }

            app.dwm
                .sync_widget_by_title(WIDGET_TITLE, theme.dark, theme.glass);

            paint_widget_surface(
                app,
                wctx,
                theme,
                &mut open_main,
                &mut toggle_top,
                &mut visible,
            );
        },
    );

    if toggle_top {
        app.d.top = !app.d.top;
    }
    if open_main {
        ctx.send_viewport_cmd_to(egui::ViewportId::ROOT, egui::ViewportCommand::Visible(true));
        ctx.send_viewport_cmd_to(egui::ViewportId::ROOT, egui::ViewportCommand::Focus);
    }
    if !visible {
        app.d.widget = false;
    }
}

pub fn paint_widget_surface(
    app: &mut PlannerApp,
    wctx: &egui::Context,
    theme: &Theme,
    open_main: &mut bool,
    toggle_top: &mut bool,
    visible: &mut bool,
) {
    egui::CentralPanel::default()
        .frame(egui::Frame::NONE.inner_margin(egui::Margin::same(4)))
        .show(wctx, |ui| {
            let card_rect = ui.available_rect_before_wrap();
            draw_glass_surface(
                ui.painter(),
                card_rect,
                GlassSurfaceSpec {
                    radius: 20,
                    fill: theme.widget_bg,
                    elevated: true,
                    hover_t: 0.0,
                },
                theme,
            );

            let inner = card_rect.shrink2(Vec2::new(18.0, 16.0));
            ui.scope_builder(egui::UiBuilder::new().max_rect(inner), |ui| {
                let today_d = today();
                let now_t = now_time();
                let wd = today_d.weekday().num_days_from_monday() as usize;

                // Draggable header bar
                let header_h = 42.0;
                let (drag_rect, drag_resp) = ui.allocate_exact_size(
                    Vec2::new((inner.width() - 104.0).max(120.0), header_h),
                    Sense::click_and_drag(),
                );
                if drag_resp.drag_started_by(egui::PointerButton::Primary) {
                    wctx.send_viewport_cmd(egui::ViewportCommand::StartDrag);
                }

                // Paint header date in drag zone while buttons sit on right
                let date_g = ui.painter().layout_no_wrap(
                    format!("{}, {}", DAYS[wd], ru_date_day_month(today_d)),
                    egui::FontId::new(16.0, egui::FontFamily::Name("Semibold".into())),
                    theme.ink,
                );
                ui.painter().galley(drag_rect.min, date_g, theme.ink);
                let sub_text = if app.d.has_demo_data {
                    "Сегодня · демо"
                } else {
                    "Сегодня"
                };
                let sub_g = ui.painter().layout_no_wrap(
                    sub_text.into(),
                    egui::FontId::proportional(12.0),
                    theme.ink_secondary,
                );
                ui.painter().galley(
                    Pos2::new(drag_rect.min.x, drag_rect.min.y + 21.0),
                    sub_g,
                    theme.ink_secondary,
                );

                // Top-right widget controls
                let ctrl_rect = Rect::from_min_max(
                    Pos2::new(drag_rect.max.x + 4.0, drag_rect.min.y),
                    Pos2::new(inner.max.x, drag_rect.min.y + 34.0),
                );
                ui.scope_builder(egui::UiBuilder::new().max_rect(ctrl_rect), |ui| {
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.spacing_mut().item_spacing.x = 2.0;
                        if ui
                            .add(
                                egui::Button::new(
                                    RichText::new("×")
                                        .size(16.0)
                                        .color(theme.ink_secondary),
                                )
                                .frame(false),
                            )
                            .on_hover_text("Скрыть виджет")
                            .clicked()
                        {
                            *visible = false;
                        }
                        if icon_button(
                            ui,
                            &mut app.motion,
                            egui::Id::new("w_open_main"),
                            Icon::WindowOpen,
                            false,
                            "Открыть основное окно",
                            theme,
                        )
                        .clicked()
                        {
                            *open_main = true;
                        }
                        if icon_button(
                            ui,
                            &mut app.motion,
                            egui::Id::new("w_pin"),
                            Icon::Pin,
                            app.d.top,
                            "Закрепить поверх окон",
                            theme,
                        )
                        .clicked()
                        {
                            *toggle_top = true;
                        }
                    });
                });

                ui.add_space(6.0);

                egui::ScrollArea::vertical().show(ui, |ui| {
                    // Next upcoming event
                    let mut events: Vec<_> =
                        app.d.events.iter().filter(|e| e.occurs(today_d)).collect();
                    events.sort_by_key(|e| time(&e.start));
                    let next_ev = events
                        .iter()
                        .find(|e| time(&e.end).is_some_and(|et| et > now_t))
                        .copied();

                    if let Some(ev) = next_ev {
                        ui.add(
                            egui::Label::new(semibold(
                                format!("{} · {}", ev.start, ev.name),
                                14.5,
                                theme.accent,
                            ))
                            .wrap(),
                        );
                        let mut sub = format!("До {}", ev.end);
                        if !ev.place.trim().is_empty() {
                            sub.push_str(&format!(" · {}", ev.place.trim()));
                        }
                        ui.add(
                            egui::Label::new(
                                RichText::new(sub).size(12.0).color(theme.ink_secondary),
                            )
                            .wrap(),
                        );
                    } else {
                        ui.add(
                            egui::Label::new(
                                RichText::new("Ближайших занятий сегодня нет")
                                    .size(13.0)
                                    .color(theme.ink_secondary),
                            )
                            .wrap(),
                        );
                    }

                    ui.add_space(10.0);

                    // Up to 4 active/today tasks (incomplete first)
                    let mut task_indices: Vec<usize> = app
                        .d
                        .tasks
                        .iter()
                        .enumerate()
                        .filter(|(_, t)| {
                            t.due.trim().is_empty()
                                || date(&t.due).is_some_and(|d| d <= today_d)
                        })
                        .map(|(i, _)| i)
                        .collect();
                    task_indices.sort_by_key(|&i| app.d.tasks[i].done);
                    task_indices.truncate(4);

                    let mut toggled = None;
                    if task_indices.is_empty() {
                        ui.label(
                            RichText::new("Задач на сегодня нет")
                                .size(13.0)
                                .color(theme.ink_muted),
                        );
                    } else {
                        for idx in task_indices {
                            let line_y = ui.cursor().min.y;
                            ui.painter().line_segment(
                                [
                                    Pos2::new(ui.cursor().min.x, line_y),
                                    Pos2::new(
                                        ui.cursor().min.x + ui.available_width(),
                                        line_y,
                                    ),
                                ],
                                Stroke::new(1.0_f32, theme.divider),
                            );
                            ui.add_space(6.0);
                            let t = &app.d.tasks[idx];
                            let mut checked = t.done;
                            let title = t.title.clone();
                            ui.horizontal_top(|ui| {
                                if reminders_checkbox(
                                    ui,
                                    &mut app.motion,
                                    egui::Id::new(("widget_cb", idx)),
                                    &mut checked,
                                    theme,
                                ) {
                                    toggled = Some((idx, checked));
                                }
                                let text_w = (ui.available_width() - 4.0).max(80.0);
                                ui.allocate_ui_with_layout(
                                    Vec2::new(text_w, 0.0),
                                    egui::Layout::top_down(egui::Align::Min),
                                    |ui| {
                                        ui.add_space(3.0);
                                        let rt = if checked {
                                            RichText::new(title)
                                                .size(13.5)
                                                .strikethrough()
                                                .color(theme.ink_muted)
                                        } else {
                                            RichText::new(title).size(13.5).color(theme.ink)
                                        };
                                        ui.add(egui::Label::new(rt).wrap());
                                    },
                                );
                            });
                            ui.add_space(6.0);
                        }
                    }

                    if let Some((idx, val)) = toggled {
                        app.push_undo();
                        app.d.tasks[idx].done = val;
                    }

                    ui.add_space(10.0);
                    let line_y = ui.cursor().min.y;
                    ui.painter().line_segment(
                        [
                            Pos2::new(ui.cursor().min.x, line_y),
                            Pos2::new(ui.cursor().min.x + ui.available_width(), line_y),
                        ],
                        Stroke::new(1.0_f32, theme.divider),
                    );
                    ui.add_space(10.0);

                    // Evening finish time
                    let p = plan(&app.d, today_d + Duration::days(1));
                    ui.horizontal(|ui| {
                        let (ic_r, _) =
                            ui.allocate_exact_size(Vec2::splat(20.0), Sense::hover());
                        draw_icon(
                            ui.painter(),
                            ic_r.center(),
                            14.0,
                            Icon::Moon,
                            theme.accent,
                            1.6,
                        );
                        ui.vertical(|ui| {
                            ui.spacing_mut().item_spacing.y = 1.0;
                            ui.label(semibold(
                                format!("Закончить до {}", p.finish.format("%H:%M")),
                                13.5,
                                theme.ink,
                            ));
                            ui.label(
                                RichText::new(format!(
                                    "Подъём в {}",
                                    p.wake.format("%H:%M")
                                ))
                                .size(12.0)
                                .color(theme.ink_secondary),
                            );
                        });
                    });
                });
            });
        });
}
