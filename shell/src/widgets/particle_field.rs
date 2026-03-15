//! Generative aurora background — smooth organic flow with simplex noise.
//!
//! Renders layered aurora-like glow using flowing curves and soft particles.
//! No rectangles, no grids — everything is curves and circles.

use crate::theme::{OpenClawPalette, ThemeMode};
use iced::mouse;
use iced::widget::canvas::{self, Cache, Canvas, Frame, Geometry, Path, Stroke};
use iced::{Color, Element, Length, Point, Rectangle, Renderer, Theme};
use noise::{NoiseFn, OpenSimplex};
use rand::Rng;

/// Soft flowing particles
const PARTICLE_COUNT: usize = 100;
/// Number of aurora flow lines
const FLOW_LINES: usize = 18;
/// Points per flow line
const LINE_POINTS: usize = 80;

struct Particle {
    x: f32,
    y: f32,
    vx: f32,
    vy: f32,
    size: f32,
    phase: f32,
}

pub struct ParticleField {
    noise: OpenSimplex,
    time: f64,
    cache: Cache,
    particles: Vec<Particle>,
    theme_mode: ThemeMode,
}

impl ParticleField {
    pub fn new() -> Self {
        Self {
            noise: OpenSimplex::new(42),
            time: 0.0,
            cache: Cache::new(),
            particles: Vec::new(),
            theme_mode: ThemeMode::Dark,
        }
    }

    pub fn set_theme_mode(&mut self, mode: ThemeMode) {
        self.theme_mode = mode;
    }

    pub fn tick(&mut self, width: f32, height: f32) {
        self.time += 0.006;

        // Initialize particles
        if self.particles.is_empty() && width > 1.0 && height > 1.0 {
            let mut rng = rand::thread_rng();
            self.particles = (0..PARTICLE_COUNT)
                .map(|_| Particle {
                    x: rng.gen_range(0.0..width),
                    y: rng.gen_range(0.0..height),
                    vx: rng.gen_range(-0.15..0.15),
                    vy: rng.gen_range(-0.1..0.1),
                    size: rng.gen_range(1.5..4.0),
                    phase: rng.gen_range(0.0..std::f32::consts::TAU),
                })
                .collect();
        }

        // Move particles using noise-based flow
        for p in &mut self.particles {
            let nx = p.x as f64 * 0.002;
            let ny = p.y as f64 * 0.002;
            let angle = self.noise.get([nx, ny, self.time * 0.3]) as f32 * std::f32::consts::TAU;
            
            p.vx += angle.cos() * 0.02;
            p.vy += angle.sin() * 0.02;
            // Damping
            p.vx *= 0.98;
            p.vy *= 0.98;
            
            p.x += p.vx;
            p.y += p.vy;
            p.phase += 0.01;

            // Wrap
            if p.x < -10.0 { p.x += width + 20.0; }
            if p.x > width + 10.0 { p.x -= width + 20.0; }
            if p.y < -10.0 { p.y += height + 20.0; }
            if p.y > height + 10.0 { p.y -= height + 20.0; }
        }

        self.cache.clear();
    }

    pub fn view(&self) -> Element<'_, ()> {
        Canvas::new(self)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }

    /// Interpolate between coral and cyan
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
            let w = bounds.width;
            let h = bounds.height;
            let palette = OpenClawPalette::from_mode(self.theme_mode);

            let is_dark = matches!(self.theme_mode, ThemeMode::Dark);
            let alpha_mult = if is_dark { 1.0 } else { 0.4 };

            // === Layer 1: Smooth flowing aurora curves ===
            // Each line starts from the left and flows across, displaced by noise
            for i in 0..FLOW_LINES {
                let base_y = (i as f32 + 0.5) / FLOW_LINES as f32 * h;
                let line_seed = i as f64 * 3.14;
                let color_t = ((i as f32 / FLOW_LINES as f32) + self.time as f32 * 0.03) % 1.0;

                // Breathing pulse per line
                let breath = ((self.time * 0.12 + line_seed * 0.5).sin() * 0.5 + 0.5) as f32;

                // Build smooth curve points
                let mut points: Vec<Point> = Vec::with_capacity(LINE_POINTS);
                for j in 0..LINE_POINTS {
                    let t = j as f64 / LINE_POINTS as f64;
                    let x = t as f32 * w;

                    // Multi-octave noise for vertical displacement
                    let n1 = self.noise.get([
                        t * 2.0 + line_seed,
                        self.time * 0.15,
                        i as f64 * 0.7,
                    ]) as f32;
                    let n2 = self.noise.get([
                        t * 4.5 + line_seed + 100.0,
                        self.time * 0.25,
                        i as f64 * 0.7 + 50.0,
                    ]) as f32;
                    
                    let displacement = (n1 * 0.65 + n2 * 0.35) * h * 0.15;
                    let y = base_y + displacement;

                    points.push(Point::new(x, y));
                }

                // Draw the curve as connected line segments with varying alpha
                for pair in points.windows(2) {
                    let p0 = pair[0];
                    let p1 = pair[1];
                    let seg_t = p0.x / w;

                    // Fade at edges
                    let edge_fade = (seg_t * 4.0).min(1.0) * ((1.0 - seg_t) * 4.0).min(1.0);

                    let alpha = 0.06 * edge_fade * breath * alpha_mult;
                    if alpha < 0.003 {
                        continue;
                    }

                    let color = self.blend_color(color_t + seg_t * 0.2, alpha, &palette);

                    // Main line
                    let line = Path::line(p0, p1);
                    frame.stroke(
                        &line,
                        Stroke::default()
                            .with_color(color)
                            .with_width(2.0),
                    );

                    // Wider glow beneath
                    let glow_color = self.blend_color(color_t + seg_t * 0.2, alpha * 0.3, &palette);
                    frame.stroke(
                        &line,
                        Stroke::default()
                            .with_color(glow_color)
                            .with_width(12.0),
                    );

                    // Even wider subtle wash
                    let wash_color = self.blend_color(color_t + seg_t * 0.2, alpha * 0.08, &palette);
                    frame.stroke(
                        &line,
                        Stroke::default()
                            .with_color(wash_color)
                            .with_width(40.0),
                    );
                }
            }

            // === Layer 2: Soft glowing particles ===
            for p in &self.particles {
                let pulse = (p.phase.sin() * 0.5 + 0.5) * 0.5;
                let alpha = (0.08 + pulse * 0.15) * alpha_mult;
                let color_t = ((p.x / w + p.y / h) * 0.5 + self.time as f32 * 0.02) % 1.0;

                // Outer glow
                let glow = Path::circle(Point::new(p.x, p.y), p.size * 4.0);
                frame.fill(&glow, self.blend_color(color_t, alpha * 0.12, &palette));

                // Inner bright core
                let core = Path::circle(Point::new(p.x, p.y), p.size);
                frame.fill(&core, self.blend_color(color_t, alpha, &palette));
            }

            // === Layer 3: Subtle connection lines between nearby particles ===
            let connect_dist = 100.0_f32;
            for i in 0..self.particles.len() {
                for j in (i + 1)..self.particles.len() {
                    let a = &self.particles[i];
                    let b = &self.particles[j];
                    let dx = a.x - b.x;
                    let dy = a.y - b.y;
                    let dist_sq = dx * dx + dy * dy;

                    if dist_sq < connect_dist * connect_dist {
                        let dist = dist_sq.sqrt();
                        let alpha = (1.0 - dist / connect_dist) * 0.04 * alpha_mult;
                        let color_t = ((a.x + b.x) / (2.0 * w) + self.time as f32 * 0.02) % 1.0;

                        let line = Path::line(
                            Point::new(a.x, a.y),
                            Point::new(b.x, b.y),
                        );
                        frame.stroke(
                            &line,
                            Stroke::default()
                                .with_color(self.blend_color(color_t, alpha, &palette))
                                .with_width(0.5),
                        );
                    }
                }
            }
        });

        vec![geometry]
    }
}
