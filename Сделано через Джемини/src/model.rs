use chrono::{Datelike, Duration, Local, NaiveDate, NaiveDateTime, NaiveTime};
use serde::{Deserialize, Serialize};
use std::{fs, io::Write, path::PathBuf};

pub const DAYS: [&str; 7] = [
    "Понедельник",
    "Вторник",
    "Среда",
    "Четверг",
    "Пятница",
    "Суббота",
    "Воскресенье",
];

pub const DAYS_SHORT: [&str; 7] = ["Пн", "Вт", "Ср", "Чт", "Пт", "Сб", "Вс"];

pub const MONTHS_GEN: [&str; 12] = [
    "января",
    "февраля",
    "марта",
    "апреля",
    "мая",
    "июня",
    "июля",
    "августа",
    "сентября",
    "октября",
    "ноября",
    "декабря",
];

pub const CATEGORIES: [&str; 4] = ["Учёба", "Секция", "Личное", "Контрольная"];

pub const DEFAULT_LESSON_SUBJECTS: [&str; 18] = [
    "Алгебра",
    "Геометрия",
    "Вероятность и статистика",
    "Русский язык",
    "Литература",
    "Английский язык",
    "История",
    "Обществознание",
    "География",
    "Биология",
    "Физика",
    "Химия",
    "Информатика",
    "Физкультура",
    "ОБЗР",
    "Музыка",
    "ИЗО",
    "Технология",
];

pub fn iso_week_key(d: NaiveDate) -> String {
    let iw = d.iso_week();
    format!("{}-W{:02}", iw.year(), iw.week())
}

pub fn week_range_label(d: NaiveDate) -> String {
    let offset = d.weekday().num_days_from_monday() as i64;
    let mon = d - Duration::days(offset);
    let sun = mon + Duration::days(6);
    if mon.month() == sun.month() {
        format!(
            "{}–{} {}",
            mon.day(),
            sun.day(),
            MONTHS_GEN[(sun.month0() as usize).min(11)]
        )
    } else {
        format!("{} – {}", ru_date_day_month(mon), ru_date_day_month(sun))
    }
}

pub fn today() -> NaiveDate {
    Local::now().date_naive()
}

pub fn now_time() -> NaiveTime {
    Local::now().time()
}

pub fn date(s: &str) -> Option<NaiveDate> {
    NaiveDate::parse_from_str(s.trim(), "%Y-%m-%d").ok()
}

pub fn time(s: &str) -> Option<NaiveTime> {
    NaiveTime::parse_from_str(s.trim(), "%H:%M").ok()
}

pub fn ru_date_long(d: NaiveDate) -> String {
    let m = MONTHS_GEN[(d.month0() as usize).min(11)];
    format!("{} {} {}", d.day(), m, d.year())
}

pub fn ru_date_day_month(d: NaiveDate) -> String {
    let m = MONTHS_GEN[(d.month0() as usize).min(11)];
    format!("{} {}", d.day(), m)
}

pub fn format_duration_ru(mins: i64) -> String {
    let m = mins.max(0);
    let h = m / 60;
    let rem = m % 60;
    match (h, rem) {
        (0, r) => format!("{r} мин"),
        (h, 0) => format!("{h} ч"),
        (h, r) => format!("{h} ч {r} мин"),
    }
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Event {
    pub name: String,
    pub place: String,
    pub day: usize,
    pub start: String,
    pub end: String,
    pub on_date: String,
    pub skip_dates: String,
    pub travel: i64,
    pub category: String,
    #[serde(default)]
    pub from_lessons: bool,
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
            category: "Учёба".into(),
            from_lessons: false,
        }
    }
}

impl Event {
    pub fn occurs(&self, d: NaiveDate) -> bool {
        if self
            .skip_dates
            .split(',')
            .any(|s| date(s.trim()) == Some(d))
        {
            return false;
        }
        if self.on_date.trim().is_empty() {
            self.day == d.weekday().num_days_from_monday() as usize
        } else {
            date(&self.on_date) == Some(d)
        }
    }

    pub fn effective_category(&self) -> &str {
        if self.category.trim().is_empty() {
            "Учёба"
        } else {
            self.category.as_str()
        }
    }

    pub fn add_skip_date(&mut self, d: NaiveDate) {
        let ds = d.to_string();
        let exists = self.skip_dates.split(',').any(|s| s.trim() == ds);
        if !exists {
            if self.skip_dates.trim().is_empty() {
                self.skip_dates = ds;
            } else {
                self.skip_dates = format!("{}, {}", self.skip_dates.trim(), ds);
            }
        }
    }
}

#[derive(Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Task {
    pub title: String,
    pub subject: String,
    pub due: String,
    pub minutes: i64,
    pub done: bool,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Grade {
    pub subject: String,
    pub value: f32,
    pub date: String,
    pub note: String,
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
#[serde(default)]
pub struct Night {
    pub date: String,
    pub bed: String,
    pub wake: String,
    pub hours: String,
    pub rem: String,
    pub core: String,
    pub deep: String,
    pub awake: String,
    pub awakenings: String,
    pub pulse: String,
    pub breathing: String,
    pub mood: String,
    pub notes: String,
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

fn default_true() -> bool {
    true
}

fn default_lesson_subjects() -> Vec<String> {
    DEFAULT_LESSON_SUBJECTS
        .iter()
        .map(|s| (*s).to_owned())
        .collect()
}

#[derive(Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub struct LessonSlot {
    pub subject: String,
    pub place: String,
}

impl Default for LessonSlot {
    fn default() -> Self {
        Self {
            subject: String::new(),
            place: String::new(),
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct WeeklyLessons {
    pub start_time: String,
    pub home_time: String,
    pub lesson_mins: i64,
    pub break_mins: i64,
    pub travel_mins: i64,
    #[serde(default = "default_lesson_subjects")]
    pub subjects_catalog: Vec<String>,
    pub days: [Vec<LessonSlot>; 7],
    pub confirmed_week: String,
}

impl Default for WeeklyLessons {
    fn default() -> Self {
        Self {
            start_time: "08:30".into(),
            home_time: "14:15".into(),
            lesson_mins: 45,
            break_mins: 10,
            travel_mins: 30,
            subjects_catalog: default_lesson_subjects(),
            days: [
                vec![],
                vec![],
                vec![],
                vec![],
                vec![],
                vec![],
                vec![],
            ],
            confirmed_week: String::new(),
        }
    }
}

impl WeeklyLessons {
    pub fn slot_times(&self, slot_idx: usize) -> Option<(NaiveTime, NaiveTime)> {
        let st = time(&self.start_time)?;
        let dur = self.lesson_mins.clamp(15, 180);
        let brk = self.break_mins.clamp(0, 60);
        let base_d = NaiveDate::from_ymd_opt(2026, 1, 1).unwrap();
        let offset_mins = slot_idx as i64 * (dur + brk);
        let s_dt = base_d.and_time(st) + Duration::minutes(offset_mins);
        let e_dt = s_dt + Duration::minutes(dur);
        if s_dt.date() != base_d || e_dt.date() != base_d {
            return None;
        }
        Some((s_dt.time(), e_dt.time()))
    }

    pub fn day_last_end(&self, wd: usize) -> Option<NaiveTime> {
        let len = self.days.get(wd)?.len();
        if len == 0 {
            return None;
        }
        self.slot_times(len - 1).map(|(_, end)| end)
    }

    pub fn has_any_lessons(&self) -> bool {
        self.days.iter().any(|d| !d.is_empty())
    }
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Data {
    pub events: Vec<Event>,
    pub tasks: Vec<Task>,
    pub grades: Vec<Grade>,
    pub nights: Vec<Night>,
    #[serde(default)]
    pub lesson_plan: WeeklyLessons,
    pub sleep_hours: f32,
    pub falling: i64,
    pub morning: i64,
    pub evening: i64,
    pub buffer: i64,
    pub free_wake: String,
    pub dark: bool,
    pub widget: bool,
    pub top: bool,
    pub grade_max: f32,
    #[serde(default = "default_true")]
    pub glass_effect: bool,
    #[serde(default)]
    pub reduce_transparency: bool,
    #[serde(default)]
    pub reduce_motion: bool,
    #[serde(default = "default_true")]
    pub schedule_grid: bool,
    #[serde(default)]
    pub sleep_tab: usize,
    #[serde(default)]
    pub has_demo_data: bool,
}

impl Default for Data {
    fn default() -> Self {
        Self {
            events: vec![],
            tasks: vec![],
            grades: vec![],
            nights: vec![],
            lesson_plan: WeeklyLessons::default(),
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
            glass_effect: true,
            reduce_transparency: false,
            reduce_motion: false,
            schedule_grid: true,
            sleep_tab: 0,
            has_demo_data: false,
        }
    }
}

impl Data {
    pub fn needs_weekly_lesson_check(&self, d: NaiveDate) -> bool {
        self.lesson_plan.confirmed_week != iso_week_key(d)
    }

    pub fn sync_lessons_to_events(&mut self) {
        self.events.retain(|e| !e.from_lessons);
        let travel_first = self.lesson_plan.travel_mins.max(0);
        for wd in 0..7 {
            let slots = self.lesson_plan.days[wd].clone();
            for (slot_idx, slot) in slots.into_iter().enumerate() {
                if slot.subject.trim().is_empty() {
                    continue;
                }
                let Some((st, en)) = self.lesson_plan.slot_times(slot_idx) else {
                    continue;
                };
                self.events.push(Event {
                    name: slot.subject,
                    place: slot.place,
                    day: wd,
                    start: st.format("%H:%M").to_string(),
                    end: en.format("%H:%M").to_string(),
                    on_date: String::new(),
                    skip_dates: String::new(),
                    travel: if slot_idx == 0 { travel_first } else { 0 },
                    category: "Учёба".into(),
                    from_lessons: true,
                });
            }
        }
    }
}

pub struct Plan {
    pub finish: NaiveDateTime,
    pub bed: NaiveDateTime,
    pub asleep: NaiveDateTime,
    pub wake: NaiveDateTime,
    pub leave: Option<NaiveDateTime>,
    pub first_event_name: Option<String>,
    pub first_event_time: Option<NaiveTime>,
}

pub fn plan(d: &Data, morning_date: NaiveDate) -> Plan {
    let first = d
        .events
        .iter()
        .filter(|e| e.occurs(morning_date))
        .filter_map(|e| time(&e.start).map(|t| (t, e)))
        .min_by_key(|(t, _)| *t);

    let (wake, leave, first_event_name, first_event_time) = if let Some((t, e)) = first {
        let leave = morning_date.and_time(t) - Duration::minutes(e.travel + d.buffer);
        (
            leave - Duration::minutes(d.morning),
            Some(leave),
            Some(e.name.clone()),
            Some(t),
        )
    } else {
        (
            morning_date.and_time(
                time(&d.free_wake).unwrap_or_else(|| NaiveTime::from_hms_opt(8, 0, 0).unwrap()),
            ),
            None,
            None,
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
        first_event_name,
        first_event_time,
    }
}

#[derive(Clone, Debug)]
pub struct CapacityReport {
    pub total_free_mins: i64,
    pub largest_window_mins: i64,
    pub task_count: usize,
    pub task_mins: i64,
    pub unfit_count: usize,
    pub fits: bool,
}

pub fn capacity_report(d: &Data, day: NaiveDate, current_time: NaiveTime) -> CapacityReport {
    let p = plan(d, day + Duration::days(1));
    let start_bound = day.and_time(current_time.max(NaiveTime::from_hms_opt(7, 0, 0).unwrap()));
    let end_bound = p.finish;

    if end_bound <= start_bound {
        let pending: Vec<i64> = d
            .tasks
            .iter()
            .filter(|t| !t.done && (t.due.trim().is_empty() || date(&t.due).is_some_and(|dd| dd <= day)))
            .map(|t| t.minutes.max(1))
            .collect();
        return CapacityReport {
            total_free_mins: 0,
            largest_window_mins: 0,
            task_count: pending.len(),
            task_mins: pending.iter().sum(),
            unfit_count: pending.len(),
            fits: pending.is_empty(),
        };
    }

    let mut busy: Vec<(NaiveDateTime, NaiveDateTime)> = d
        .events
        .iter()
        .filter(|e| e.occurs(day))
        .filter_map(|e| {
            let s = time(&e.start)?;
            let en = time(&e.end)?;
            let bs = day.and_time(s) - Duration::minutes(e.travel + d.buffer);
            let be = day.and_time(en);
            Some((bs.max(start_bound), be.min(end_bound)))
        })
        .filter(|(a, b)| a < b)
        .collect();

    // If this day has scheduled school lessons and a valid home-return time, block time until home_time.
    let has_lessons_today = d.events.iter().any(|e| e.from_lessons && e.occurs(day));
    if has_lessons_today {
        if let (Some(st), Some(ht)) = (time(&d.lesson_plan.start_time), time(&d.lesson_plan.home_time)) {
            if ht > st {
                let bs = day.and_time(st) - Duration::minutes(d.lesson_plan.travel_mins.max(0) + d.buffer);
                let be = day.and_time(ht);
                let span = (bs.max(start_bound), be.min(end_bound));
                if span.0 < span.1 {
                    busy.push(span);
                }
            }
        }
    }

    busy.sort_by_key(|(a, _)| *a);

    let mut merged: Vec<(NaiveDateTime, NaiveDateTime)> = Vec::new();
    for (s, e) in busy {
        if let Some(last) = merged.last_mut() {
            if s <= last.1 {
                if e > last.1 {
                    last.1 = e;
                }
                continue;
            }
        }
        merged.push((s, e));
    }

    let mut windows: Vec<i64> = Vec::new();
    let mut cursor = start_bound;
    for (bs, be) in merged {
        if bs > cursor {
            windows.push((bs - cursor).num_minutes());
        }
        if be > cursor {
            cursor = be;
        }
    }
    if end_bound > cursor {
        windows.push((end_bound - cursor).num_minutes());
    }

    let total_free_mins: i64 = windows.iter().sum();
    let largest_window_mins: i64 = windows.iter().copied().max().unwrap_or(0);

    let mut task_durations: Vec<i64> = d
        .tasks
        .iter()
        .filter(|t| !t.done && (t.due.trim().is_empty() || date(&t.due).is_some_and(|dd| dd <= day)))
        .map(|t| t.minutes.max(1))
        .collect();
    let task_count = task_durations.len();
    let task_mins: i64 = task_durations.iter().sum();

    // First-fit decreasing into real free windows
    task_durations.sort_by(|a, b| b.cmp(a));
    let mut rem_windows = windows;
    let mut unfit_count = 0usize;
    for dur in task_durations {
        if let Some(w) = rem_windows.iter_mut().find(|w| **w >= dur) {
            *w -= dur;
        } else {
            unfit_count += 1;
        }
    }

    CapacityReport {
        total_free_mins,
        largest_window_mins,
        task_count,
        task_mins,
        unfit_count,
        fits: unfit_count == 0,
    }
}

pub fn storage() -> PathBuf {
    if let Some(custom) = std::env::var_os("STUDY_PLANNER_DATA").filter(|s| !s.is_empty()) {
        return PathBuf::from(custom);
    }
    directories::ProjectDirs::from("", "", "StudyPlanner")
        .unwrap()
        .data_local_dir()
        .join("planner.json")
}

pub fn save_data(d: &Data, path: &std::path::Path) -> Result<(), String> {
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

pub fn demo_data(base_day: NaiveDate, current: &Data) -> Data {
    let wd = base_day.weekday().num_days_from_monday() as usize;
    let next_wd = (wd + 1) % 7;
    let prev_wd = (wd + 6) % 7;

    let mut lesson_plan = WeeklyLessons {
        start_time: "09:00".into(),
        home_time: "14:15".into(),
        lesson_mins: 45,
        break_mins: 15,
        travel_mins: 30,
        ..Default::default()
    };
    lesson_plan.days[wd] = vec![
        LessonSlot {
            subject: "Алгебра".into(),
            place: "Кабинет 204".into(),
        },
        LessonSlot {
            subject: "Русский язык".into(),
            place: "Кабинет 108".into(),
        },
        LessonSlot {
            subject: "Физика".into(),
            place: "Кабинет 312".into(),
        },
    ];
    lesson_plan.days[next_wd] = vec![LessonSlot {
        subject: "Английский язык".into(),
        place: "Кабинет 215".into(),
    }];
    lesson_plan.days[prev_wd] = vec![LessonSlot {
        subject: "История".into(),
        place: "Кабинет 112".into(),
    }];

    let mut data = Data {
        events: vec![
            Event {
                name: "Плавание".into(),
                place: "Бассейн «Волна»".into(),
                day: wd,
                start: "17:00".into(),
                end: "18:00".into(),
                travel: 25,
                category: "Секция".into(),
                ..Default::default()
            },
            Event {
                name: "Контрольная по химии".into(),
                place: "Кабинет 305".into(),
                day: next_wd,
                start: "11:00".into(),
                end: "11:45".into(),
                travel: 0,
                category: "Контрольная".into(),
                ..Default::default()
            },
        ],
        lesson_plan,
        tasks: vec![
            Task {
                title: "Решить задачи № 24–28".into(),
                subject: "Алгебра".into(),
                due: base_day.to_string(),
                minutes: 40,
                done: false,
            },
            Task {
                title: "Прочитать § 12 и выписать формулы".into(),
                subject: "Физика".into(),
                due: base_day.to_string(),
                minutes: 20,
                done: false,
            },
            Task {
                title: "Собрать сумку на плавание".into(),
                subject: "Личное".into(),
                due: base_day.to_string(),
                minutes: 10,
                done: false,
            },
            Task {
                title: "Повторить неправильные глаголы".into(),
                subject: "Английский".into(),
                due: (base_day + Duration::days(1)).to_string(),
                minutes: 25,
                done: false,
            },
        ],
        grades: vec![
            Grade {
                subject: "Алгебра".into(),
                value: 5.0,
                date: (base_day - Duration::days(2)).to_string(),
                note: "Самостоятельная работа".into(),
            },
            Grade {
                subject: "Физика".into(),
                value: 4.0,
                date: (base_day - Duration::days(4)).to_string(),
                note: "Лабораторная № 3".into(),
            },
            Grade {
                subject: "Русский язык".into(),
                value: 5.0,
                date: (base_day - Duration::days(1)).to_string(),
                note: "Сочинение".into(),
            },
        ],
        nights: vec![
            Night {
                date: (base_day - Duration::days(3)).to_string(),
                bed: "23:20".into(),
                wake: "07:35".into(),
                hours: "7.8".into(),
                rem: "92".into(),
                core: "285".into(),
                deep: "78".into(),
                awake: "12".into(),
                awakenings: "1".into(),
                pulse: "58".into(),
                breathing: "14".into(),
                mood: "Спокойно".into(),
                notes: "".into(),
            },
            Night {
                date: (base_day - Duration::days(2)).to_string(),
                bed: "23:45".into(),
                wake: "07:45".into(),
                hours: "7.5".into(),
                rem: "84".into(),
                core: "270".into(),
                deep: "72".into(),
                awake: "18".into(),
                awakenings: "2".into(),
                pulse: "61".into(),
                breathing: "15".into(),
                mood: "Сонливость утром".into(),
                notes: "Лёг позже обычного".into(),
            },
            Night {
                date: base_day.to_string(),
                bed: "23:10".into(),
                wake: "07:40".into(),
                hours: "8.2".into(),
                rem: "105".into(),
                core: "300".into(),
                deep: "85".into(),
                awake: "8".into(),
                awakenings: "1".into(),
                pulse: "56".into(),
                breathing: "14".into(),
                mood: "Бодро".into(),
                notes: "".into(),
            },
        ],
        dark: current.dark,
        widget: current.widget,
        top: current.top,
        glass_effect: current.glass_effect,
        reduce_transparency: current.reduce_transparency,
        reduce_motion: current.reduce_motion,
        schedule_grid: current.schedule_grid,
        sleep_tab: current.sleep_tab,
        has_demo_data: true,
        ..Default::default()
    };
    data.sync_lessons_to_events();
    data
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

    #[test]
    fn capacity_checks_real_windows() {
        let day = NaiveDate::from_ymd_opt(2026, 11, 10).unwrap(); // Tuesday (wd=1)
        let mut d = Data::default();
        d.buffer = 0;
        // Evening finish = 23:25 if no events tomorrow
        d.free_wake = "08:15".into(); // wake 08:15 -> asleep 00:15 -> bed 23:55 -> finish 23:25
        d.events.push(Event {
            name: "Секция".into(),
            day: 1,
            start: "21:55".into(),
            end: "22:35".into(),
            travel: 0,
            ..Default::default()
        });
        // From 21:05 to 23:25 -> gaps: [21:05..21:55]=50m and [22:35..23:25]=50m -> total 100m, max 50m!
        d.tasks.push(Task {
            title: "Большая задача".into(),
            due: day.to_string(),
            minutes: 90,
            done: false,
            ..Default::default()
        });
        let rep = capacity_report(&d, day, NaiveTime::from_hms_opt(21, 5, 0).unwrap());
        assert_eq!(rep.total_free_mins, 100);
        assert_eq!(rep.largest_window_mins, 50);
        assert!(!rep.fits);
        assert_eq!(rep.unfit_count, 1);
    }

    #[test]
    fn lessons_sync_preserves_manual_events_and_orders_slots() {
        let mut d = Data::default();
        d.events.push(Event {
            name: "Плавание".into(),
            day: 0,
            start: "17:00".into(),
            end: "18:00".into(),
            category: "Секция".into(),
            from_lessons: false,
            ..Default::default()
        });
        d.lesson_plan.start_time = "08:30".into();
        d.lesson_plan.home_time = "13:30".into();
        d.lesson_plan.lesson_mins = 45;
        d.lesson_plan.break_mins = 10;
        d.lesson_plan.travel_mins = 25;
        d.lesson_plan.days[0] = vec![
            LessonSlot {
                subject: "Алгебра".into(),
                place: String::new(),
            },
            LessonSlot {
                subject: "Физика".into(),
                place: String::new(),
            },
        ];
        d.sync_lessons_to_events();

        assert_eq!(d.events.len(), 3);
        assert!(d.events.iter().any(|e| e.name == "Плавание" && !e.from_lessons));
        let alg = d.events.iter().find(|e| e.name == "Алгебра").unwrap();
        assert_eq!(alg.start, "08:30");
        assert_eq!(alg.end, "09:15");
        assert_eq!(alg.travel, 25);
        let phy = d.events.iter().find(|e| e.name == "Физика").unwrap();
        assert_eq!(phy.start, "09:25");
        assert_eq!(phy.end, "10:10");
        assert_eq!(phy.travel, 0);
    }

    #[test]
    fn weekly_lesson_prompt_resets_on_new_iso_week() {
        let mon = NaiveDate::from_ymd_opt(2026, 11, 9).unwrap();
        let sun = NaiveDate::from_ymd_opt(2026, 11, 15).unwrap();
        let next_mon = NaiveDate::from_ymd_opt(2026, 11, 16).unwrap();

        let mut d = Data::default();
        assert!(d.needs_weekly_lesson_check(mon));
        d.lesson_plan.confirmed_week = iso_week_key(mon);
        assert!(!d.needs_weekly_lesson_check(sun));
        assert!(d.needs_weekly_lesson_check(next_mon));
    }

    #[test]
    fn home_time_blocks_daytime_capacity_after_lessons() {
        let mon = NaiveDate::from_ymd_opt(2026, 11, 9).unwrap(); // Monday (wd=0)
        let mut d = Data::default();
        d.buffer = 0;
        d.free_wake = "08:15".into(); // wake 08:15 -> asleep 00:15 -> bed 23:55 -> finish 23:25
        d.lesson_plan.start_time = "08:30".into();
        d.lesson_plan.home_time = "14:30".into();
        d.lesson_plan.lesson_mins = 45;
        d.lesson_plan.break_mins = 10;
        d.lesson_plan.travel_mins = 20;
        d.lesson_plan.days[0] = vec![LessonSlot {
            subject: "Алгебра".into(),
            place: String::new(),
        }];
        d.sync_lessons_to_events();

        // At 12:00, lesson ended at 09:15, but home_time is 14:30. Free from 14:30 to 23:25 = 8h 55m = 535m
        let rep = capacity_report(&d, mon, NaiveTime::from_hms_opt(12, 0, 0).unwrap());
        assert_eq!(rep.total_free_mins, 535);
    }

    #[test]
    fn backward_compatible_json() {
        let legacy = r#"{"events":[],"tasks":[],"grades":[],"nights":[],"sleep_hours":8.0,"falling":20,"morning":45,"evening":30,"buffer":10,"free_wake":"08:00","dark":true,"widget":true,"top":false,"grade_max":5.0}"#;
        let parsed: Data = serde_json::from_str(legacy).unwrap();
        assert!(parsed.dark);
        assert!(parsed.glass_effect);
        assert!(!parsed.reduce_motion);
        assert_eq!(parsed.lesson_plan.subjects_catalog.len(), 18);
    }
}
