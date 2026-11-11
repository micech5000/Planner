use crate::glass::{Icon, Theme, glass_button, glass_panel, icon_button, semibold};
use crate::model::{Grade, date};
use crate::ui::{PlannerApp, form_columns, labeled_input};
use eframe::egui::{self, Pos2, RichText, Stroke, Vec2};
use std::collections::BTreeMap;

pub fn show(app: &mut PlannerApp, ui: &mut egui::Ui, theme: &Theme) {
    ui.label(semibold("Оценки", 28.0, theme.ink));
    ui.label(
        RichText::new(format!(
            "Предметы, история записей и среднее по шкале до {:.0}",
            app.d.grade_max
        ))
        .size(13.5)
        .color(theme.ink_secondary),
    );
    ui.add_space(16.0);

    // Subject summary strip
    let mut groups: BTreeMap<String, (f32, usize)> = BTreeMap::new();
    for g in &app.d.grades {
        let e = groups.entry(g.subject.clone()).or_default();
        e.0 += g.value;
        e.1 += 1;
    }

    if !groups.is_empty() {
        ui.horizontal_wrapped(|ui| {
            ui.spacing_mut().item_spacing = Vec2::new(12.0, 10.0);
            for (subj, (sum, n)) in &groups {
                let avg = *sum / (*n as f32);
                glass_panel(ui, theme, 12, false, 12, |ui| {
                    ui.set_min_width(140.0);
                    ui.vertical(|ui| {
                        ui.spacing_mut().item_spacing.y = 3.0;
                        ui.label(RichText::new(subj).size(13.0).color(theme.ink_secondary));
                        ui.horizontal(|ui| {
                            ui.label(semibold(format!("{avg:.2}"), 20.0, theme.accent));
                            ui.label(
                                RichText::new(format!("· записей: {n}"))
                                    .size(12.0)
                                    .color(theme.ink_muted),
                            );
                        });
                    });
                });
            }
        });
        ui.add_space(18.0);
    }

    // Add / edit form
    let w = ui.available_width();
    glass_panel(ui, theme, 16, false, 16, |ui| {
        ui.set_min_width((w - 32.0).max(200.0));
        ui.label(semibold(
            if app.edit_grade.is_some() {
                "Изменить оценку"
            } else {
                "Новая запись"
            },
            15.0,
            theme.ink,
        ));
        ui.add_space(8.0);
        let form_w = ui.available_width();
        let gap_x = 12.0;
        if form_w >= 620.0 {
            let usable = form_w - gap_x * 3.0;
            let w_subj = (usable * 0.28).max(130.0);
            let w_date = (usable * 0.24).max(115.0);
            let w_val = (usable * 0.14).max(80.0);
            let w_note = (usable - w_subj - w_date - w_val).max(140.0);
            ui.horizontal_top(|ui| {
                ui.spacing_mut().item_spacing.x = gap_x;
                labeled_input(ui, "Предмет", &mut app.grade.subject, w_subj, theme);
                labeled_input(ui, "Дата (ГГГГ-ММ-ДД)", &mut app.grade.date, w_date, theme);
                ui.allocate_ui_with_layout(
                    Vec2::new(w_val, 0.0),
                    egui::Layout::top_down(egui::Align::Min),
                    |ui| {
                        ui.spacing_mut().item_spacing.y = 4.0;
                        ui.label(RichText::new("Оценка").size(12.5).color(theme.ink_secondary));
                        ui.add_sized(
                            [w_val, 32.0],
                            egui::DragValue::new(&mut app.grade.value)
                                .range(0.0..=app.d.grade_max)
                                .speed(0.1),
                        );
                    },
                );
                labeled_input(ui, "Комментарий", &mut app.grade.note, w_note, theme);
            });
        } else {
            let (_, cw) = form_columns(form_w, 2, 160.0, gap_x);
            ui.horizontal_top(|ui| {
                ui.spacing_mut().item_spacing.x = gap_x;
                labeled_input(ui, "Предмет", &mut app.grade.subject, cw, theme);
                labeled_input(ui, "Дата (ГГГГ-ММ-ДД)", &mut app.grade.date, cw, theme);
            });
            ui.add_space(8.0);
            ui.horizontal_top(|ui| {
                ui.spacing_mut().item_spacing.x = gap_x;
                ui.allocate_ui_with_layout(
                    Vec2::new(cw, 0.0),
                    egui::Layout::top_down(egui::Align::Min),
                    |ui| {
                        ui.spacing_mut().item_spacing.y = 4.0;
                        ui.label(RichText::new("Оценка").size(12.5).color(theme.ink_secondary));
                        ui.add_sized(
                            [cw, 32.0],
                            egui::DragValue::new(&mut app.grade.value)
                                .range(0.0..=app.d.grade_max)
                                .speed(0.1),
                        );
                    },
                );
                labeled_input(ui, "Комментарий", &mut app.grade.note, cw, theme);
            });
        }
        ui.add_space(10.0);
        ui.horizontal(|ui| {
            if glass_button(
                ui,
                &mut app.motion,
                egui::Id::new("grade_save_btn"),
                if app.edit_grade.is_some() {
                    "Сохранить изменения"
                } else {
                    "Добавить оценку"
                },
                Some(Icon::Plus),
                true,
                theme,
            )
            .clicked()
            {
                if !app.grade.subject.trim().is_empty()
                    && date(&app.grade.date).is_some()
                    && app.grade.value <= app.d.grade_max
                {
                    app.push_undo();
                    if let Some(i) = app.edit_grade.take() {
                        app.d.grades[i] = app.grade.clone();
                    } else {
                        app.d.grades.push(app.grade.clone());
                    }
                    app.grade = Grade::default();
                    app.notify("Оценка сохранена");
                } else {
                    app.notify_error("Проверь предмет, дату и выбранную шкалу оценки.");
                }
            }
            if app.edit_grade.is_some()
                && glass_button(
                    ui,
                    &mut app.motion,
                    egui::Id::new("grade_cancel_btn"),
                    "Отмена",
                    None,
                    false,
                    theme,
                )
                .clicked()
            {
                app.edit_grade = None;
                app.grade = Grade::default();
            }
        });
    });

    ui.add_space(18.0);
    ui.label(semibold("История оценок", 16.5, theme.ink));
    ui.add_space(8.0);

    if app.d.grades.is_empty() {
        ui.label(
            RichText::new("Записей пока нет. Добавь оценку в форме выше.")
                .size(14.0)
                .color(theme.ink_secondary),
        );
        return;
    }

    // Tabular history sorted newest first
    let mut order: Vec<usize> = (0..app.d.grades.len()).collect();
    order.sort_by(|&a, &b| app.d.grades[b].date.cmp(&app.d.grades[a].date));

    let mut edit = None;
    let mut remove = None;

    for idx in order {
        let g = &app.d.grades[idx];
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
            let left_max = (ui.available_width() - 88.0).max(160.0);
            let left_resp = ui.allocate_ui_with_layout(
                Vec2::new(left_max, 0.0),
                egui::Layout::left_to_right(egui::Align::Min),
                |ui| {
                    ui.allocate_ui_with_layout(
                        Vec2::new(88.0, 24.0),
                        egui::Layout::left_to_right(egui::Align::Center),
                        |ui| {
                            ui.label(RichText::new(&g.date).size(13.0).color(theme.ink_secondary));
                        },
                    );
                    ui.allocate_ui_with_layout(
                        Vec2::new(142.0, 0.0),
                        egui::Layout::top_down(egui::Align::Min),
                        |ui| {
                            ui.add_space(2.5);
                            ui.add(
                                egui::Label::new(semibold(&g.subject, 14.0, theme.ink)).wrap(),
                            );
                        },
                    );
                    ui.allocate_ui_with_layout(
                        Vec2::new(46.0, 24.0),
                        egui::Layout::left_to_right(egui::Align::Center),
                        |ui| {
                            ui.label(semibold(format!("{}", g.value), 14.5, theme.accent));
                        },
                    );
                    if !g.note.trim().is_empty() {
                        let note_w = (ui.available_width() - 4.0).max(70.0);
                        ui.allocate_ui_with_layout(
                            Vec2::new(note_w, 0.0),
                            egui::Layout::top_down(egui::Align::Min),
                            |ui| {
                                ui.add_space(3.0);
                                ui.add(
                                    egui::Label::new(
                                        RichText::new(&g.note)
                                            .size(13.0)
                                            .color(theme.ink_secondary),
                                    )
                                    .wrap(),
                                );
                            },
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
                        egui::Id::new(("grade_del", idx)),
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
                        egui::Id::new(("grade_edit", idx)),
                        Icon::Edit,
                        app.edit_grade == Some(idx),
                        "Изменить запись",
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
        app.grade = app.d.grades[i].clone();
        app.edit_grade = Some(i);
    }
    if let Some(i) = remove {
        app.push_undo();
        app.d.grades.remove(i);
        app.edit_grade = None;
        app.notify("Оценка удалена");
    }
}
