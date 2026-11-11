#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use chrono::{Datelike, Duration, Local, NaiveDate, NaiveDateTime, NaiveTime};
use eframe::egui::{self, Color32, RichText};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, fs, io::Write, path::PathBuf};
use tray_icon::{
    TrayIcon, TrayIconBuilder,
    menu::{Menu, MenuEvent, MenuItem},
};

const DAYS: [&str; 7] = [
    "Понедельник",
    "Вторник",
    "Среда",
    "Четверг",
    "Пятница",
    "Суббота",
    "Воскресенье",
];
const SECTIONS: [&str; 6] = [
    "Сегодня",
    "Расписание",
    "Задачи",
    "Оценки",
    "Сон",
    "Настройки",
];
fn today() -> NaiveDate {
    Local::now().date_naive()
}
fn date(s: &str) -> Option<NaiveDate> {
    NaiveDate::parse_from_str(s, "%Y-%m-%d").ok()
}
fn time(s: &str) -> Option<NaiveTime> {
    NaiveTime::parse_from_str(s, "%H:%M").ok()
}
fn field(ui: &mut egui::Ui, label: &str, value: &mut String) {
    ui.label(label);
    ui.text_edit_singleline(value);
}
fn number(ui: &mut egui::Ui, label: &str, value: &mut i64, range: std::ops::RangeInclusive<i64>) {
    ui.horizontal(|ui| {
        ui.label(label);
        ui.add(egui::DragValue::new(value).range(range));
    });
}

#[derive(Clone, Serialize, Deserialize)]
struct Event {
    name: String,
    place: String,
    day: usize,
    start: String,
    end: String,
    on_date: String,
    skip_dates: String,
    travel: i64,
}
impl Default for Event {
    fn default() -> Self {
        Self {
            name: String::new(),
            place: String::new(),
            day: 0,
            start: "09:00".into(),
            end: "09:45".into(),
            on_date: String::new(),
            skip_dates: String::new(),
            travel: 30,
        }
    }
}
impl Event {
    fn occurs(&self, d: NaiveDate) -> bool {
        if self
            .skip_dates
            .split(',')
            .any(|s| date(s.trim()) == Some(d))
        {
            return false;
        }
        if self.on_date.is_empty() {
            self.day == d.weekday().num_days_from_monday() as usize
        } else {
            date(&self.on_date) == Some(d)
        }
    }
}
#[derive(Clone, Default, Serialize, Deserialize)]
struct Task {
    title: String,
    subject: String,
    due: String,
    minutes: i64,
    done: bool,
}
#[derive(Clone, Serialize, Deserialize)]
struct Grade {
    subject: String,
    value: f32,
    date: String,
    note: String,
}
impl Default for Grade {
    fn default() -> Self {
        Self {
            subject: String::new(),
            value: 5.,
            date: today().to_string(),
            note: String::new(),
        }
    }
}
#[derive(Clone, Serialize, Deserialize)]
struct Night {
    date: String,
    bed: String,
    wake: String,
    hours: String,
    rem: String,
    core: String,
    deep: String,
    awake: String,
    awakenings: String,
    pulse: String,
    breathing: String,
    mood: String,
    notes: String,
}
impl Default for Night {
    fn default() -> Self {
        Self {
            date: today().to_string(),
            bed: String::new(),
            wake: String::new(),
            hours: String::new(),
            rem: String::new(),
            core: String::new(),
            deep: String::new(),
            awake: String::new(),
            awakenings: String::new(),
            pulse: String::new(),
            breathing: String::new(),
            mood: String::new(),
            notes: String::new(),
        }
    }
}
#[derive(Clone, Serialize, Deserialize)]
#[serde(default)]
struct Data {
    events: Vec<Event>,
    tasks: Vec<Task>,
    grades: Vec<Grade>,
    nights: Vec<Night>,
    sleep_hours: f32,
    falling: i64,
    morning: i64,
    evening: i64,
    buffer: i64,
    free_wake: String,
    dark: bool,
    widget: bool,
    top: bool,
    grade_max: f32,
}
impl Default for Data {
    fn default() -> Self {
        Self {
            events: vec![],
            tasks: vec![],
            grades: vec![],
            nights: vec![],
            sleep_hours: 8.,
            falling: 20,
            morning: 45,
            evening: 30,
            buffer: 10,
            free_wake: "08:00".into(),
            dark: false,
            widget: true,
            top: false,
            grade_max: 5.,
        }
    }
}
struct Plan {
    finish: NaiveDateTime,
    bed: NaiveDateTime,
    asleep: NaiveDateTime,
    wake: NaiveDateTime,
    leave: Option<NaiveDateTime>,
}
fn plan(d: &Data, morning_date: NaiveDate) -> Plan {
    let first = d
        .events
        .iter()
        .filter(|e| e.occurs(morning_date))
        .filter_map(|e| time(&e.start).map(|t| (t, e)))
        .min_by_key(|(t, _)| *t);
    let (wake, leave) = if let Some((t, e)) = first {
        let leave = morning_date.and_time(t) - Duration::minutes(e.travel + d.buffer);
        (leave - Duration::minutes(d.morning), Some(leave))
    } else {
        (
            morning_date
                .and_time(time(&d.free_wake).unwrap_or(NaiveTime::from_hms_opt(8, 0, 0).unwrap())),
            None,
        )
    };
    let asleep = wake - Duration::minutes((d.sleep_hours * 60.).round() as i64);
    let bed = asleep - Duration::minutes(d.falling);
    Plan {
        finish: bed - Duration::minutes(d.evening),
        bed,
        asleep,
        wake,
        leave,
    }
}
fn storage() -> PathBuf {
    directories::ProjectDirs::from("", "", "StudyPlanner")
        .unwrap()
        .data_local_dir()
        .join("planner.json")
}
fn save_data(d: &Data, path: &std::path::Path) -> Result<(), String> {
    let parent = path.parent().ok_or("Нет каталога данных")?;
    fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    let bytes = serde_json::to_vec_pretty(d).map_err(|e| e.to_string())?;
    let mut temp = tempfile::NamedTempFile::new_in(parent).map_err(|e| e.to_string())?;
    temp.write_all(&bytes)
        .and_then(|_| temp.as_file().sync_all())
        .map_err(|e| e.to_string())?;
    if path.exists() {
        fs::copy(path, path.with_extension("backup.json")).map_err(|e| e.to_string())?;
    }
    temp.persist(path).map_err(|e| e.to_string())?;
    Ok(())
}

struct App {
    d: Data,
    section: usize,
    event: Event,
    task: Task,
    grade: Grade,
    night: Night,
    edit_event: Option<usize>,
    edit_task: Option<usize>,
    edit_grade: Option<usize>,
    edit_night: Option<usize>,
    message: String,
    path: PathBuf,
    last: String,
    load_failed: bool,
    undo: Option<Data>,
    tray: Option<TrayIcon>,
    open_id: String,
    widget_id: String,
    quit_id: String,
    quit: bool,
    selected_day: usize,
}
impl App {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let mut fonts = egui::FontDefinitions::default();
        if let Ok(bytes) = fs::read("C:/Windows/Fonts/segoeui.ttf") {
            fonts
                .font_data
                .insert("Segoe".into(), egui::FontData::from_owned(bytes).into());
            fonts
                .families
                .entry(egui::FontFamily::Proportional)
                .or_default()
                .insert(0, "Segoe".into());
        }
        cc.egui_ctx.set_fonts(fonts);
        let mut style = (*cc.egui_ctx.style()).clone();
        style.spacing.item_spacing = egui::vec2(12., 10.);
        style.spacing.button_padding = egui::vec2(12., 7.);
        style
            .text_styles
            .insert(egui::TextStyle::Body, egui::FontId::proportional(16.));
        style
            .text_styles
            .insert(egui::TextStyle::Button, egui::FontId::proportional(15.));
        style
            .text_styles
            .insert(egui::TextStyle::Heading, egui::FontId::proportional(28.));
        cc.egui_ctx.set_style(style);
        let path = storage();
        let _ = fs::create_dir_all(path.parent().unwrap());
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
                let x = i % 32;
                let y = i / 32;
                if (8..24).contains(&x) && (8..24).contains(&y) {
                    [255, 255, 255, 255]
                } else {
                    [35, 112, 210, 255]
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
        Self {
            d,
            section: 0,
            event: Event::default(),
            task: Task {
                due: today().to_string(),
                minutes: 30,
                ..Default::default()
            },
            grade: Grade::default(),
            night: Night::default(),
            edit_event: None,
            edit_task: None,
            edit_grade: None,
            edit_night: None,
            message,
            path,
            last,
            load_failed,
            undo: None,
            tray,
            open_id: open.id().0.clone(),
            widget_id: widget.id().0.clone(),
            quit_id: quit.id().0.clone(),
            quit: false,
            selected_day: today().weekday().num_days_from_monday() as usize,
        }
    }
    fn task_rows(&mut self, ui: &mut egui::Ui, compact: bool) {
        if self.d.tasks.is_empty() {
            ui.weak("Добавь первое дело в разделе «Задачи».");
        }
        let mut remove = None;
        let mut edit = None;
        for (i, t) in self.d.tasks.iter_mut().enumerate() {
            if compact && !t.due.is_empty() && date(&t.due).is_some_and(|d| d > today()) {
                continue;
            }
            ui.push_id(i, |ui| {
                ui.horizontal_wrapped(|ui| {
                    let response = ui.selectable_label(t.done, if t.done { "●" } else { "○" });
                    if response.clicked() {
                        t.done = !t.done;
                    }
                    let text = RichText::new(&t.title);
                    ui.label(if t.done {
                        text.strikethrough().weak()
                    } else {
                        text
                    });
                });
                ui.horizontal_wrapped(|ui| {
                    ui.weak(format!("{}  ·  {} мин  ·  {}", t.subject, t.minutes, t.due));
                    if !compact {
                        if ui.small_button("Изменить").clicked() {
                            edit = Some(i);
                        }
                        if ui.small_button("Удалить").clicked() {
                            remove = Some(i);
                        }
                    }
                });
                ui.separator();
            });
        }
        if let Some(i) = edit {
            self.task = self.d.tasks[i].clone();
            self.edit_task = Some(i);
        }
        if let Some(i) = remove {
            self.undo = Some(self.d.clone());
            self.d.tasks.remove(i);
            self.edit_task = None;
        }
    }
    fn agenda(&self, ui: &mut egui::Ui, d: NaiveDate) {
        let mut events: Vec<_> = self.d.events.iter().filter(|e| e.occurs(d)).collect();
        events.sort_by_key(|e| time(&e.start));
        if events.is_empty() {
            ui.weak("На этот день занятий нет.");
        }
        for e in events {
            ui.horizontal(|ui| {
                ui.label(RichText::new(&e.start).color(Color32::from_rgb(55, 130, 220)));
                ui.label(RichText::new(&e.name).strong());
            });
            ui.weak(format!("До {}  ·  {}", e.end, e.place));
            ui.add_space(8.);
            ui.separator();
        }
    }
    fn sleep_line(&self, ui: &mut egui::Ui) {
        let p = plan(&self.d, today() + Duration::days(1));
        ui.label(
            RichText::new(format!(
                "Завершить дела: {}",
                p.finish.format("%d.%m в %H:%M")
            ))
            .strong(),
        );
        ui.weak(format!(
            "Лечь {}  ·  подъём завтра {}",
            p.bed.format("%H:%M"),
            p.wake.format("%H:%M")
        ));
    }
    fn today_ui(&mut self, ui: &mut egui::Ui) {
        ui.heading("Сегодня");
        ui.weak(format!(
            "{}, {}",
            DAYS[today().weekday().num_days_from_monday() as usize],
            today().format("%d.%m.%Y")
        ));
        ui.add_space(20.);
        ui.columns(2, |cols| {
            cols[0].label(RichText::new("Занятия").size(19.).strong());
            self.agenda(&mut cols[0], today());
            cols[1].label(RichText::new("Сделать").size(19.).strong());
            self.task_rows(&mut cols[1], true);
        });
        ui.add_space(24.);
        self.sleep_line(ui);
    }
    fn schedule(&mut self, ui: &mut egui::Ui) {
        ui.heading("Расписание");
        ui.weak("Повторяющаяся неделя и отдельные даты");
        ui.horizontal_wrapped(|ui| {
            for (i, n) in DAYS.iter().enumerate() {
                ui.selectable_value(&mut self.selected_day, i, *n);
            }
        });
        let mut edit = None;
        let mut remove = None;
        for (i, e) in self
            .d
            .events
            .iter()
            .enumerate()
            .filter(|(_, e)| e.day == self.selected_day || !e.on_date.is_empty())
        {
            ui.horizontal_wrapped(|ui| {
                ui.label(format!("{}–{}  {}  ·  {}", e.start, e.end, e.name, e.place));
                if !e.on_date.is_empty() {
                    ui.weak(&e.on_date);
                }
                if ui.small_button("Изменить").clicked() {
                    edit = Some(i);
                }
                if ui.small_button("Удалить").clicked() {
                    remove = Some(i);
                }
            });
            ui.separator();
        }
        if let Some(i) = edit {
            self.event = self.d.events[i].clone();
            self.edit_event = Some(i);
        }
        if let Some(i) = remove {
            self.undo = Some(self.d.clone());
            self.d.events.remove(i);
            self.edit_event = None;
        }
        ui.add_space(18.);
        ui.label(
            RichText::new(if self.edit_event.is_some() {
                "Изменить занятие"
            } else {
                "Новое занятие"
            })
            .strong(),
        );
        field(ui, "Название / предмет", &mut self.event.name);
        field(ui, "Кабинет или место", &mut self.event.place);
        egui::ComboBox::from_label("День недели")
            .selected_text(DAYS[self.event.day.min(6)])
            .show_ui(ui, |ui| {
                for (i, n) in DAYS.iter().enumerate() {
                    ui.selectable_value(&mut self.event.day, i, *n);
                }
            });
        ui.horizontal(|ui| {
            field(ui, "Начало", &mut self.event.start);
            field(ui, "Конец", &mut self.event.end);
        });
        number(ui, "Дорога, мин", &mut self.event.travel, 0..=240);
        ui.collapsing("Отдельная дата и исключения",|ui|{field(ui,"Только дата (ГГГГ-ММ-ДД), пусто — каждую неделю",&mut self.event.on_date);field(ui,"Пропустить даты через запятую",&mut self.event.skip_dates);ui.weak("Чтобы изменить один день: исключи его из серии и добавь отдельное занятие на эту дату.");});
        if ui.button("Сохранить занятие").clicked() {
            let e = &self.event;
            let valid = !e.name.trim().is_empty()
                && time(&e.start).zip(time(&e.end)).is_some_and(|(a, b)| a < b)
                && (e.on_date.is_empty() || date(&e.on_date).is_some())
                && e.skip_dates
                    .split(',')
                    .all(|s| s.trim().is_empty() || date(s.trim()).is_some());
            if valid {
                self.undo = Some(self.d.clone());
                if let Some(i) = self.edit_event.take() {
                    self.d.events[i] = self.event.clone();
                } else {
                    self.d.events.push(self.event.clone());
                }
                self.event = Event::default();
                self.message = "Занятие сохранено".into();
            } else {
                self.message =
                    "Проверь название, даты и время ЧЧ:ММ. Конец должен быть позже начала.".into();
            }
        }
    }
    fn tasks(&mut self, ui: &mut egui::Ui) {
        ui.heading("Задачи");
        self.task_rows(ui, false);
        ui.add_space(14.);
        field(ui, "Задача", &mut self.task.title);
        field(ui, "Предмет", &mut self.task.subject);
        field(ui, "Срок (ГГГГ-ММ-ДД)", &mut self.task.due);
        number(ui, "Длительность, мин", &mut self.task.minutes, 1..=1440);
        if ui.button("Сохранить задачу").clicked() {
            if !self.task.title.trim().is_empty()
                && (self.task.due.is_empty() || date(&self.task.due).is_some())
            {
                self.undo = Some(self.d.clone());
                if let Some(i) = self.edit_task.take() {
                    self.d.tasks[i] = self.task.clone();
                } else {
                    self.d.tasks.push(self.task.clone());
                }
                self.task = Task {
                    due: today().to_string(),
                    minutes: 30,
                    ..Default::default()
                };
                self.message = "Задача сохранена".into();
            } else {
                self.message = "Укажи название и корректную дату срока".into();
            }
        }
    }
    fn grades(&mut self, ui: &mut egui::Ui) {
        ui.heading("Оценки");
        let mut groups: BTreeMap<&str, (f32, usize)> = BTreeMap::new();
        for g in &self.d.grades {
            let v = groups.entry(&g.subject).or_default();
            v.0 += g.value;
            v.1 += 1;
        }
        for (s, (sum, n)) in groups {
            ui.label(format!("{s}   {:.2} · оценок: {n}", sum / n as f32));
        }
        ui.separator();
        let mut remove = None;
        let mut edit = None;
        for (i, g) in self.d.grades.iter().enumerate() {
            ui.horizontal_wrapped(|ui| {
                ui.label(format!(
                    "{}  {}  —  {}  {}",
                    g.date, g.subject, g.value, g.note
                ));
                if ui.small_button("Изменить").clicked() {
                    edit = Some(i);
                }
                if ui.small_button("Удалить").clicked() {
                    remove = Some(i);
                }
            });
        }
        if let Some(i) = edit {
            self.grade = self.d.grades[i].clone();
            self.edit_grade = Some(i);
        }
        if let Some(i) = remove {
            self.undo = Some(self.d.clone());
            self.d.grades.remove(i);
            self.edit_grade = None;
        }
        ui.separator();
        field(ui, "Предмет", &mut self.grade.subject);
        field(ui, "Дата", &mut self.grade.date);
        ui.horizontal(|ui| {
            ui.label("Оценка");
            ui.add(
                egui::DragValue::new(&mut self.grade.value)
                    .range(0.0..=self.d.grade_max)
                    .speed(0.1),
            );
        });
        field(ui, "Комментарий", &mut self.grade.note);
        if ui.button("Сохранить оценку").clicked() {
            if !self.grade.subject.trim().is_empty()
                && date(&self.grade.date).is_some()
                && self.grade.value <= self.d.grade_max
            {
                self.undo = Some(self.d.clone());
                if let Some(i) = self.edit_grade.take() {
                    self.d.grades[i] = self.grade.clone();
                } else {
                    self.d.grades.push(self.grade.clone());
                }
                self.grade = Grade::default();
            } else {
                self.message = "Проверь предмет, дату и шкалу оценки".into();
            }
        }
    }
    fn sleep(&mut self, ui: &mut egui::Ui) {
        ui.heading("Сон");
        ui.weak("План на следующую ночь");
        let p = plan(&self.d, today() + Duration::days(1));
        ui.horizontal_wrapped(|ui| {
            for (label, t) in [
                ("Завершить дела", p.finish),
                ("Лечь", p.bed),
                ("Уснуть", p.asleep),
                ("Подъём", p.wake),
            ] {
                ui.vertical(|ui| {
                    ui.weak(label);
                    ui.label(RichText::new(t.format("%H:%M").to_string()).size(25.));
                    ui.small(t.format("%d.%m").to_string());
                });
            }
            if let Some(t) = p.leave {
                ui.label(format!("Выход {}", t.format("%H:%M")));
            }
        });
        ui.collapsing("Настроить расчёт", |ui| {
            ui.horizontal(|ui| {
                ui.label("Длительность сна, ч");
                ui.add(
                    egui::DragValue::new(&mut self.d.sleep_hours)
                        .range(1.0..=16.0)
                        .speed(0.25),
                );
            });
            number(ui, "Засыпание, мин", &mut self.d.falling, 0..=180);
            number(ui, "Утренние сборы, мин", &mut self.d.morning, 0..=240);
            number(ui, "Вечерняя подготовка, мин", &mut self.d.evening, 0..=240);
            number(ui, "Запас на дорогу, мин", &mut self.d.buffer, 0..=120);
            field(ui, "Подъём в свободный день ЧЧ:ММ", &mut self.d.free_wake);
            if time(&self.d.free_wake).is_none() {
                ui.colored_label(
                    Color32::RED,
                    "Некорректное время; временно используется 08:00",
                );
            }
        });
        ui.add_space(16.);
        ui.label(RichText::new("Дневник и аналитика").size(20.).strong());
        let mut values: Vec<_> = self
            .d
            .nights
            .iter()
            .filter_map(|n| Some((date(&n.date)?, n.hours.parse::<f64>().ok()?)))
            .collect();
        values.sort_by_key(|(d, _)| *d);
        if values.is_empty() {
            ui.weak("Запиши длительность сна — здесь появится график.");
        } else {
            let avg = values.iter().map(|v| v.1).sum::<f64>() / values.len() as f64;
            ui.label(format!(
                "Средний сон {:.1} ч · {} записей за всё время",
                avg,
                values.len()
            ));
            let bars = values
                .iter()
                .map(|(d, h)| {
                    egui_plot::Bar::new(d.num_days_from_ce() as f64, *h)
                        .width(0.65)
                        .name(d.format("%d.%m").to_string())
                })
                .collect();
            egui_plot::Plot::new("sleep_history")
                .height(150.)
                .show_axes([false, true])
                .include_y(0.)
                .allow_drag(false)
                .show(ui, |p| {
                    p.bar_chart(egui_plot::BarChart::new("Сон, часов", bars))
                });
        }
        let mut remove = None;
        let mut edit = None;
        for (i, n) in self.d.nights.iter().enumerate() {
            ui.horizontal_wrapped(|ui| {
                ui.label(format!(
                    "{} · {} ч · пульс {} · дыхание {}",
                    n.date,
                    if n.hours.is_empty() { "—" } else { &n.hours },
                    if n.pulse.is_empty() { "—" } else { &n.pulse },
                    if n.breathing.is_empty() {
                        "—"
                    } else {
                        &n.breathing
                    }
                ));
                if ui.small_button("Открыть").clicked() {
                    edit = Some(i);
                }
                if ui.small_button("Удалить").clicked() {
                    remove = Some(i);
                }
            });
        }
        if let Some(i) = edit {
            self.night = self.d.nights[i].clone();
            self.edit_night = Some(i);
        }
        if let Some(i) = remove {
            self.undo = Some(self.d.clone());
            self.d.nights.remove(i);
            self.edit_night = None;
        }
        ui.separator();
        ui.label("Запись по дате пробуждения · поля показателей необязательны");
        egui::Grid::new("night_form")
            .num_columns(2)
            .spacing([20., 8.])
            .show(ui, |ui| {
                for (label, value) in [
                    ("Дата ГГГГ-ММ-ДД", &mut self.night.date),
                    ("Лёг ЧЧ:ММ", &mut self.night.bed),
                    ("Проснулся ЧЧ:ММ", &mut self.night.wake),
                    ("Фактический сон, часов", &mut self.night.hours),
                    ("Быстрый сон, мин", &mut self.night.rem),
                    ("Базовый сон, мин", &mut self.night.core),
                    ("Глубокий сон, мин", &mut self.night.deep),
                    ("Бодрствование ночью, мин", &mut self.night.awake),
                    ("Количество пробуждений", &mut self.night.awakenings),
                    ("Средний пульс, уд/мин", &mut self.night.pulse),
                    ("Дыхание, в минуту", &mut self.night.breathing),
                    ("Самочувствие", &mut self.night.mood),
                ] {
                    field(ui, label, value);
                    ui.end_row();
                }
            });
        ui.label("Сновидения и заметки");
        ui.text_edit_multiline(&mut self.night.notes);
        if ui.button("Сохранить запись сна").clicked() {
            let n = &self.night;
            let numeric = [
                (&n.hours, 24.),
                (&n.rem, 1440.),
                (&n.core, 1440.),
                (&n.deep, 1440.),
                (&n.awake, 1440.),
                (&n.awakenings, 1000.),
                (&n.pulse, 300.),
                (&n.breathing, 100.),
            ]
            .iter()
            .all(|(s, max)| {
                s.is_empty()
                    || s.parse::<f32>()
                        .is_ok_and(|v| v.is_finite() && v >= 0. && v <= *max)
            });
            let duplicate = self
                .d
                .nights
                .iter()
                .enumerate()
                .any(|(i, v)| v.date == n.date && Some(i) != self.edit_night);
            if date(&n.date).is_some()
                && numeric
                && !duplicate
                && [&n.bed, &n.wake]
                    .iter()
                    .all(|s| s.is_empty() || time(s).is_some())
            {
                self.undo = Some(self.d.clone());
                if let Some(i) = self.edit_night.take() {
                    self.d.nights[i] = self.night.clone();
                } else {
                    self.d.nights.push(self.night.clone());
                }
                self.night = Night::default();
                self.message = "Запись сна сохранена".into();
            } else {
                self.message="Проверь дату, время и числовые поля (десятичная точка). На одну дату — одна запись.".into();
            }
        }
    }
    fn settings(&mut self, ui: &mut egui::Ui) {
        ui.heading("Настройки");
        ui.checkbox(&mut self.d.dark, "Тёмная тема");
        ui.checkbox(&mut self.d.widget, "Показывать виджет");
        ui.checkbox(&mut self.d.top, "Виджет поверх других окон");
        ui.horizontal(|ui| {
            ui.label("Максимум шкалы оценок");
            ui.add(egui::DragValue::new(&mut self.d.grade_max).range(1.0..=100.));
        });
        ui.separator();
        ui.label("Данные хранятся только на этом компьютере");
        ui.monospace(self.path.display().to_string());
        if ui.button("Открыть папку данных").clicked() {
            let _ = std::process::Command::new("explorer")
                .arg(self.path.parent().unwrap())
                .spawn();
        }
        if ui.button("Создать резервную копию").clicked() {
            let dest = self.path.with_file_name(format!(
                "backup-{}.json",
                Local::now().format("%Y%m%d-%H%M%S")
            ));
            self.message = match fs::write(&dest, serde_json::to_vec_pretty(&self.d).unwrap()) {
                Ok(_) => format!("Копия: {}", dest.display()),
                Err(e) => e.to_string(),
            };
        }
        ui.weak("Закрытие окна сворачивает приложение в трей. Выход — через меню значка.");
        if self.tray.is_none() {
            ui.colored_label(
                Color32::RED,
                "Трей недоступен: закрытие окна завершит приложение.",
            );
        }
        if ui.button("Выйти из приложения").clicked() {
            self.quit = true;
        }
    }
}
impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _: &mut eframe::Frame) {
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
        let mut visuals = if self.d.dark {
            egui::Visuals::dark()
        } else {
            egui::Visuals::light()
        };
        visuals.selection.bg_fill = Color32::from_rgb(45, 113, 185);
        if !self.d.dark {
            visuals.selection.bg_fill = Color32::from_rgb(218, 232, 248);
            visuals.panel_fill = Color32::from_rgb(250, 250, 252);
            visuals.override_text_color = Some(Color32::from_rgb(38, 40, 45));
        }
        ctx.set_visuals(visuals);
        egui::SidePanel::left("navigation")
            .exact_width(210.)
            .resizable(false)
            .frame(egui::Frame::new().inner_margin(16).fill(if self.d.dark {
                Color32::from_rgb(38, 39, 43)
            } else {
                Color32::from_rgb(240, 241, 244)
            }))
            .show(ctx, |ui| {
                ui.add_space(24.);
                ui.label(RichText::new("Мой день").size(22.).strong());
                ui.add_space(28.);
                for (i, name) in SECTIONS.iter().enumerate() {
                    if ui
                        .add_sized(
                            [168., 38.],
                            egui::Button::new(*name)
                                .selected(self.section == i)
                                .frame(self.section == i),
                        )
                        .clicked()
                    {
                        self.section = i;
                        self.message.clear();
                    }
                }
            });
        egui::TopBottomPanel::bottom("status").show(ctx, |ui| {
            ui.horizontal_wrapped(|ui| {
                if self.load_failed {
                    ui.colored_label(Color32::RED, &self.message);
                } else {
                    ui.weak(if self.message.is_empty() {
                        "Локально на этом компьютере"
                    } else {
                        &self.message
                    });
                }
                if self.undo.is_some() && ui.small_button("Отменить последнее изменение").clicked()
                {
                    self.d = self.undo.take().unwrap();
                    self.edit_event = None;
                    self.edit_task = None;
                    self.edit_grade = None;
                    self.edit_night = None;
                }
            });
        });
        egui::CentralPanel::default()
            .frame(egui::Frame::new().inner_margin(24))
            .show(ctx, |ui| {
                ui.add_space(16.);
                egui::ScrollArea::vertical().show(ui, |ui| match self.section {
                    0 => self.today_ui(ui),
                    1 => self.schedule(ui),
                    2 => self.tasks(ui),
                    3 => self.grades(ui),
                    4 => self.sleep(ui),
                    _ => self.settings(ui),
                });
            });
        if self.d.widget {
            let mut visible = true;
            ctx.show_viewport_immediate(
                egui::ViewportId::from_hash_of("desktop_widget"),
                egui::ViewportBuilder::default()
                    .with_title("Мой день · виджет")
                    .with_inner_size([340., 440.])
                    .with_min_inner_size([280., 280.])
                    .with_active(false),
                |ctx, _| {
                    ctx.send_viewport_cmd(egui::ViewportCommand::WindowLevel(if self.d.top {
                        egui::WindowLevel::AlwaysOnTop
                    } else {
                        egui::WindowLevel::Normal
                    }));
                    if ctx.input(|i| i.viewport().close_requested()) {
                        visible = false;
                    }
                    egui::CentralPanel::default().show(ctx, |ui| {
                        egui::ScrollArea::vertical().show(ui, |ui| {
                            ui.label(
                                RichText::new(today().format("%d %m %Y").to_string())
                                    .size(21.)
                                    .strong(),
                            );
                            self.agenda(ui, today());
                            self.task_rows(ui, true);
                            ui.add_space(10.);
                            self.sleep_line(ui);
                            if ui.button("Открыть приложение").clicked() {
                                ctx.send_viewport_cmd_to(
                                    egui::ViewportId::ROOT,
                                    egui::ViewportCommand::Visible(true),
                                );
                                ctx.send_viewport_cmd_to(
                                    egui::ViewportId::ROOT,
                                    egui::ViewportCommand::Focus,
                                );
                            }
                        });
                    });
                },
            );
            self.d.widget = visible;
        }
        let serialized = serde_json::to_string(&self.d).unwrap();
        if serialized != self.last && !self.load_failed {
            match save_data(&self.d, &self.path) {
                Ok(()) => self.last = serialized,
                Err(e) => self.message = format!("Ошибка сохранения: {e}"),
            }
        }
    }
}
fn main() -> eframe::Result {
    eframe::run_native(
        "Мой день",
        eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default()
                .with_inner_size([1100., 800.])
                .with_min_inner_size([850., 600.]),
            ..Default::default()
        },
        Box::new(|cc| Ok(Box::new(App::new(cc)))),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn sleep_crosses_midnight() {
        let mut d = Data::default();
        d.buffer = 0;
        d.events.push(Event {
            day: 2,
            ..Default::default()
        });
        let p = plan(&d, NaiveDate::from_ymd_opt(2026, 11, 11).unwrap());
        assert_eq!(p.finish.to_string(), "2026-11-10 22:55:00");
        assert_eq!(p.wake.time().to_string(), "07:45:00");
    }
    #[test]
    fn exception_and_one_off() {
        let d = NaiveDate::from_ymd_opt(2026, 11, 11).unwrap();
        let mut e = Event {
            day: 2,
            ..Default::default()
        };
        assert!(e.occurs(d));
        e.skip_dates = d.to_string();
        assert!(!e.occurs(d));
        e.skip_dates.clear();
        e.on_date = "2026-11-12".into();
        assert!(!e.occurs(d));
        assert!(e.occurs(d + Duration::days(1)));
    }
    #[test]
    fn atomic_save_and_backup() {
        let temp = tempfile::tempdir().unwrap();
        let p = temp.path().join("planner.json");
        let mut d = Data::default();
        save_data(&d, &p).unwrap();
        d.dark = true;
        save_data(&d, &p).unwrap();
        let loaded: Data = serde_json::from_slice(&fs::read(&p).unwrap()).unwrap();
        assert!(loaded.dark);
        let prev: Data =
            serde_json::from_slice(&fs::read(p.with_extension("backup.json")).unwrap()).unwrap();
        assert!(!prev.dark);
    }
}
