use eframe::egui;
use std::collections::HashMap;

#[derive(Clone, Copy, Debug)]
pub struct SpringParams {
    /// Time in seconds for the spring to roughly reach its target (Apple HIG response).
    pub response: f32,
    /// Damping ratio: 1.0 = critically damped (smooth settle, no bounce); <1.0 = subtle tactile bounce.
    pub damping_ratio: f32,
    /// If true, snapped in Reduce Motion mode (spatial movement). If false, quick gentle fade remains.
    pub spatial: bool,
}

impl SpringParams {
    /// Critically damped UI transition (sidebar capsule, segmented tabs, screen switch).
    pub const SNAPPY: Self = Self {
        response: 0.21,
        damping_ratio: 1.0,
        spatial: true,
    };

    /// Fast micro-interaction (hover glow, press feedback, button tint).
    pub const MICRO: Self = Self {
        response: 0.14,
        damping_ratio: 1.0,
        spatial: false,
    };

    /// Tactile spring with slight organic settle (checkbox pop, HUD toast).
    pub const TACTILE: Self = Self {
        response: 0.25,
        damping_ratio: 0.80,
        spatial: true,
    };

    /// Opacity cross-fade (kept gently animated even when spatial motion is reduced).
    pub const FADE: Self = Self {
        response: 0.16,
        damping_ratio: 1.0,
        spatial: false,
    };
}

#[derive(Clone, Copy, Debug)]
struct SpringState {
    value: f32,
    velocity: f32,
}

pub struct Motion {
    springs: HashMap<egui::Id, SpringState>,
    dt: f32,
    reduce_motion: bool,
    any_moving: bool,
    last_section: Option<usize>,
}

impl Default for Motion {
    fn default() -> Self {
        Self {
            springs: HashMap::new(),
            dt: 1.0 / 60.0,
            reduce_motion: false,
            any_moving: false,
            last_section: None,
        }
    }
}

impl Motion {
    pub fn begin_frame(&mut self, ctx: &egui::Context, reduce_motion: bool) {
        let raw_dt = ctx.input(|i| i.unstable_dt);
        self.dt = if raw_dt.is_finite() && raw_dt > 0.0005 {
            raw_dt.clamp(0.001, 0.04)
        } else {
            1.0 / 60.0
        };
        self.reduce_motion = reduce_motion;
        self.any_moving = false;
    }

    /// Steps or initializes an interruptible spring toward `target`.
    pub fn spring(&mut self, id: egui::Id, target: f32, params: SpringParams) -> f32 {
        let entry = self.springs.entry(id).or_insert(SpringState {
            value: target,
            velocity: 0.0,
        });

        if self.reduce_motion && params.spatial {
            entry.value = target;
            entry.velocity = 0.0;
            return target;
        }

        let response = if self.reduce_motion {
            params.response.min(0.10)
        } else {
            params.response.max(0.05)
        };

        let diff = (target - entry.value).abs();
        let threshold = if target.abs() > 10.0 { 0.08 } else { 0.0015 };
        let vel_threshold = if target.abs() > 10.0 { 0.25 } else { 0.004 };

        if diff < threshold && entry.velocity.abs() < vel_threshold {
            entry.value = target;
            entry.velocity = 0.0;
            return target;
        }

        // Substep integration for consistent curves at 60Hz–240Hz
        let omega = std::f32::consts::TAU / response;
        let stiffness = omega * omega;
        let damping = 2.0 * params.damping_ratio * omega;

        let substeps = if self.dt > 0.012 { 2 } else { 1 };
        let sub_dt = self.dt / substeps as f32;
        for _ in 0..substeps {
            let accel = stiffness * (target - entry.value) - damping * entry.velocity;
            entry.velocity += accel * sub_dt;
            entry.value += entry.velocity * sub_dt;
        }

        if (target - entry.value).abs() < threshold && entry.velocity.abs() < vel_threshold {
            entry.value = target;
            entry.velocity = 0.0;
        } else {
            self.any_moving = true;
        }

        entry.value
    }

    /// Animates a value starting from `initial` when first seen, then moving toward `target`.
    pub fn spring_from(
        &mut self,
        id: egui::Id,
        initial: f32,
        target: f32,
        params: SpringParams,
    ) -> f32 {
        self.springs.entry(id).or_insert(SpringState {
            value: initial,
            velocity: 0.0,
        });
        self.spring(id, target, params)
    }

    pub fn reset(&mut self, id: egui::Id, value: f32) {
        self.springs.insert(
            id,
            SpringState {
                value,
                velocity: 0.0,
            },
        );
    }

    /// Tracks section switches so each arrival plays one subtle cross-fade/settle.
    pub fn section_transition(&mut self, section: usize) -> (f32, f32) {
        let id_fade = egui::Id::new("section_fade_active");
        let id_slide = egui::Id::new("section_slide_active");
        let switched = self.last_section.is_some_and(|prev| prev != section);
        if switched {
            self.reset(id_fade, 0.15);
            self.reset(id_slide, 10.0);
        }
        self.last_section = Some(section);

        let alpha = self
            .spring_from(id_fade, 1.0, 1.0, SpringParams::FADE)
            .clamp(0.0, 1.0);
        let y_offset = if self.reduce_motion {
            0.0
        } else {
            self.spring_from(id_slide, 0.0, 0.0, SpringParams::SNAPPY)
        };
        (alpha, y_offset)
    }

    pub fn is_reduced(&self) -> bool {
        self.reduce_motion
    }

    pub fn is_moving(&self) -> bool {
        self.any_moving
    }

    pub fn end_frame(&self, ctx: &egui::Context) {
        if self.any_moving {
            ctx.request_repaint();
        }
    }
}

#[cfg(target_os = "windows")]
pub fn system_reduce_motion() -> bool {
    const SPI_GETCLIENTAREAANIMATION: u32 = 0x1042;
    #[link(name = "user32")]
    unsafe extern "system" {
        fn SystemParametersInfoW(
            ui_action: u32,
            ui_param: u32,
            pv_param: *mut std::ffi::c_void,
            f_win_ini: u32,
        ) -> i32;
    }
    let mut enabled: i32 = 1;
    let ok = unsafe {
        SystemParametersInfoW(
            SPI_GETCLIENTAREAANIMATION,
            0,
            (&mut enabled as *mut i32).cast(),
            0,
        )
    };
    ok != 0 && enabled == 0
}

#[cfg(not(target_os = "windows"))]
pub fn system_reduce_motion() -> bool {
    false
}
