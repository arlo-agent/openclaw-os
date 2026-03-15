//! Generative ambient background — soft glowing orbs with subtle flow.
//!
//! Renders a calm, organic background with:
//! - Large soft gradient orbs that drift slowly (aurora-like)
//! - Small floating particles with soft glow
//! - No hard lines, no grids, no visible edges
//! - Everything uses filled circles with alpha for smoothness

use crate::theme::{OpenClawPalette, ThemeMode};
use iced::mouse;
use iced::widget::canvas::{self, Cache, Canvas, Frame, Geometry, Path};
use iced::{Color, Element, Length, Point, Rectangle, Renderer, Theme};
use noise::{NoiseFn, OpenSimplex};
use rand::Rng;

/// Large ambient orbs (aurora-like glow)
const ORB_COUNT: usize = 6;
/// Small floating particles
const PARTICLE_COUNT: usize = 60;

struct Orb {
    x: f32,
    y: f32,
    base_x: f32,
    base_y: f32,
    radius: f32,
    color_phase: f32,  // 0.0 = coral, 1.0 = cyan
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

        // Initialize once
        if !self.initialized {
            self.initialized = true;
            let mut rng = rand::thread_rng();

            // Create large ambient orbs spread across the screen
            self.orbs = (0..ORB_COUNT)
                .map(|i| {
                    let t = i as f32 / ORB_COUNT as f32;
                    let bx = t * width * 0.8 + width * 0.1;
                    let by = rng.gen_range(height * 0.15..height * 0.85);
                    Orb {
                        x: bx,
                        y: by,
                        base_x: bx,
                        base_y: by,
                        radius: rng.gen_range(width * 0.12..width * 0.25),
                        color_phase: t,
                    }
                })
                .collect();

            // Create small particles
            self.particles = (0..PARTICLE_COUNT)
                .map(|_| Particle {
                    x: rng.gen_range(0.0..width),
                    y: rng.gen_range(0.0..height),
                    vx: rng.gen_range(-0.08..0.08),
                    vy: rng.gen_range(-0.06..0.06),
                    size: rng.gen_range(1.0..3.0),
                    phase: rng.gen_range(0.0..std::f32::consts::TAU),
                    color_phase: rng.gen_range(0.0..1.0),
                })
                .collect();
        }

        let w = width;
        let h = height;

        // Drift orbs with noise
        for (i, orb) in self.orbs.iter_mut().enumerate() {
            let seed = i as f64 * 7.3;
            let nx = self.noise.get([seed, self.time * 0.15, 0.0]) as f32;
            let ny = self.noise.get([seed + 100.0, self.time * 0.12, 0.0]) as f32;
            orb.x = orb.base_x + nx * w * 0.08;
            orb.y = orb.base_y + ny * h * 0.08;
            // Slowly shift color
            orb.color_phase = (orb.color_phase + 0.0003) % 1.0;
        }

        // Move particles with gentle noise-based flow
        for p in &mut self.particles {
            let nx = p.x as f64 * 0.003;
            let ny = p.y as f64 * 0.003;
            let angle = self.noise.get([nx, ny, self.time * 0.2]) as f32 * std::f32::consts::TAU;

            p.vx += angle.cos() * 0.005;
            p.vy += angle.sin() * 0.005;
            p.vx *= 0.99;
            p.vy *= 0.99;

            p.x += p.vx;
            p.y += p.vy;
            p.phase += 0.008;

            // Wrap edges smoothly
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

    fn blend_color(&self, t: f32, alpha: f32, palette: &OpenClawPalette) -> Color {
        let t = t.clamp(0.0, 1.0);
        let c = palette.coral_bright;
        let n = palette.cyan_bright;
        Color::from_rgba(
            c.r * (1.0 - t) + n.r * t,
            c.g * (1.0 - t) + n.g * t,
            c.b * (1.0 - t) + n.b * t,
            alpha,
        )
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
            let palette = OpenClawPalette::from_mode(self.theme_mode);
            let is_dark = matches!(self.theme_mode, ThemeMode::Dark);

            // Breathing pulse
            let breath = ((self.time * 0.15).sin() * 0.5 + 0.5) as f32;

            // === Layer 1: Large soft ambient orbs ===
            // These create the aurora-like color atmosphere
            for orb in &self.orbs {
                let base_alpha = if is_dark { 0.06 } else { 0.04 };
                let alpha = base_alpha * (0.7 + breath * 0.3);

                // Draw multiple concentric circles with decreasing alpha (soft glow)
                let steps = 8;
                for step in (0..steps).rev() {
                    let t = step as f32 / steps as f32;
                    let r = orb.radius * (0.3 + t * 0.7);
                    let a = alpha * (1.0 - t) * (1.0 - t); // quadratic falloff
                    if a < 0.001 { continue; }

                    let circle = Path::circle(Point::new(orb.x, orb.y), r);
                    frame.fill(&circle, self.blend_color(orb.color_phase, a, &palette));
                }
            }

            // === Layer 2: Floating particles with glow ===
            for p in &self.particles {
                let pulse = (p.phase.sin() * 0.5 + 0.5) * 0.6;
                let base_alpha = if is_dark { 0.15 } else { 0.1 };
                let alpha = base_alpha * (0.4 + pulse);
                let ct = (p.color_phase + self.time as f32 * 0.01) % 1.0;

                // Outer glow (3 layers for smoothness)
                let glow3 = Path::circle(Point::new(p.x, p.y), p.size * 6.0);
                frame.fill(&glow3, self.blend_color(ct, alpha * 0.04, &palette));

                let glow2 = Path::circle(Point::new(p.x, p.y), p.size * 3.5);
                frame.fill(&glow2, self.blend_color(ct, alpha * 0.1, &palette));

                let glow1 = Path::circle(Point::new(p.x, p.y), p.size * 2.0);
                frame.fill(&glow1, self.blend_color(ct, alpha * 0.25, &palette));

                // Core
                let core = Path::circle(Point::new(p.x, p.y), p.size);
                frame.fill(&core, self.blend_color(ct, alpha, &palette));
            }
        });

        vec![geometry]
    }
}
