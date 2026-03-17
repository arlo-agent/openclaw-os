//! Generative ambient background — ultra-smooth glowing orbs with subtle flow.
//!
//! Uses many concentric filled circles with tiny alpha steps to simulate
//! smooth radial gradients. No visible rings, no hard edges.

use crate::theme::{OpenClawPalette, ThemeMode};
use iced::mouse;
use iced::widget::canvas::{self, Cache, Canvas, Frame, Geometry, Path};
use iced::{Color, Element, Length, Point, Rectangle, Renderer, Theme};
use noise::{NoiseFn, OpenSimplex};
use rand::Rng;

const ORB_COUNT: usize = 5;
const PARTICLE_COUNT: usize = 50;
/// More steps = smoother gradient. 40 is smooth enough to be invisible.
const ORB_GRADIENT_STEPS: usize = 40;

struct Orb {
    base_x: f32,
    base_y: f32,
    radius: f32,
    color_phase: f32,
}

struct Particle {
    x: f32,
    y: f32,
    vx: f32,
    vy: f32,
    size: f32,
    phase: f32,
    color_phase: f32,
}

pub struct ParticleField {
    noise: OpenSimplex,
    time: f64,
    cache: Cache,
    orbs: Vec<Orb>,
    particles: Vec<Particle>,
    theme_mode: ThemeMode,
    initialized: bool,
}

impl ParticleField {
    pub fn new() -> Self {
        Self {
            noise: OpenSimplex::new(42),
            time: 0.0,
            cache: Cache::new(),
            orbs: Vec::new(),
            particles: Vec::new(),
            theme_mode: ThemeMode::Dark,
            initialized: false,
        }
    }

    pub fn set_theme_mode(&mut self, mode: ThemeMode) {
        self.theme_mode = mode;
    }

    pub fn tick(&mut self, width: f32, height: f32) {
        if width < 1.0 || height < 1.0 {
            return;
        }

        self.time += 0.004;

        if !self.initialized {
            self.initialized = true;
            let mut rng = rand::thread_rng();

            self.orbs = (0..ORB_COUNT)
                .map(|i| {
                    let t = i as f32 / ORB_COUNT as f32;
                    Orb {
                        base_x: t * width * 0.7 + width * 0.15,
                        base_y: rng.gen_range(height * 0.2..height * 0.8),
                        radius: rng.gen_range(width * 0.1..width * 0.18),
                        color_phase: t,
                    }
                })
                .collect();

            self.particles = (0..PARTICLE_COUNT)
                .map(|_| Particle {
                    x: rng.gen_range(0.0..width),
                    y: rng.gen_range(0.0..height),
                    vx: rng.gen_range(-0.06..0.06),
                    vy: rng.gen_range(-0.04..0.04),
                    size: rng.gen_range(1.5..3.5),
                    phase: rng.gen_range(0.0..std::f32::consts::TAU),
                    color_phase: rng.gen_range(0.0..1.0),
                })
                .collect();
        }

        // Move particles
        let w = width;
        let h = height;
        for p in &mut self.particles {
            let nx = p.x as f64 * 0.003;
            let ny = p.y as f64 * 0.003;
            let angle = self.noise.get([nx, ny, self.time * 0.2]) as f32 * std::f32::consts::TAU;
            p.vx += angle.cos() * 0.004;
            p.vy += angle.sin() * 0.004;
            p.vx *= 0.99;
            p.vy *= 0.99;
            p.x += p.vx;
            p.y += p.vy;
            p.phase += 0.008;
            if p.x < -20.0 { p.x += w + 40.0; }
            if p.x > w + 20.0 { p.x -= w + 40.0; }
            if p.y < -20.0 { p.y += h + 40.0; }
            if p.y > h + 20.0 { p.y -= h + 40.0; }
        }

        self.cache.clear();
    }

    pub fn view(&self) -> Element<'_, ()> {
        Canvas::new(self)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }

    /// Blend between coral and cyan.
    /// Dark mode: color at given alpha over dark bg.
    /// Light mode: vivid color that fades toward white (not dark).
    fn blend_color(&self, t: f32, alpha: f32, _palette: &OpenClawPalette) -> Color {
        let t = t.clamp(0.0, 1.0);
        let is_dark = matches!(self.theme_mode, ThemeMode::Dark);

        if is_dark {
            // Dark mode: bright coral ↔ bright cyan, with alpha transparency
            let coral = (1.0, 0.3, 0.3);   // vivid coral
            let cyan = (0.0, 0.9, 0.8);    // vivid cyan
            Color::from_rgba(
                coral.0 * (1.0 - t) + cyan.0 * t,
                coral.1 * (1.0 - t) + cyan.1 * t,
                coral.2 * (1.0 - t) + cyan.2 * t,
                alpha,
            )
        } else {
            // Light mode: vivid saturated colors, standard alpha blending
            // Over a white bg, alpha-blended color naturally fades to white at edges
            let coral = (0.93, 0.22, 0.28);  // vivid coral
            let cyan = (0.0, 0.65, 0.60);    // vivid teal
            Color::from_rgba(
                coral.0 * (1.0 - t) + cyan.0 * t,
                coral.1 * (1.0 - t) + cyan.1 * t,
                coral.2 * (1.0 - t) + cyan.2 * t,
                alpha,
            )
        }
    }
}

impl canvas::Program<()> for ParticleField {
    type State = ();

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &Renderer,
        _theme: &Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<Geometry> {
        let geometry = self.cache.draw(renderer, bounds.size(), |frame: &mut Frame| {
            let w = bounds.width;
            let h = bounds.height;
            let palette = OpenClawPalette::from_mode(self.theme_mode);
            let is_dark = matches!(self.theme_mode, ThemeMode::Dark);

            let breath = ((self.time * 0.15).sin() * 0.5 + 0.5) as f32;

            // === Large soft orbs ===
            // Draw from outermost to innermost (painter's algorithm).
            // Each ring is slightly smaller and slightly more opaque.
            // With 40 steps the rings are invisible — looks like a smooth gradient.
            for (i, orb) in self.orbs.iter().enumerate() {
                let seed = i as f64 * 7.3;
                let dx = self.noise.get([seed, self.time * 0.12, 0.0]) as f32;
                let dy = self.noise.get([seed + 100.0, self.time * 0.1, 0.0]) as f32;
                let cx = orb.base_x + dx * w * 0.06;
                let cy = orb.base_y + dy * h * 0.06;
                let ct = (orb.color_phase + self.time as f32 * 0.008) % 1.0;

                // Peak alpha at center of the orb
                let peak_alpha = if is_dark {
                    0.10 * (0.75 + breath * 0.25)
                } else {
                    // Light mode: needs higher alpha since colors compete with white bg
                    0.25 * (0.75 + breath * 0.25)
                };

                // Draw rings from outside in
                for step in 0..ORB_GRADIENT_STEPS {
                    // t goes from 1.0 (outermost) to ~0.0 (center)
                    let t = 1.0 - (step as f32 / ORB_GRADIENT_STEPS as f32);
                    let r = orb.radius * t;
                    if r < 0.5 { continue; }

                    // Smooth cubic falloff: more transparent at edges, opaque at center
                    // Each ring adds a tiny bit of alpha on top of previous rings
                    let ring_alpha = peak_alpha / ORB_GRADIENT_STEPS as f32;
                    // Weight inner rings slightly more
                    let weighted_alpha = ring_alpha * (1.0 - t * t * 0.5);

                    if weighted_alpha < 0.0005 { continue; }

                    let circle = Path::circle(Point::new(cx, cy), r);
                    frame.fill(&circle, self.blend_color(ct, weighted_alpha, &palette));
                }
            }

            // === Soft particles ===
            for p in &self.particles {
                let pulse = (p.phase.sin() * 0.5 + 0.5) * 0.5;
                let base_alpha = if is_dark { 0.12 } else { 0.35 };
                let alpha = base_alpha * (0.5 + pulse);
                let ct = (p.color_phase + self.time as f32 * 0.01) % 1.0;

                // 3-layer glow for smoothness
                let glow_outer = Path::circle(Point::new(p.x, p.y), p.size * 5.0);
                frame.fill(&glow_outer, self.blend_color(ct, alpha * 0.03, &palette));

                let glow_mid = Path::circle(Point::new(p.x, p.y), p.size * 2.5);
                frame.fill(&glow_mid, self.blend_color(ct, alpha * 0.08, &palette));

                let glow_inner = Path::circle(Point::new(p.x, p.y), p.size * 1.5);
                frame.fill(&glow_inner, self.blend_color(ct, alpha * 0.2, &palette));

                // Bright core
                let core = Path::circle(Point::new(p.x, p.y), p.size * 0.7);
                frame.fill(&core, self.blend_color(ct, alpha * 0.6, &palette));
            }
        });

        vec![geometry]
    }
}
