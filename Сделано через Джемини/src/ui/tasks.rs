use crate::glass::{
    Icon, Theme, draw_icon, glass_button, icon_button, reminders_checkbox, semibold,
};
use crate::model::{Task, date, format_duration_ru, today};
use crate::ui::{PlannerApp, form_columns, labeled_input};
use eframe::egui::{self, Pos2, RichText, Sense, Stroke, Vec2};

pub fn show(app: &mut PlannerApp, ui: &mut egui::Ui, theme: &Theme) {
    ui.horizontal(|ui| {
        ui.vertical(|ui| {
            ui.label(semibold("Задачи", 28.0, theme.ink));
            let active_n = app.d.tasks.iter().filter(|t| !t.done).count();
            let active_m: i64 = app
                .d
                .tasks
                .iter()
                .filter(|t| !t.done)
                .map(|t| t.minutes.max(0))
                .sum();
            let sub = if active_n == 0 {
                "Активных задач нет".to_owned()
            } else {
                format!(
                    "Активных: {active_n} · {}",
                    format_duration_ru(active_m)
                )
            };
            ui.label(RichText::new(sub).size(13.5).color(theme.ink_secondary));
        });
    });

    ui.add_space(16.0);

    // Add / Edit task bar
    let w = ui.available_width();
    crate::glass::glass_panel(ui, theme, 16, false, 16, |ui| {
        ui.set_min_width((w - 32.0).max(200.0));
        ui.label(semibold(
            if app.edit_task.is_some() {
                "Изменить задачу"
            } else {
                "Новая задача"
            },
            15.0,
            theme.ink,
        ));
        ui.add_space(8.0);
        let form_w = ui.available_width();
        let gap_x = 12.0;
        if form_w >= 620.0 {
            let usable = form_w - gap_x * 3.0;
            let w_title = (usable * 0.38).max(170.0);
            let w_subj = (usable * 0.24).max(115.0);
            let w_due = (usable * 0.23).max(115.0);
            let w_mins = (usable - w_title - w_subj - w_due).max(84.0);
            ui.horizontal_top(|ui| {
                ui.spacing_mut().item_spacing.x = gap_x;
                labeled_input(ui, "Что сделать", &mut app.task.title, w_title, theme);
                labeled_input(ui, "Предмет", &mut app.task.subject, w_subj, theme);
                labeled_input(ui, "Срок (ГГГГ-ММ-ДД)", &mut app.task.due, w_due, theme);
                ui.allocate_ui_with_layout(
                    Vec2::new(w_mins, 0.0),
                    egui::Layout::top_down(egui::Align::Min),
                    |ui| {
                        ui.spacing_mut().item_spacing.y = 4.0;
                        ui.label(RichText::new("Время, мин").size(12.5).color(theme.ink_secondary));
                        ui.add_sized([w_mins, 32.0], egui::DragValue::new(&mut app.task.minutes).range(1..=1440));
                    },
                );
            });
        } else {
            let (_, cw) = form_columns(form_w, 2, 160.0, gap_x);
            ui.horizontal_top(|ui| {
                ui.spacing_mut().item_spacing.x = gap_x;
                labeled_input(ui, "Что сделать", &mut app.task.title, cw, theme);
                labeled_input(ui, "Предмет", &mut app.task.subject, cw, theme);
            });
            ui.add_space(8.0);
            ui.horizontal_top(|ui| {
                ui.spacing_mut().item_spacing.x = gap_x;
                labeled_input(ui, "Срок (ГГГГ-ММ-ДД)", &mut app.task.due, cw, theme);
                ui.allocate_ui_with_layout(
                    Vec2::new(cw, 0.0),
                    egui::Layout::top_down(egui::Align::Min),
                    |ui| {
                        ui.spacing_mut().item_spacing.y = 4.0;
                        ui.label(RichText::new("Время, мин").size(12.5).color(theme.ink_secondary));
                        ui.add_sized([cw, 32.0], egui::DragValue::new(&mut app.task.minutes).range(1..=1440));
                    },
                );
            });
        }
        ui.add_space(10.0);
        ui.horizontal(|ui| {
            if glass_button(
                ui,
                &mut app.motion,
                egui::Id::new("task_save_btn"),
                if app.edit_task.is_some() {
                    "Сохранить изменения"
                } else {
                    "Добавить задачу"
                },
                Some(Icon::Plus),
                true,
                theme,
            )
            .clicked()
            {
                if !app.task.title.trim().is_empty()
                    && (app.task.due.trim().is_empty() || date(&app.task.due).is_some())
                {
                    app.push_undo();
                    if let Some(i) = app.edit_task.take() {
                        app.d.tasks[i] = app.task.clone();
                    } else {
                        app.d.tasks.push(app.task.clone());
                    }
                    app.task = Task {
                        due: today().to_string(),
                        minutes: 30,
                        ..Default::default()
                    };
                    app.notify("Задача сохранена");
                } else {
                    app.notify_error("Укажи название и корректную дату (ГГГГ-ММ-ДД).");
                }
            }

            if app.edit_task.is_some()
                && glass_button(
                    ui,
                    &mut app.motion,
                    egui::Id::new("task_cancel_edit"),
                    "Отмена",
                    None,
                    false,
                    theme,
                )
                .clicked()
            {
                app.edit_task = None;
                app.task = Task {
                    due: today().to_string(),
                    minutes: 30,
                    ..Default::default()
                };
            }
        });
    });

    ui.add_space(18.0);

    let active_indices: Vec<usize> = app
        .d
        .tasks
        .iter()
        .enumerate()
        .filter(|(_, t)| !t.done)
        .map(|(i, _)| i)
        .collect();
    let done_indices: Vec<usize> = app
        .d
        .tasks
        .iter()
        .enumerate()
        .filter(|(_, t)| t.done)
        .map(|(i, _)| i)
        .collect();

    if active_indices.is_empty() && done_indices.is_empty() {
        ui.label(
            RichText::new("Список задач пуст. Добавь первую задачу в форме выше.")
                .size(14.0)
                .color(theme.ink_secondary),
        );
        return;
    }

    if active_indices.is_empty() {
        ui.label(
            RichText::new("Все задачи выполнены.")
                .size(14.5)
                .color(theme.ink_secondary),
        );
    } else {
        render_task_group(app, ui, theme, &active_indices);
    }

    if !done_indices.is_empty() {
        ui.add_space(16.0);
        ui.horizontal(|ui| {
            let text = format!("Выполненные ({})", done_indices.len());
            let font = egui::FontId::new(14.5, egui::FontFamily::Name("Semibold".into()));
            let measure = ui.painter().layout_no_wrap(
                text.clone(),
                font.clone(),
                egui::Color32::PLACEHOLDER,
            );
            let total_w = 20.0 + measure.size().x + 6.0;
            let (rect, resp) = ui.allocate_exact_size(Vec2::new(total_w, 26.0), Sense::click());
            let col = if resp.hovered() {
                theme.ink
            } else {
                theme.ink_secondary
            };
            let galley = ui.painter().layout_no_wrap(text, font, col);
            draw_icon(
                ui.painter(),
                Pos2::new(rect.min.x + 7.0, rect.center().y),
                11.0,
                if app.show_completed_tasks {
                    Icon::ChevronDown
                } else {
                    Icon::ChevronRight
                },
                col,
                1.6,
            );
            ui.painter().galley(
                Pos2::new(rect.min.x + 20.0, rect.center().y - galley.size().y * 0.5),
                galley,
                col,
            );
            if resp.clicked() {
                app.show_completed_tasks = !app.show_completed_tasks;
            }
        });
        if app.show_completed_tasks {
            ui.add_space(6.0);
            render_task_group(app, ui, theme, &done_indices);
        }
    }
}

fn render_task_group(
    app: &mut PlannerApp,
    ui: &mut egui::Ui,
    theme: &Theme,
    indices: &[usize],
) {
    let mut toggled = None;
    let mut edit = None;
    let mut remove = None;

    for &idx in indices {
        let line_y = ui.cursor().min.y;
        ui.painter().line_segment(
            [
                Pos2::new(ui.cursor().min.x, line_y),
                Pos2::new(ui.cursor().min.x + ui.available_width(), line_y),
            ],
            Stroke::new(1.0_f32, theme.divider),
        );
        ui.add_space(9.0);

        let t = app.d.tasks[idx].clone();
        let mut checked = t.done;

        ui.horizontal_top(|ui| {
            if reminders_checkbox(
                ui,
                &mut app.motion,
                egui::Id::new(("tasks_cb", idx)),
                &mut checked,
                theme,
            ) {
                toggled = Some((idx, checked));
            }

            let text_max_w = (ui.available_width() - 88.0).max(120.0);
            let left_resp = ui.allocate_ui_with_layout(
                Vec2::new(text_max_w, 0.0),
                egui::Layout::top_down(egui::Align::Min),
                |ui| {
                    ui.spacing_mut().item_spacing.y = 2.0;
                    ui.add_space(2.0);
                    let title_rt = if checked {
                        RichText::new(&t.title)
                            .size(14.5)
                            .strikethrough()
                            .color(theme.ink_muted)
                    } else {
                        RichText::new(&t.title).size(14.5).color(theme.ink)
                    };
                    ui.add(egui::Label::new(title_rt).wrap());

                    let mut meta = Vec::new();
                    if !t.subject.trim().is_empty() {
                        meta.push(t.subject.clone());
                    }
                    meta.push(format_duration_ru(t.minutes));
                    if !t.due.trim().is_empty() {
                        meta.push(format!("до {}", t.due));
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

            let row_h = left_resp.response.rect.height().max(32.0);
            let rem_w = ui.available_width().max(72.0);
            ui.allocate_ui_with_layout(
                Vec2::new(rem_w, row_h),
                egui::Layout::right_to_left(egui::Align::Center),
                |ui| {
                    if icon_button(
                        ui,
                        &mut app.motion,
                        egui::Id::new(("task_del", idx)),
                        Icon::Trash,
                        false,
                        "Удалить задачу",
                        theme,
                    )
                    .clicked()
                    {
                        remove = Some(idx);
                    }
                    if icon_button(
                        ui,
                        &mut app.motion,
                        egui::Id::new(("task_edit", idx)),
                        Icon::Edit,
                        app.edit_task == Some(idx),
                        "Изменить задачу",
                        theme,
                    )
                    .clicked()
                    {
                        edit = Some(idx);
                    }
                },
            );
        });

        ui.add_space(9.0);
    }

    if let Some((idx, val)) = toggled {
        app.push_undo();
        app.d.tasks[idx].done = val;
    }
    if let Some(i) = edit {
        app.task = app.d.tasks[i].clone();
        app.edit_task = Some(i);
    }
    if let Some(i) = remove {
        app.push_undo();
        app.d.tasks.remove(i);
        app.edit_task = None;
        app.notify("Задача удалена");
    }
}
