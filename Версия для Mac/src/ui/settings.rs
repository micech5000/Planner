use crate::glass::{Icon, Theme, glass_button, glass_panel, semibold, settings_switch};
use crate::ui::PlannerApp;
use chrono::Local;
use eframe::egui::{self, RichText};
use std::fs;

pub fn show(app: &mut PlannerApp, ui: &mut egui::Ui, theme: &Theme) {
    ui.label(semibold("Настройки", 28.0, theme.ink));
    ui.label(
        RichText::new("Материалы, движение, виджет рабочего стола и локальные данные")
            .size(13.5)
            .color(theme.ink_secondary),
    );
    ui.add_space(10.0);

    let w = ui.available_width();

    // Group 1: Appearance & Motion
    glass_panel(ui, theme, 16, false, 12, |ui| {
        ui.set_min_width((w - 24.0).max(200.0));
        ui.label(semibold("Оформление и Liquid Glass", 15.5, theme.ink));
        ui.add_space(4.0);
        ui.spacing_mut().item_spacing.y = 5.0;
        let dark_label = if cfg!(target_os = "macos") {
            "Тёмная тема (⌘D)"
        } else {
            "Тёмная тема (Ctrl+D)"
        };
        settings_switch(
            ui,
            &mut app.motion,
            egui::Id::new("set_sw_dark"),
            &mut app.d.dark,
            dark_label,
            theme,
        );
        settings_switch(
            ui,
            &mut app.motion,
            egui::Id::new("set_sw_glass"),
            &mut app.d.glass_effect,
            "Эффект Liquid Glass (полупрозрачные слои и блики на гранях)",
            theme,
        );
        settings_switch(
            ui,
            &mut app.motion,
            egui::Id::new("set_sw_transp"),
            &mut app.d.reduce_transparency,
            "Уменьшить прозрачность (плотные фоны для максимальной контрастности)",
            theme,
        );
        settings_switch(
            ui,
            &mut app.motion,
            egui::Id::new("set_sw_motion"),
            &mut app.d.reduce_motion,
            "Уменьшить движение (заменить пространственные переходы на мягкое проявление)",
            theme,
        );
        if app.system_reduce_motion {
            let motion_note = if cfg!(target_os = "macos") {
                "Активна системная настройка macOS «Уменьшение движения» — пространственные сдвиги уже уменьшены."
            } else {
                "Активна системная настройка Windows «Отключить анимации» — пространственные сдвиги уже уменьшены."
            };
            ui.add(
                egui::Label::new(
                    RichText::new(motion_note)
                        .size(12.0)
                        .color(theme.ink_muted),
                )
                .wrap(),
            );
        }
    });

    ui.add_space(10.0);

    // Group 2: Widget & Tray
    glass_panel(ui, theme, 16, false, 12, |ui| {
        ui.set_min_width((w - 24.0).max(200.0));
        ui.label(semibold("Виджет и окно", 15.5, theme.ink));
        ui.add_space(4.0);
        ui.spacing_mut().item_spacing.y = 5.0;
        settings_switch(
            ui,
            &mut app.motion,
            egui::Id::new("set_sw_widget"),
            &mut app.d.widget,
            "Показывать виджет на рабочем столе",
            theme,
        );
        settings_switch(
            ui,
            &mut app.motion,
            egui::Id::new("set_sw_top"),
            &mut app.d.top,
            "Закрепить виджет поверх других окон",
            theme,
        );
        ui.add_space(2.0);
        ui.add(
            egui::Label::new(
                RichText::new(
                    "Перетаскивайте виджет за верхнюю область с датой. Закрытие основного окна сворачивает приложение в трей.",
                )
                .size(12.5)
                .color(theme.ink_secondary),
            )
            .wrap(),
        );
        if app.tray.is_none() {
            ui.add(
                egui::Label::new(
                    RichText::new("Трей недоступен: закрытие основного окна завершит приложение.")
                        .size(12.5)
                        .color(theme.danger),
                )
                .wrap(),
            );
        }
    });

    ui.add_space(10.0);

    // Group 3: Study preferences
    glass_panel(ui, theme, 16, false, 12, |ui| {
        ui.set_min_width((w - 24.0).max(200.0));
        ui.horizontal(|ui| {
            ui.label(semibold("Шкала оценок", 15.5, theme.ink));
            ui.add_space(12.0);
            ui.label(
                RichText::new("Максимум шкалы")
                    .size(13.5)
                    .color(theme.ink_secondary),
            );
            ui.add_sized(
                [72.0, 30.0],
                egui::DragValue::new(&mut app.d.grade_max).range(1.0..=100.0),
            );
        });
    });

    ui.add_space(10.0);

    // Group 4: Local Data & Backups
    glass_panel(ui, theme, 16, false, 12, |ui| {
        ui.set_min_width((w - 24.0).max(200.0));
        ui.label(semibold("Локальные данные и резервные копии", 15.5, theme.ink));
        ui.add_space(4.0);
        ui.add(
            egui::Label::new(
                RichText::new("Все данные хранятся только на этом компьютере в файле JSON:")
                    .size(13.0)
                    .color(theme.ink_secondary),
            )
            .wrap(),
        );
        ui.add_space(4.0);
        ui.add(
            egui::Label::new(
                RichText::new(app.path.display().to_string())
                    .monospace()
                    .size(12.0)
                    .color(theme.ink),
            )
            .wrap(),
        );
        ui.add_space(8.0);

        ui.horizontal_wrapped(|ui| {
            if glass_button(
                ui,
                &mut app.motion,
                egui::Id::new("set_open_folder"),
                "Открыть папку данных",
                None,
                false,
                theme,
            )
            .clicked()
            {
                if let Some(parent) = app.path.parent() {
                    #[cfg(target_os = "macos")]
                    let _ = std::process::Command::new("open").arg(parent).spawn();
                    #[cfg(target_os = "windows")]
                    let _ = std::process::Command::new("explorer").arg(parent).spawn();
                    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
                    let _ = std::process::Command::new("xdg-open").arg(parent).spawn();
                }
            }

            if glass_button(
                ui,
                &mut app.motion,
                egui::Id::new("set_backup"),
                "Создать резервную копию",
                None,
                false,
                theme,
            )
            .clicked()
            {
                let dest = app.path.with_file_name(format!(
                    "backup-{}.json",
                    Local::now().format("%Y%m%d-%H%M%S")
                ));
                match fs::write(&dest, serde_json::to_vec_pretty(&app.d).unwrap()) {
                    Ok(_) => app.notify(format!("Копия создана: {}", dest.display())),
                    Err(e) => app.notify_error(e.to_string()),
                }
            }

            if app.d.has_demo_data {
                if glass_button(
                    ui,
                    &mut app.motion,
                    egui::Id::new("set_clear_demo"),
                    "Очистить демо-данные",
                    Some(Icon::Trash),
                    false,
                    theme,
                )
                .clicked()
                {
                    app.clear_demo();
                }
            } else if glass_button(
                ui,
                &mut app.motion,
                egui::Id::new("set_load_demo"),
                "Загрузить демо-неделю",
                Some(Icon::Sparkles),
                false,
                theme,
            )
            .clicked()
            {
                app.load_demo();
            }

            if glass_button(
                ui,
                &mut app.motion,
                egui::Id::new("set_quit"),
                "Выйти",
                None,
                false,
                theme,
            )
            .clicked()
            {
                app.quit = true;
            }
        });
    });
}
