use crate::ui::{
    SEC_GRADES, SEC_LESSONS, SEC_SCHEDULE, SEC_SETTINGS, SEC_SLEEP, SEC_TASKS, SEC_TODAY,
};
use eframe::egui;
use std::fs;
use std::path::PathBuf;

pub struct Inspector {
    pub out_dir: PathBuf,
    step: usize,
    step_initialized: bool,
    frames_in_step: usize,
    settled_frames: usize,
    requested_main: bool,
    pub finished: bool,
}

impl Inspector {
    pub fn is_env_enabled() -> bool {
        std::env::var_os("STUDY_PLANNER_INSPECT_DIR")
            .is_some_and(|s| !s.is_empty())
    }

    pub fn from_env() -> Option<Self> {
        let dir = std::env::var_os("STUDY_PLANNER_INSPECT_DIR")
            .filter(|s| !s.is_empty())
            .map(PathBuf::from)?;
        let _ = fs::create_dir_all(&dir);
        Some(Self {
            out_dir: dir,
            step: 0,
            step_initialized: false,
            frames_in_step: 0,
            settled_frames: 0,
            requested_main: false,
            finished: false,
        })
    }

    pub fn is_widget_step(&self) -> bool {
        matches!(self.step, 14 | 15)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn tick_main(
        &mut self,
        ctx: &egui::Context,
        app_section: &mut usize,
        dark: &mut bool,
        widget: &mut bool,
        schedule_grid: &mut bool,
        sleep_tab: &mut usize,
        show_completed_tasks: &mut bool,
        motion_is_moving: bool,
    ) {
        if self.finished {
            return;
        }
        ctx.request_repaint();

        // Collect any pending root screenshot events
        let events: Vec<egui::Event> = ctx.input(|i| i.events.clone());
        for ev in events {
            if let egui::Event::Screenshot {
                user_data, image, ..
            } = ev
            {
                if let Some(name) = user_data.data.as_ref().and_then(|d| d.downcast_ref::<String>())
                {
                    let bg = if *dark { [22, 26, 36] } else { [214, 224, 238] };
                    let _ = save_color_image_png(&self.out_dir.join(name), &image, bg);
                    self.step += 1;
                    self.step_initialized = false;
                    self.frames_in_step = 0;
                    self.settled_frames = 0;
                    self.requested_main = false;
                    return;
                }
            }
        }

        // Configure scene for current step
        let target: Option<(&str, Option<[f32; 2]>)> = match self.step {
            0 => {
                *dark = false;
                *widget = true;
                *app_section = SEC_TODAY;
                *schedule_grid = true;
                *sleep_tab = 0;
                Some(("01_today_light.png", Some([1120.0, 780.0])))
            }
            1 => {
                *dark = false;
                *app_section = SEC_LESSONS;
                Some(("02_lessons_light.png", None))
            }
            2 => {
                *dark = false;
                *app_section = SEC_SCHEDULE;
                *schedule_grid = true;
                Some(("03_schedule_grid_light.png", None))
            }
            3 => {
                *dark = false;
                *app_section = SEC_SCHEDULE;
                *schedule_grid = false;
                Some(("04_schedule_list_light.png", None))
            }
            4 => {
                *dark = false;
                *app_section = SEC_TASKS;
                *show_completed_tasks = true;
                Some(("05_tasks_light.png", None))
            }
            5 => {
                *dark = false;
                *app_section = SEC_GRADES;
                Some(("06_grades_light.png", None))
            }
            6 => {
                *dark = false;
                *app_section = SEC_SLEEP;
                *sleep_tab = 0;
                Some(("07_sleep_overview_light.png", None))
            }
            7 => {
                *dark = false;
                *app_section = SEC_SLEEP;
                *sleep_tab = 1;
                Some(("08_sleep_diary_light.png", None))
            }
            8 => {
                *dark = false;
                *app_section = SEC_SETTINGS;
                Some(("09_settings_light.png", None))
            }
            9 => {
                *dark = true;
                *app_section = SEC_TODAY;
                *schedule_grid = true;
                Some(("10_today_dark.png", None))
            }
            10 => {
                *dark = true;
                *app_section = SEC_LESSONS;
                Some(("11_lessons_dark.png", None))
            }
            11 => {
                *dark = true;
                *app_section = SEC_SCHEDULE;
                *schedule_grid = true;
                Some(("12_schedule_dark.png", None))
            }
            12 => {
                *dark = false;
                *app_section = SEC_TODAY;
                Some(("13_compact_light.png", Some([680.0, 620.0])))
            }
            13 => {
                *dark = false;
                *app_section = SEC_LESSONS;
                Some(("14_lessons_compact_light.png", Some([680.0, 620.0])))
            }
            14 => {
                *dark = false;
                *app_section = SEC_TODAY;
                Some(("15_widget_light.png", Some([328.0, 392.0])))
            }
            15 => {
                *dark = true;
                *app_section = SEC_TODAY;
                Some(("16_widget_dark.png", Some([328.0, 392.0])))
            }
            _ => {
                self.finished = true;
                None
            }
        };

        if let Some((filename, resize_to)) = target {
            if !self.step_initialized {
                self.step_initialized = true;
                if let Some(sz) = resize_to {
                    ctx.send_viewport_cmd(egui::ViewportCommand::MinInnerSize(egui::Vec2::new(
                        276.0, 280.0,
                    )));
                    ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(egui::Vec2::new(
                        sz[0], sz[1],
                    )));
                }
            }

            self.frames_in_step += 1;

            let size_ready = resize_to.is_none_or(|sz| {
                let rect = ctx.content_rect();
                (rect.width() - sz[0]).abs() <= 4.0 && (rect.height() - sz[1]).abs() <= 4.0
            });

            if self.frames_in_step >= 5 && size_ready && !motion_is_moving {
                self.settled_frames += 1;
            } else {
                self.settled_frames = 0;
            }

            if !self.requested_main
                && (self.settled_frames >= 3 || self.frames_in_step >= 180)
            {
                self.requested_main = true;
                ctx.send_viewport_cmd(egui::ViewportCommand::Screenshot(egui::UserData::new(
                    filename.to_owned(),
                )));
            }
        }
    }
}

/// Writes an `egui::ColorImage` (premultiplied RGBA) composited over `bg_rgb` as a valid uncompressed PNG.
pub fn save_color_image_png(
    path: &std::path::Path,
    img: &egui::ColorImage,
    bg_rgb: [u8; 3],
) -> std::io::Result<()> {
    let w = img.width();
    let h = img.height();
    if w == 0 || h == 0 {
        return Ok(());
    }

    let mut raw_scanlines = Vec::with_capacity(h * (1 + w * 4));
    for y in 0..h {
        raw_scanlines.push(0u8); // Filter type 0 (None)
        for x in 0..w {
            let p = img.pixels[y * w + x];
            let a = p.a() as f32 / 255.0;
            let inv = 1.0 - a;
            // Framebuffer is premultiplied RGBA
            let r = (p.r() as f32 + bg_rgb[0] as f32 * inv).round().clamp(0.0, 255.0) as u8;
            let g = (p.g() as f32 + bg_rgb[1] as f32 * inv).round().clamp(0.0, 255.0) as u8;
            let b = (p.b() as f32 + bg_rgb[2] as f32 * inv).round().clamp(0.0, 255.0) as u8;
            raw_scanlines.extend_from_slice(&[r, g, b, 255]);
        }
    }

    let png_bytes = encode_uncompressed_png(w as u32, h as u32, &raw_scanlines);
    fs::write(path, png_bytes)
}

fn encode_uncompressed_png(width: u32, height: u32, scanlines: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(scanlines.len() + 128);
    out.extend_from_slice(&[137, 80, 78, 71, 13, 10, 26, 10]);

    let mut ihdr = Vec::with_capacity(13);
    ihdr.extend_from_slice(&width.to_be_bytes());
    ihdr.extend_from_slice(&height.to_be_bytes());
    ihdr.push(8); // bit depth
    ihdr.push(6); // RGBA
    ihdr.push(0); // compression
    ihdr.push(0); // filter
    ihdr.push(0); // interlace
    write_chunk(&mut out, b"IHDR", &ihdr);

    // Zlib stream with stored (uncompressed) deflate blocks
    let mut zlib = Vec::with_capacity(scanlines.len() + 64);
    zlib.push(0x78);
    zlib.push(0x01);

    let max_block = 65535usize;
    let mut pos = 0usize;
    while pos < scanlines.len() {
        let end = (pos + max_block).min(scanlines.len());
        let is_last = end == scanlines.len();
        let len = (end - pos) as u16;
        let nlen = !len;
        zlib.push(if is_last { 1 } else { 0 });
        zlib.extend_from_slice(&len.to_le_bytes());
        zlib.extend_from_slice(&nlen.to_le_bytes());
        zlib.extend_from_slice(&scanlines[pos..end]);
        pos = end;
    }

    let adler = adler32(scanlines);
    zlib.extend_from_slice(&adler.to_be_bytes());
    write_chunk(&mut out, b"IDAT", &zlib);
    write_chunk(&mut out, b"IEND", &[]);
    out
}

fn write_chunk(out: &mut Vec<u8>, chunk_type: &[u8; 4], data: &[u8]) {
    out.extend_from_slice(&(data.len() as u32).to_be_bytes());
    out.extend_from_slice(chunk_type);
    out.extend_from_slice(data);
    let mut crc_buf = Vec::with_capacity(4 + data.len());
    crc_buf.extend_from_slice(chunk_type);
    crc_buf.extend_from_slice(data);
    out.extend_from_slice(&crc32(&crc_buf).to_be_bytes());
}

fn crc32(data: &[u8]) -> u32 {
    let mut crc = 0xFFFF_FFFFu32;
    for &b in data {
        crc ^= b as u32;
        for _ in 0..8 {
            let mask = (crc & 1).wrapping_neg();
            crc = (crc >> 1) ^ (0xEDB8_8320 & mask);
        }
    }
    !crc
}

fn adler32(data: &[u8]) -> u32 {
    let mut a = 1u32;
    let mut b = 0u32;
    for &byte in data {
        a = (a + byte as u32) % 65521;
        b = (b + a) % 65521;
    }
    (b << 16) | a
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encodes_valid_png_signature_and_chunks() {
        let img = egui::ColorImage::filled([2, 2], egui::Color32::from_rgb(10, 20, 30));
        let mut scan = Vec::new();
        for _ in 0..2 {
            scan.push(0);
            for _ in 0..2 {
                scan.extend_from_slice(&[10, 20, 30, 255]);
            }
        }
        let png = encode_uncompressed_png(img.width() as u32, img.height() as u32, &scan);
        assert_eq!(&png[0..8], &[137, 80, 78, 71, 13, 10, 26, 10]);
        assert_eq!(&png[12..16], b"IHDR");
        assert_eq!(&png[png.len() - 8..png.len() - 4], b"IEND");
    }
}
