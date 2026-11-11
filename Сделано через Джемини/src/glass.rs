use crate::model::Data;
use crate::motion::{Motion, SpringParams};
use eframe::egui::{
    self, Color32, CornerRadius, FontFamily, FontId, Pos2, Rect, RichText, Sense, Shape, Stroke,
    StrokeKind, Vec2,
};
use std::collections::HashMap;

#[derive(Clone, Copy, Debug)]
#[allow(dead_code)]
pub struct Theme {
    pub dark: bool,
    pub glass: bool,
    pub reduce_transparency: bool,
    pub canvas_bg: Color32,
    pub content_bg: Color32,
    pub sidebar_bg: Color32,
    pub surface: Color32,
    pub surface_elevated: Color32,
    pub pill_active: Color32,
    pub pill_track: Color32,
    pub ink: Color32,
    pub ink_secondary: Color32,
    pub ink_muted: Color32,
    pub divider: Color32,
    pub accent: Color32,
    pub accent_soft: Color32,
    pub check_warm: Color32,
    pub danger: Color32,
    pub success: Color32,
    pub rim_top: Color32,
    pub rim_outer: Color32,
    pub shadow_color: Color32,
    pub widget_bg: Color32,
}

impl Theme {
    pub fn from_data(d: &Data) -> Self {
        let glass = d.glass_effect && !d.reduce_transparency;
        if d.dark {
            Self {
                dark: true,
                glass,
                reduce_transparency: d.reduce_transparency,
                canvas_bg: if glass {
                    Color32::from_rgba_unmultiplied(22, 23, 27, 215)
                } else {
                    Color32::from_rgb(24, 25, 29)
                },
                content_bg: if glass {
                    Color32::from_rgba_unmultiplied(32, 33, 36, 238)
                } else {
                    Color32::from_rgb(32, 33, 36)
                },
                sidebar_bg: if glass {
                    Color32::from_rgba_unmultiplied(40, 41, 46, 188)
                } else {
                    Color32::from_rgb(40, 41, 44)
                },
                surface: if glass {
                    Color32::from_rgba_unmultiplied(46, 48, 54, 178)
                } else {
                    Color32::from_rgb(44, 46, 51)
                },
                surface_elevated: if glass {
                    Color32::from_rgba_unmultiplied(54, 57, 64, 210)
                } else {
                    Color32::from_rgb(52, 54, 60)
                },
                pill_active: if glass {
                    Color32::from_rgba_unmultiplied(68, 72, 82, 220)
                } else {
                    Color32::from_rgb(62, 65, 73)
                },
                pill_track: if glass {
                    Color32::from_rgba_unmultiplied(255, 255, 255, 16)
                } else {
                    Color32::from_rgb(48, 50, 56)
                },
                ink: Color32::from_rgb(242, 242, 244),
                ink_secondary: Color32::from_rgb(177, 179, 186),
                ink_muted: Color32::from_rgb(134, 138, 148),
                divider: Color32::from_rgba_unmultiplied(255, 255, 255, 24),
                accent: Color32::from_rgb(105, 170, 255),
                accent_soft: Color32::from_rgba_unmultiplied(105, 170, 255, 38),
                check_warm: Color32::from_rgb(230, 188, 86),
                danger: Color32::from_rgb(255, 112, 106),
                success: Color32::from_rgb(92, 206, 134),
                rim_top: Color32::from_rgba_unmultiplied(255, 255, 255, 58),
                rim_outer: Color32::from_rgba_unmultiplied(255, 255, 255, 24),
                shadow_color: Color32::from_rgba_unmultiplied(0, 0, 0, 88),
                widget_bg: if glass {
                    Color32::from_rgba_unmultiplied(36, 38, 44, 202)
                } else {
                    Color32::from_rgb(40, 41, 45)
                },
            }
        } else {
            Self {
                dark: false,
                glass,
                reduce_transparency: d.reduce_transparency,
                canvas_bg: if glass {
                    Color32::from_rgba_unmultiplied(238, 241, 247, 218)
                } else {
                    Color32::from_rgb(241, 242, 244)
                },
                content_bg: if glass {
                    Color32::from_rgba_unmultiplied(251, 251, 252, 242)
                } else {
                    Color32::from_rgb(251, 251, 252)
                },
                sidebar_bg: if glass {
                    Color32::from_rgba_unmultiplied(245, 246, 249, 196)
                } else {
                    Color32::from_rgb(241, 242, 244)
                },
                surface: if glass {
                    Color32::from_rgba_unmultiplied(255, 255, 255, 188)
                } else {
                    Color32::from_rgb(255, 255, 255)
                },
                surface_elevated: if glass {
                    Color32::from_rgba_unmultiplied(255, 255, 255, 224)
                } else {
                    Color32::from_rgb(255, 255, 255)
                },
                pill_active: if glass {
                    Color32::from_rgba_unmultiplied(255, 255, 255, 236)
                } else {
                    Color32::from_rgb(255, 255, 255)
                },
                pill_track: if glass {
                    Color32::from_rgba_unmultiplied(26, 30, 40, 14)
                } else {
                    Color32::from_rgb(232, 234, 238)
                },
                ink: Color32::from_rgb(32, 33, 36),
                ink_secondary: Color32::from_rgb(98, 101, 107),
                ink_muted: Color32::from_rgb(138, 142, 150),
                divider: Color32::from_rgba_unmultiplied(32, 33, 36, 20),
                accent: Color32::from_rgb(0, 104, 217),
                accent_soft: Color32::from_rgba_unmultiplied(0, 104, 217, 26),
                check_warm: Color32::from_rgb(152, 102, 0),
                danger: Color32::from_rgb(206, 48, 42),
                success: Color32::from_rgb(22, 140, 74),
                rim_top: Color32::from_rgba_unmultiplied(255, 255, 255, 230),
                rim_outer: Color32::from_rgba_unmultiplied(26, 30, 40, 22),
                shadow_color: Color32::from_rgba_unmultiplied(20, 26, 40, 24),
                widget_bg: if glass {
                    Color32::from_rgba_unmultiplied(247, 248, 251, 216)
                } else {
                    Color32::from_rgb(245, 246, 248)
                },
            }
        }
    }

    pub fn category_color(&self, cat: &str) -> Color32 {
        match cat {
            "Секция" => {
                if self.dark {
                    Color32::from_rgb(86, 204, 157)
                } else {
                    Color32::from_rgb(16, 134, 92)
                }
            }
            "Личное" => {
                if self.dark {
                    Color32::from_rgb(230, 188, 86)
                } else {
                    Color32::from_rgb(152, 102, 0)
                }
            }
            "Контрольная" => self.danger,
            _ => self.accent,
        }
    }
}

pub fn semibold(text: impl Into<String>, size: f32, color: Color32) -> RichText {
    RichText::new(text)
        .font(FontId::new(size, FontFamily::Name("Semibold".into())))
        .color(color)
}

pub fn apply_egui_visuals(ctx: &egui::Context, theme: &Theme) {
    let mut visuals = if theme.dark {
        egui::Visuals::dark()
    } else {
        egui::Visuals::light()
    };
    visuals.panel_fill = Color32::TRANSPARENT;
    visuals.window_fill = theme.surface_elevated;
    visuals.extreme_bg_color = if theme.dark {
        Color32::from_rgba_unmultiplied(20, 21, 24, 210)
    } else {
        Color32::from_rgba_unmultiplied(255, 255, 255, 225)
    };
    visuals.faint_bg_color = theme.pill_track;
    visuals.override_text_color = Some(theme.ink);
    visuals.selection.bg_fill = theme.accent_soft;
    visuals.selection.stroke = Stroke::new(1.2_f32, theme.accent);

    visuals.widgets.noninteractive.bg_fill = theme.surface;
    visuals.widgets.noninteractive.weak_bg_fill = theme.pill_track;
    visuals.widgets.noninteractive.bg_stroke = Stroke::new(1.0_f32, theme.divider);
    visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0_f32, theme.ink_secondary);
    visuals.widgets.noninteractive.corner_radius = CornerRadius::same(8);

    visuals.widgets.inactive.bg_fill = theme.surface;
    visuals.widgets.inactive.weak_bg_fill = theme.pill_track;
    visuals.widgets.inactive.bg_stroke = Stroke::new(1.0_f32, theme.rim_outer);
    visuals.widgets.inactive.fg_stroke = Stroke::new(1.0_f32, theme.ink);
    visuals.widgets.inactive.corner_radius = CornerRadius::same(8);

    visuals.widgets.hovered.bg_fill = theme.surface_elevated;
    visuals.widgets.hovered.weak_bg_fill = theme.pill_active;
    visuals.widgets.hovered.bg_stroke = Stroke::new(1.0_f32, theme.accent.gamma_multiply(0.55));
    visuals.widgets.hovered.fg_stroke = Stroke::new(1.2_f32, theme.ink);
    visuals.widgets.hovered.corner_radius = CornerRadius::same(8);

    visuals.widgets.active.bg_fill = theme.pill_active;
    visuals.widgets.active.bg_stroke = Stroke::new(1.3_f32, theme.accent);
    visuals.widgets.active.fg_stroke = Stroke::new(1.4_f32, theme.ink);
    visuals.widgets.active.corner_radius = CornerRadius::same(8);

    visuals.window_corner_radius = CornerRadius::same(14);
    visuals.menu_corner_radius = CornerRadius::same(12);
    ctx.set_visuals(visuals);
}

#[derive(Clone, Copy, Debug)]
pub struct GlassSurfaceSpec {
    pub radius: u8,
    pub fill: Color32,
    pub elevated: bool,
    pub hover_t: f32,
}

pub fn glass_shapes(rect: Rect, spec: GlassSurfaceSpec, theme: &Theme) -> Vec<Shape> {
    let mut shapes = Vec::with_capacity(5);
    let radius = CornerRadius::same(spec.radius);

    if spec.elevated {
        let shadow = egui::epaint::Shadow {
            offset: [0, if theme.dark { 6 } else { 4 }],
            blur: if theme.dark { 18 } else { 14 },
            spread: 0,
            color: theme.shadow_color,
        };
        shapes.push(Shape::Rect(shadow.as_shape(rect, radius)));
    }

    let fill = if spec.hover_t > 0.01 {
        lerp_color(spec.fill, theme.surface_elevated, spec.hover_t * 0.55)
    } else {
        spec.fill
    };
    shapes.push(Shape::Rect(egui::epaint::RectShape::filled(
        rect,
        radius,
        fill,
    )));

    // Optical Liquid Glass specular sheen at the top edge
    if theme.glass && rect.height() > 14.0 && rect.width() > 24.0 {
        let inset_x = (spec.radius as f32 + 2.0).min(rect.width() * 0.36);
        if rect.max.x - inset_x > rect.min.x + inset_x + 4.0 {
            let top_glint = Rect::from_min_max(
                Pos2::new(rect.min.x + inset_x, rect.min.y + 1.0),
                Pos2::new(rect.max.x - inset_x, rect.min.y + 1.8),
            );
            let glint_alpha = if theme.dark {
                (28.0 + spec.hover_t * 24.0) as u8
            } else {
                (120.0 + spec.hover_t * 45.0) as u8
            };
            shapes.push(Shape::Rect(egui::epaint::RectShape::filled(
                top_glint,
                CornerRadius::same(1),
                Color32::from_rgba_unmultiplied(255, 255, 255, glint_alpha),
            )));
        }
    }

    let rim_color = if spec.hover_t > 0.01 {
        lerp_color(theme.rim_outer, theme.accent.gamma_multiply(0.45), spec.hover_t)
    } else {
        theme.rim_outer
    };
    shapes.push(Shape::Rect(egui::epaint::RectShape::stroke(
        rect,
        radius,
        Stroke::new(1.0_f32, rim_color),
        StrokeKind::Inside,
    )));

    if theme.glass && rect.width() > 20.0 && rect.height() > 20.0 {
        let inner = rect.shrink(1.0);
        let inner_r = CornerRadius::same(spec.radius.saturating_sub(1));
        let top_rim = if theme.dark {
            Color32::from_rgba_unmultiplied(255, 255, 255, (28.0 + spec.hover_t * 22.0) as u8)
        } else {
            Color32::from_rgba_unmultiplied(255, 255, 255, (140.0 + spec.hover_t * 60.0) as u8)
        };
        shapes.push(Shape::Rect(egui::epaint::RectShape::stroke(
            inner,
            inner_r,
            Stroke::new(0.9_f32, top_rim),
            StrokeKind::Inside,
        )));
    }

    shapes
}

pub fn draw_glass_surface(painter: &egui::Painter, rect: Rect, spec: GlassSurfaceSpec, theme: &Theme) {
    for shape in glass_shapes(rect, spec, theme) {
        painter.add(shape);
    }
}

/// Renders a Liquid Glass container strictly behind its child widgets.
pub fn glass_panel<R>(
    ui: &mut egui::Ui,
    theme: &Theme,
    radius: u8,
    elevated: bool,
    margin: i8,
    add_contents: impl FnOnce(&mut egui::Ui) -> R,
) -> egui::InnerResponse<R> {
    let slot = ui.painter().add(Shape::Noop);
    let inner = egui::Frame::new()
        .inner_margin(egui::Margin::same(margin))
        .show(ui, |ui| add_contents(ui));
    let shapes = glass_shapes(
        inner.response.rect,
        GlassSurfaceSpec {
            radius,
            fill: theme.surface,
            elevated,
            hover_t: 0.0,
        },
        theme,
    );
    ui.painter().set(slot, Shape::Vec(shapes));
    inner
}

pub fn draw_liquid_pill(painter: &egui::Painter, rect: Rect, radius: u8, theme: &Theme) {
    draw_glass_surface(
        painter,
        rect,
        GlassSurfaceSpec {
            radius,
            fill: theme.pill_active,
            elevated: true,
            hover_t: 0.0,
        },
        theme,
    );
}

pub fn lerp_color(a: Color32, b: Color32, t: f32) -> Color32 {
    let t = t.clamp(0.0, 1.0);
    let mix = |x: u8, y: u8| -> u8 { ((x as f32) + ((y as f32) - (x as f32)) * t).round() as u8 };
    Color32::from_rgba_unmultiplied(mix(a.r(), b.r()), mix(a.g(), b.g()), mix(a.b(), b.b()), mix(a.a(), b.a()))
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[allow(dead_code)]
pub enum Icon {
    Sun,
    Lessons,
    Calendar,
    Checklist,
    Book,
    Moon,
    Sliders,
    Plus,
    Edit,
    Trash,
    Pin,
    WindowOpen,
    Grid,
    List,
    ChevronLeft,
    ChevronRight,
    ChevronDown,
    ArrowUp,
    ArrowDown,
    Sparkles,
    Check,
    Undo,
}

pub fn draw_icon(painter: &egui::Painter, center: Pos2, size: f32, icon: Icon, color: Color32, stroke_w: f32) {
    let r = size * 0.5;
    let stroke = Stroke::new(stroke_w, color);
    match icon {
        Icon::Sun => {
            painter.circle_stroke(center, r * 0.42, stroke);
            for k in 0..8 {
                let ang = (k as f32) * std::f32::consts::FRAC_PI_4;
                let p1 = center + Vec2::angled(ang) * (r * 0.66);
                let p2 = center + Vec2::angled(ang) * (r * 0.90);
                painter.line_segment([p1, p2], stroke);
            }
        }
        Icon::Moon => {
            let c1 = center + Vec2::new(-r * 0.06, r * 0.06);
            let r1 = r * 0.78;
            let c2 = center + Vec2::new(r * 0.44, -r * 0.34);
            let r2 = r * 0.66;
            let d_vec = c2 - c1;
            let d = d_vec.length().max(0.001);
            let u = d_vec / d;
            let v = Vec2::new(-u.y, u.x);
            let x = ((r1 * r1 - r2 * r2 + d * d) / (2.0 * d)).clamp(-r1, r1);
            let h = (r1 * r1 - x * x).max(0.0).sqrt();
            let p_top = c1 + u * x - v * h;
            let p_bot = c1 + u * x + v * h;

            let a_bot = (p_bot - c1).y.atan2((p_bot - c1).x);
            let mut a_top = (p_top - c1).y.atan2((p_top - c1).x);
            if a_top <= a_bot {
                a_top += std::f32::consts::TAU;
            }
            let b_top = (p_top - c2).y.atan2((p_top - c2).x);
            let mut b_bot = (p_bot - c2).y.atan2((p_bot - c2).x);
            if b_bot >= b_top {
                b_bot -= std::f32::consts::TAU;
            }

            let mut pts = Vec::with_capacity(24);
            let outer_steps = 14usize;
            for i in 0..=outer_steps {
                let t = i as f32 / outer_steps as f32;
                let ang = a_bot + (a_top - a_bot) * t;
                pts.push(c1 + Vec2::angled(ang) * r1);
            }
            let inner_steps = 10usize;
            for i in 1..inner_steps {
                let t = i as f32 / inner_steps as f32;
                let ang = b_top + (b_bot - b_top) * t;
                pts.push(c2 + Vec2::angled(ang) * r2);
            }
            painter.add(Shape::closed_line(pts, stroke));
        }
        Icon::Calendar => {
            let box_rect = Rect::from_center_size(center + Vec2::new(0.0, r * 0.08), Vec2::splat(size * 0.76));
            painter.rect_stroke(box_rect, CornerRadius::same(3), stroke, StrokeKind::Middle);
            painter.line_segment(
                [
                    Pos2::new(box_rect.min.x, box_rect.min.y + size * 0.22),
                    Pos2::new(box_rect.max.x, box_rect.min.y + size * 0.22),
                ],
                stroke,
            );
            for dx in [-r * 0.34, r * 0.34] {
                painter.line_segment(
                    [
                        Pos2::new(center.x + dx, box_rect.min.y - r * 0.20),
                        Pos2::new(center.x + dx, box_rect.min.y + r * 0.10),
                    ],
                    stroke,
                );
            }
        }
        Icon::Checklist => {
            for idx in 0..3 {
                let y = center.y + (idx as f32 - 1.0) * (r * 0.55);
                painter.circle_filled(Pos2::new(center.x - r * 0.58, y), stroke_w * 0.78, color);
                painter.line_segment(
                    [
                        Pos2::new(center.x - r * 0.22, y),
                        Pos2::new(center.x + r * 0.74, y),
                    ],
                    stroke,
                );
            }
        }
        Icon::Book => {
            let left = Rect::from_min_max(
                center + Vec2::new(-r * 0.78, -r * 0.62),
                center + Vec2::new(0.0, r * 0.64),
            );
            let right = Rect::from_min_max(
                center + Vec2::new(0.0, -r * 0.62),
                center + Vec2::new(r * 0.78, r * 0.64),
            );
            painter.rect_stroke(left, CornerRadius::same(2), stroke, StrokeKind::Middle);
            painter.rect_stroke(right, CornerRadius::same(2), stroke, StrokeKind::Middle);
        }
        Icon::Sliders => {
            for (idx, knob_x) in [-0.28f32, 0.28, -0.10].iter().enumerate() {
                let y = center.y + (idx as f32 - 1.0) * (r * 0.56);
                let kx = center.x + knob_x * r;
                painter.line_segment([Pos2::new(center.x - r * 0.75, y), Pos2::new(kx - r * 0.20, y)], stroke);
                painter.line_segment([Pos2::new(kx + r * 0.20, y), Pos2::new(center.x + r * 0.75, y)], stroke);
                painter.circle_stroke(Pos2::new(kx, y), r * 0.19, stroke);
            }
        }
        Icon::Plus => {
            let arm = r * 0.62;
            painter.line_segment([center - Vec2::new(arm, 0.0), center + Vec2::new(arm, 0.0)], stroke);
            painter.line_segment([center - Vec2::new(0.0, arm), center + Vec2::new(0.0, arm)], stroke);
        }
        Icon::Edit => {
            let p1 = center + Vec2::new(-r * 0.56, r * 0.56);
            let p2 = center + Vec2::new(r * 0.42, -r * 0.56);
            let p3 = center + Vec2::new(r * 0.62, -r * 0.36);
            let p4 = center + Vec2::new(-r * 0.36, r * 0.64);
            painter.add(Shape::closed_line(vec![p1, p2, p3, p4], stroke));
        }
        Icon::Trash => {
            let top = center.y - r * 0.48;
            painter.line_segment([Pos2::new(center.x - r * 0.68, top), Pos2::new(center.x + r * 0.68, top)], stroke);
            painter.line_segment(
                [
                    Pos2::new(center.x - r * 0.24, top - r * 0.22),
                    Pos2::new(center.x + r * 0.24, top - r * 0.22),
                ],
                stroke,
            );
            let body = Rect::from_min_max(
                Pos2::new(center.x - r * 0.48, top),
                Pos2::new(center.x + r * 0.48, center.y + r * 0.68),
            );
            painter.rect_stroke(body, CornerRadius::same(2), stroke, StrokeKind::Middle);
        }
        Icon::Pin => {
            painter.circle_stroke(center + Vec2::new(0.0, -r * 0.25), r * 0.36, stroke);
            painter.line_segment([center + Vec2::new(0.0, r * 0.12), center + Vec2::new(0.0, r * 0.78)], stroke);
        }
        Icon::WindowOpen => {
            let box_r = Rect::from_center_size(center, Vec2::splat(size * 0.76));
            painter.rect_stroke(box_r, CornerRadius::same(3), stroke, StrokeKind::Middle);
            painter.line_segment(
                [center + Vec2::new(-r * 0.20, r * 0.20), center + Vec2::new(r * 0.32, -r * 0.32)],
                stroke,
            );
            painter.line_segment(
                [center + Vec2::new(-r * 0.02, -r * 0.32), center + Vec2::new(r * 0.32, -r * 0.32)],
                stroke,
            );
            painter.line_segment(
                [center + Vec2::new(r * 0.32, -r * 0.32), center + Vec2::new(r * 0.32, r * 0.02)],
                stroke,
            );
        }
        Icon::Grid => {
            for dx in [-0.36f32, 0.36] {
                for dy in [-0.36f32, 0.36] {
                    let cell = Rect::from_center_size(center + Vec2::new(dx * r, dy * r), Vec2::splat(r * 0.52));
                    painter.rect_stroke(cell, CornerRadius::same(2), stroke, StrokeKind::Middle);
                }
            }
        }
        Icon::List => {
            for idx in 0..3 {
                let y = center.y + (idx as f32 - 1.0) * (r * 0.55);
                painter.line_segment([Pos2::new(center.x - r * 0.66, y), Pos2::new(center.x + r * 0.66, y)], stroke);
            }
        }
        Icon::ChevronLeft => {
            let p1 = center + Vec2::new(r * 0.26, -r * 0.52);
            let p2 = center + Vec2::new(-r * 0.28, 0.0);
            let p3 = center + Vec2::new(r * 0.26, r * 0.52);
            painter.add(Shape::line(vec![p1, p2, p3], stroke));
        }
        Icon::ChevronRight => {
            let p1 = center + Vec2::new(-r * 0.24, -r * 0.52);
            let p2 = center + Vec2::new(r * 0.28, 0.0);
            let p3 = center + Vec2::new(-r * 0.24, r * 0.52);
            painter.add(Shape::line(vec![p1, p2, p3], stroke));
        }
        Icon::ChevronDown => {
            let p1 = center + Vec2::new(-r * 0.52, -r * 0.22);
            let p2 = center + Vec2::new(0.0, r * 0.28);
            let p3 = center + Vec2::new(r * 0.52, -r * 0.22);
            painter.add(Shape::line(vec![p1, p2, p3], stroke));
        }
        Icon::ArrowUp => {
            painter.line_segment(
                [center + Vec2::new(0.0, r * 0.58), center + Vec2::new(0.0, -r * 0.54)],
                stroke,
            );
            let p1 = center + Vec2::new(-r * 0.42, -r * 0.14);
            let p2 = center + Vec2::new(0.0, -r * 0.56);
            let p3 = center + Vec2::new(r * 0.42, -r * 0.14);
            painter.add(Shape::line(vec![p1, p2, p3], stroke));
        }
        Icon::ArrowDown => {
            painter.line_segment(
                [center + Vec2::new(0.0, -r * 0.58), center + Vec2::new(0.0, r * 0.54)],
                stroke,
            );
            let p1 = center + Vec2::new(-r * 0.42, r * 0.14);
            let p2 = center + Vec2::new(0.0, r * 0.56);
            let p3 = center + Vec2::new(r * 0.42, r * 0.14);
            painter.add(Shape::line(vec![p1, p2, p3], stroke));
        }
        Icon::Lessons => {
            // Crisp notebook / curriculum sheet with spine line and lesson rows
            let sheet = Rect::from_center_size(center, Vec2::new(size * 0.72, size * 0.82));
            painter.rect_stroke(sheet, CornerRadius::same(3), stroke, StrokeKind::Middle);
            let spine_x = sheet.min.x + size * 0.20;
            painter.line_segment(
                [Pos2::new(spine_x, sheet.min.y), Pos2::new(spine_x, sheet.max.y)],
                stroke,
            );
            for k in [-0.26f32, 0.0, 0.26] {
                let y = center.y + k * r;
                painter.line_segment(
                    [
                        Pos2::new(spine_x + size * 0.10, y),
                        Pos2::new(sheet.max.x - size * 0.12, y),
                    ],
                    stroke,
                );
            }
        }
        Icon::Sparkles => {
            let pts = [
                center + Vec2::new(0.0, -r * 0.78),
                center + Vec2::new(r * 0.22, -r * 0.22),
                center + Vec2::new(r * 0.78, 0.0),
                center + Vec2::new(r * 0.22, r * 0.22),
                center + Vec2::new(0.0, r * 0.78),
                center + Vec2::new(-r * 0.22, r * 0.22),
                center + Vec2::new(-r * 0.78, 0.0),
                center + Vec2::new(-r * 0.22, -r * 0.22),
            ];
            painter.add(Shape::closed_line(pts.to_vec(), stroke));
        }
        Icon::Check => {
            let p1 = center + Vec2::new(-r * 0.54, r * 0.02);
            let p2 = center + Vec2::new(-r * 0.14, r * 0.44);
            let p3 = center + Vec2::new(r * 0.58, -r * 0.40);
            painter.add(Shape::line(vec![p1, p2, p3], stroke));
        }
        Icon::Undo => {
            let p1 = center + Vec2::new(-r * 0.60, -r * 0.18);
            let p2 = center + Vec2::new(r * 0.18, -r * 0.18);
            let p3 = center + Vec2::new(r * 0.56, r * 0.18);
            let p4 = center + Vec2::new(r * 0.18, r * 0.54);
            let p5 = center + Vec2::new(-r * 0.30, r * 0.54);
            painter.add(Shape::line(vec![p1, p2, p3, p4, p5], stroke));
            painter.add(Shape::line(
                vec![
                    center + Vec2::new(-r * 0.28, -r * 0.50),
                    p1,
                    center + Vec2::new(-r * 0.28, r * 0.14),
                ],
                stroke,
            ));
        }
    }
}

/// Apple Reminders circular checkbox with spring fill, checkmark stroke progress, and tactile pulse.
pub fn reminders_checkbox(
    ui: &mut egui::Ui,
    motion: &mut Motion,
    id: egui::Id,
    checked: &mut bool,
    theme: &Theme,
) -> bool {
    let hit_size = Vec2::splat(26.0);
    let (rect, response) = ui.allocate_exact_size(hit_size, Sense::click());
    let mut toggled = false;
    if response.clicked() {
        *checked = !*checked;
        toggled = true;
    }

    let check_t = motion.spring(
        id.with("check_t"),
        if *checked { 1.0 } else { 0.0 },
        SpringParams::TACTILE,
    );
    let hover_t = motion.spring(
        id.with("hover"),
        if response.hovered() { 1.0 } else { 0.0 },
        SpringParams::MICRO,
    );
    let press_scale = motion.spring(
        id.with("scale"),
        if response.is_pointer_button_down_on() {
            0.88
        } else if toggled {
            1.12
        } else {
            1.0
        },
        SpringParams::TACTILE,
    );

    if ui.is_rect_visible(rect) {
        let p = ui.painter();
        let radius = 9.5 * press_scale;
        let center = rect.center();

        if hover_t > 0.02 {
            p.circle_filled(
                center,
                radius + 3.5,
                theme.check_warm.gamma_multiply(0.11 * hover_t),
            );
        }

        if check_t > 0.02 {
            p.circle_filled(
                center,
                radius * (0.4 + 0.6 * check_t.min(1.0)),
                theme.check_warm.gamma_multiply(check_t.clamp(0.0, 1.0)),
            );
        }

        let ring_color = lerp_color(
            lerp_color(theme.ink_muted, theme.check_warm, hover_t * 0.6),
            theme.check_warm,
            check_t.clamp(0.0, 1.0),
        );
        p.circle_stroke(center, radius, Stroke::new(1.5_f32, ring_color));

        if check_t > 0.12 {
            let prog = ((check_t - 0.12) / 0.88).clamp(0.0, 1.0);
            let tick_color = if theme.dark {
                Color32::from_rgb(24, 24, 28)
            } else {
                Color32::WHITE
            };
            let a = center + Vec2::new(-4.3, 0.2) * press_scale;
            let b = center + Vec2::new(-1.2, 3.4) * press_scale;
            let c = center + Vec2::new(4.8, -3.2) * press_scale;
            let stroke = Stroke::new(1.85_f32, tick_color.gamma_multiply(prog));
            if prog < 0.45 {
                let k = prog / 0.45;
                p.line_segment([a, a.lerp(b, k)], stroke);
            } else {
                let k = (prog - 0.45) / 0.55;
                p.line_segment([a, b], stroke);
                p.line_segment([b, b.lerp(c, k)], stroke);
            }
        }
    }

    toggled
}

/// Liquid Glass button with smooth hover/press spring feedback and optional vector icon.
pub fn glass_button(
    ui: &mut egui::Ui,
    motion: &mut Motion,
    id: egui::Id,
    label: &str,
    icon: Option<Icon>,
    primary: bool,
    theme: &Theme,
) -> egui::Response {
    let fg = if primary {
        if theme.dark {
            Color32::from_rgb(16, 20, 28)
        } else {
            Color32::WHITE
        }
    } else {
        theme.ink
    };
    let font_id = FontId::new(14.0, FontFamily::Name("Semibold".into()));
    let galley = ui.painter().layout_no_wrap(label.to_owned(), font_id, fg);
    let icon_w = if icon.is_some() { 20.0 } else { 0.0 };
    let pad_x = 14.0;
    let h = 34.0;
    let w = (galley.size().x + icon_w + pad_x * 2.0).max(if label.is_empty() { 34.0 } else { 64.0 });

    let (rect, response) = ui.allocate_exact_size(Vec2::new(w, h), Sense::click());
    let hover_t = motion.spring(
        id.with("hover"),
        if response.hovered() { 1.0 } else { 0.0 },
        SpringParams::MICRO,
    );
    let scale = motion.spring(
        id.with("press"),
        if response.is_pointer_button_down_on() {
            0.97
        } else {
            1.0
        },
        SpringParams::MICRO,
    );

    if ui.is_rect_visible(rect) {
        let draw_rect = Rect::from_center_size(rect.center(), rect.size() * scale);
        let p = ui.painter();
        if primary {
            let base = theme.accent;
            let fill = lerp_color(base, Color32::WHITE, hover_t * 0.10);
            draw_glass_surface(
                p,
                draw_rect,
                GlassSurfaceSpec {
                    radius: 10,
                    fill,
                    elevated: true,
                    hover_t,
                },
                theme,
            );
        } else {
            draw_glass_surface(
                p,
                draw_rect,
                GlassSurfaceSpec {
                    radius: 10,
                    fill: theme.surface,
                    elevated: false,
                    hover_t,
                },
                theme,
            );
        }

        let content_w = galley.size().x + icon_w;
        let mut cursor_x = draw_rect.center().x - content_w * 0.5;
        if let Some(ic) = icon {
            draw_icon(
                p,
                Pos2::new(cursor_x + 7.5, draw_rect.center().y),
                15.0,
                ic,
                fg,
                1.6,
            );
            cursor_x += icon_w;
        }
        p.galley(
            Pos2::new(cursor_x, draw_rect.center().y - galley.size().y * 0.5),
            galley,
            fg,
        );
    }

    response
}

/// Compact icon-only glass pill button.
pub fn icon_button(
    ui: &mut egui::Ui,
    motion: &mut Motion,
    id: egui::Id,
    icon: Icon,
    active: bool,
    tooltip: &str,
    theme: &Theme,
) -> egui::Response {
    let (rect, response) = ui.allocate_exact_size(Vec2::splat(32.0), Sense::click());
    let hover_t = motion.spring(
        id.with("hover"),
        if response.hovered() { 1.0 } else { 0.0 },
        SpringParams::MICRO,
    );

    if ui.is_rect_visible(rect) {
        let p = ui.painter();
        let fill = if active {
            theme.accent_soft
        } else if hover_t > 0.02 {
            theme.surface.gamma_multiply(hover_t)
        } else {
            Color32::TRANSPARENT
        };
        if active || hover_t > 0.02 {
            draw_glass_surface(
                p,
                rect,
                GlassSurfaceSpec {
                    radius: 9,
                    fill,
                    elevated: active,
                    hover_t,
                },
                theme,
            );
        }
        let fg = if active {
            theme.accent
        } else {
            lerp_color(theme.ink_secondary, theme.ink, hover_t)
        };
        draw_icon(p, rect.center(), 15.5, icon, fg, 1.55);
    }

    response.on_hover_text(tooltip)
}

/// macOS Liquid Glass Segmented Control with spring-sliding active capsule.
pub fn segmented_control(
    ui: &mut egui::Ui,
    motion: &mut Motion,
    id: egui::Id,
    selected: &mut usize,
    options: &[&str],
    theme: &Theme,
) -> bool {
    if options.is_empty() {
        return false;
    }
    let count = options.len();
    let pad = 3.0;
    let text_size = if count >= 7 && ui.available_width() < 650.0 {
        12.0
    } else {
        13.5
    };
    let font_id = FontId::new(text_size, FontFamily::Name("Semibold".into()));
    let max_label_w = options
        .iter()
        .map(|s| ui.painter().layout_no_wrap((*s).to_owned(), font_id.clone(), theme.ink).size().x)
        .fold(44.0_f32, f32::max);
    let seg_w = (max_label_w + 22.0).min(168.0);
    let total_w = (seg_w * count as f32 + pad * 2.0).min(ui.available_width());
    let actual_seg_w = (total_w - pad * 2.0) / count as f32;
    let h = 34.0;

    let (outer_rect, _) = ui.allocate_exact_size(Vec2::new(total_w, h), Sense::hover());
    let p = ui.painter();
    p.rect_filled(outer_rect, CornerRadius::same(11), theme.pill_track);
    p.rect_stroke(
        outer_rect,
        CornerRadius::same(11),
        Stroke::new(1.0_f32, theme.divider),
        StrokeKind::Inside,
    );

    let active_idx = (*selected).min(count - 1);
    let anim_idx = motion.spring(id.with("seg_idx"), active_idx as f32, SpringParams::SNAPPY);
    let pill_rect = Rect::from_min_size(
        Pos2::new(outer_rect.min.x + pad + anim_idx * actual_seg_w, outer_rect.min.y + pad),
        Vec2::new(actual_seg_w, h - pad * 2.0),
    );
    draw_liquid_pill(p, pill_rect, 8, theme);

    let mut changed = false;
    for (idx, opt) in options.iter().enumerate() {
        let seg_rect = Rect::from_min_size(
            Pos2::new(outer_rect.min.x + pad + idx as f32 * actual_seg_w, outer_rect.min.y + pad),
            Vec2::new(actual_seg_w, h - pad * 2.0),
        );
        let resp = ui.interact(seg_rect, id.with(("seg", idx)), Sense::click());
        if resp.clicked() && *selected != idx {
            *selected = idx;
            changed = true;
        }
        let is_sel = *selected == idx;
        let fg = if is_sel {
            theme.ink
        } else if resp.hovered() {
            theme.ink
        } else {
            theme.ink_secondary
        };
        let f = if is_sel {
            FontId::new(text_size, FontFamily::Name("Semibold".into()))
        } else {
            FontId::new(text_size, FontFamily::Proportional)
        };
        let galley = p.layout_no_wrap((*opt).to_owned(), f, fg);
        p.galley(
            Pos2::new(
                seg_rect.center().x - galley.size().x * 0.5,
                seg_rect.center().y - galley.size().y * 0.5,
            ),
            galley,
            fg,
        );
    }

    changed
}

/// Apple-inspired spring pill toggle switch with wrapped label text.
pub fn settings_switch(
    ui: &mut egui::Ui,
    motion: &mut Motion,
    id: egui::Id,
    value: &mut bool,
    label: &str,
    theme: &Theme,
) -> bool {
    let mut changed = false;
    let total_w = ui.available_width().max(120.0);
    let switch_w = 38.0;
    let switch_h = 22.0;
    let gap = 12.0;
    let text_w = (total_w - switch_w - gap).max(60.0);

    let row_resp = ui.horizontal_top(|ui| {
        let (sw_rect, sw_resp) =
            ui.allocate_exact_size(Vec2::new(switch_w, switch_h), Sense::click());

        ui.add_space(gap);

        let text_resp = ui.allocate_ui_with_layout(
            Vec2::new(text_w, 0.0),
            egui::Layout::top_down(egui::Align::Min),
            |ui| {
                ui.add_space(2.0);
                ui.add(
                    egui::Label::new(RichText::new(label).size(13.5).color(theme.ink))
                        .wrap()
                        .sense(Sense::click()),
                )
            },
        );

        let clicked = sw_resp.clicked() || text_resp.inner.clicked();
        if clicked {
            *value = !*value;
            changed = true;
        }

        let on_t = motion.spring(
            id.with("on"),
            if *value { 1.0 } else { 0.0 },
            SpringParams::TACTILE,
        );
        let hover_t = motion.spring(
            id.with("hover"),
            if sw_resp.hovered() || text_resp.inner.hovered() {
                1.0
            } else {
                0.0
            },
            SpringParams::MICRO,
        );

        if ui.is_rect_visible(sw_rect) {
            let p = ui.painter();
            let off_track = if theme.dark {
                Color32::from_rgba_unmultiplied(255, 255, 255, 34)
            } else {
                Color32::from_rgba_unmultiplied(26, 30, 40, 32)
            };
            let track_fill = lerp_color(off_track, theme.accent, on_t);
            let track_r = CornerRadius::same(11);
            p.rect_filled(sw_rect, track_r, track_fill);
            let border_col = lerp_color(
                theme.rim_outer,
                theme.accent.gamma_multiply(0.7),
                on_t.max(hover_t * 0.5),
            );
            p.rect_stroke(sw_rect, track_r, Stroke::new(1.0_f32, border_col), StrokeKind::Inside);

            let pad = 3.0;
            let knob_d = switch_h - pad * 2.0;
            let travel = switch_w - pad * 2.0 - knob_d;
            let knob_x = sw_rect.min.x + pad + knob_d * 0.5 + travel * on_t;
            let knob_center = Pos2::new(knob_x, sw_rect.center().y);
            p.circle_filled(
                knob_center + Vec2::new(0.0, 1.0),
                knob_d * 0.5,
                Color32::from_rgba_unmultiplied(0, 0, 0, if theme.dark { 70 } else { 36 }),
            );
            p.circle_filled(knob_center, knob_d * 0.5, Color32::WHITE);
        }
    });
    let _ = row_resp;
    changed
}

/// Truncates a single line of text with an ellipsis so it fits cleanly inside `max_w`.
pub fn truncate_to_width(
    painter: &egui::Painter,
    text: &str,
    font: &FontId,
    max_w: f32,
) -> String {
    if max_w <= 12.0 || text.is_empty() {
        return String::new();
    }
    let full = painter.layout_no_wrap(text.to_owned(), font.clone(), Color32::WHITE);
    if full.size().x <= max_w {
        return text.to_owned();
    }
    let chars: Vec<char> = text.chars().collect();
    let mut lo = 0usize;
    let mut hi = chars.len();
    while lo < hi {
        let mid = (lo + hi).div_ceil(2);
        let mut candidate: String = chars[..mid].iter().collect();
        candidate.push('…');
        let w = painter
            .layout_no_wrap(candidate, font.clone(), Color32::WHITE)
            .size()
            .x;
        if w <= max_w {
            lo = mid;
        } else {
            hi = mid - 1;
        }
    }
    if lo == 0 {
        "…".to_owned()
    } else {
        let mut out: String = chars[..lo].iter().collect();
        out.push('…');
        out
    }
}

// ---------- Windows 11 DWM Integration ----------

#[derive(Default)]
pub struct DwmTracker {
    applied: HashMap<isize, (bool, bool, bool)>,
}

impl DwmTracker {
    pub fn sync_main(&mut self, frame: &eframe::Frame, dark: bool, glass: bool) {
        #[cfg(target_os = "windows")]
        {
            use raw_window_handle::{HasWindowHandle, RawWindowHandle};
            if let Ok(handle) = frame.window_handle() {
                if let RawWindowHandle::Win32(w) = handle.as_raw() {
                    self.apply_hwnd(w.hwnd.get(), dark, glass, false);
                }
            }
        }
        #[cfg(not(target_os = "windows"))]
        let _ = (frame, dark, glass);
    }

    pub fn sync_widget_by_title(&mut self, title: &str, dark: bool, glass: bool) {
        #[cfg(target_os = "windows")]
        {
            if let Some(hwnd) = find_window_by_title(title) {
                self.apply_hwnd(hwnd, dark, glass, true);
            }
        }
        #[cfg(not(target_os = "windows"))]
        let _ = (title, dark, glass);
    }

    #[cfg(target_os = "windows")]
    fn apply_hwnd(&mut self, hwnd: isize, dark: bool, glass: bool, is_widget: bool) {
        let state = (dark, glass, is_widget);
        if self.applied.get(&hwnd) == Some(&state) {
            return;
        }
        apply_win32_dwm(hwnd, dark, glass, is_widget);
        self.applied.insert(hwnd, state);
    }
}

#[cfg(target_os = "windows")]
fn find_window_by_title(title: &str) -> Option<isize> {
    #[link(name = "user32")]
    unsafe extern "system" {
        fn FindWindowW(lp_class_name: *const u16, lp_window_name: *const u16) -> isize;
    }
    let wide: Vec<u16> = title.encode_utf16().chain(std::iter::once(0)).collect();
    let hwnd = unsafe { FindWindowW(std::ptr::null(), wide.as_ptr()) };
    if hwnd != 0 { Some(hwnd) } else { None }
}

#[cfg(target_os = "windows")]
fn apply_win32_dwm(hwnd: isize, dark: bool, glass: bool, is_widget: bool) {
    #[repr(C)]
    struct Margins {
        left: i32,
        right: i32,
        top: i32,
        bottom: i32,
    }
    #[link(name = "dwmapi")]
    unsafe extern "system" {
        fn DwmSetWindowAttribute(
            hwnd: isize,
            dw_attribute: u32,
            pv_attribute: *const std::ffi::c_void,
            cb_attribute: u32,
        ) -> i32;
        fn DwmExtendFrameIntoClientArea(hwnd: isize, p_mar_inset: *const Margins) -> i32;
    }

    const DWMWA_USE_IMMERSIVE_DARK_MODE: u32 = 20;
    const DWMWA_WINDOW_CORNER_PREFERENCE: u32 = 33;
    const DWMWA_SYSTEMBACKDROP_TYPE: u32 = 38;
    const DWMWCP_ROUND: i32 = 2;
    const DWMSBT_DISABLE: i32 = 1;
    const DWMSBT_MAINWINDOW: i32 = 2; // Mica
    const DWMSBT_TRANSIENTWINDOW: i32 = 3; // Acrylic

    let dark_val: i32 = if dark { 1 } else { 0 };
    let corner_val: i32 = DWMWCP_ROUND;
    let backdrop_val: i32 = if !glass {
        DWMSBT_DISABLE
    } else if is_widget {
        DWMSBT_TRANSIENTWINDOW
    } else {
        DWMSBT_MAINWINDOW
    };

    let margins = if glass {
        Margins {
            left: -1,
            right: -1,
            top: -1,
            bottom: -1,
        }
    } else {
        Margins {
            left: 0,
            right: 0,
            top: 0,
            bottom: 0,
        }
    };

    unsafe {
        DwmSetWindowAttribute(
            hwnd,
            DWMWA_USE_IMMERSIVE_DARK_MODE,
            (&dark_val as *const i32).cast(),
            std::mem::size_of::<i32>() as u32,
        );
        DwmSetWindowAttribute(
            hwnd,
            DWMWA_WINDOW_CORNER_PREFERENCE,
            (&corner_val as *const i32).cast(),
            std::mem::size_of::<i32>() as u32,
        );
        DwmExtendFrameIntoClientArea(hwnd, &margins);
        DwmSetWindowAttribute(
            hwnd,
            DWMWA_SYSTEMBACKDROP_TYPE,
            (&backdrop_val as *const i32).cast(),
            std::mem::size_of::<i32>() as u32,
        );
    }
}
