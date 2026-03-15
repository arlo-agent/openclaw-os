//! Generative aurora background — organic flow fields with simplex noise.
//!
//! Creates layered aurora-like color bands using coral and cyan brand colors,
//! driven by multi-octave simplex noise. Think northern lights through frosted glass.

use crate::theme::{OpenClawPalette, ThemeMode};
use iced::mouse;
use iced::widget::canvas::{self, Cache, Canvas, Frame, Geometry, Path, Stroke};
use iced::{Color, Element, Length, Point, Rectangle, Renderer, Theme};
use noise::{NoiseFn, OpenSimplex};

/// Number of horizontal and vertical grid divisions for the flow field
const GRID_COLS: usize = 24;
const GRID_ROWS: usize = 16;

/// Number of flowing curves (aurora bands)
const AURORA_BANDS: usize = 7;
/// Points per aurora band curve
const BAND_POINTS: usize = 60;

/// Floating particles that drift through the flow field
const DRIFT_PARTICLES: usize = 120;

pub struct ParticleField {
    noise: OpenSimplex,
    time: f64,
    cache: Cache,
    /// Flow field angles (GRID_ROWS x GRID_COLS)
    flow_field: Vec<Vec<f32>>,
    /// Drift particles: (x, y, speed_factor, size, age)
    drifters: Vec<(f32, f32, f32, f32, f32)>,
    theme_mode: ThemeMode,
}

impl ParticleField {
    pub fn new() -> Self {
        Self {
            noise: OpenSimplex::new(42),
            time: 0.0,
            cache: Cache::new(),
            flow_field: vec![vec![0.0; GRID_COLS]; GRID_ROWS],
            drifters: Vec::new(),
            theme_mode: ThemeMode::Dark,
        }
    }

    pub fn set_theme_mode(&mut self, mode: ThemeMode) {
        self.theme_mode = mode;
    }

    pub fn tick(&mut self, width: f32, height: f32) {
        self.time += 0.008; // Slow evolution

        // Update flow field with multi-octave noise
        for row in 0..GRID_ROWS {
            for col in 0..GRID_COLS {
                let nx = col as f64 * 0.15;
                let ny = row as f64 * 0.15;

                // Layer 1: large-scale flow
                let n1 = self.noise.get([nx * 0.5, ny * 0.5, self.time * 0.3]);
                // Layer 2: medium detail
                let n2 = self.noise.get([nx * 1.2, ny * 1.2, self.time * 0.5 + 100.0]);
                // Layer 3: fine turbulence
                let n3 = self.noise.get([nx * 2.5, ny * 2.5, self.time * 0.7 + 200.0]);

                let angle = (n1 * 0.6 + n2 * 0.3 + n3 * 0.1) as f32 * std::f32::consts::TAU;
                self.flow_field[row][col] = angle;
            }
        }

        // Initialize drifters if needed
        if self.drifters.is_empty() && width > 1.0 && height > 1.0 {
            let mut rng = rand::thread_rng();
            use rand::Rng;
            self.drifters = (0..DRIFT_PARTICLES)
                .map(|_| {
                    (
                        rng.gen_range(0.0..width),
                        rng.gen_range(0.0..height),
                        rng.gen_range(0.3..1.2),  // speed factor
                        rng.gen_range(1.0..3.5),   // size
                        rng.gen_range(0.0..1.0),   // age/phase
                    )
                })
                .collect();
        }

        // Update drifters based on flow field
        for d in &mut self.drifters {
            let col = ((d.0 / width.max(1.0)) * GRID_COLS as f32).min(GRID_COLS as f32 - 1.0).max(0.0) as usize;
            let row = ((d.1 / height.max(1.0)) * GRID_ROWS as f32).min(GRID_ROWS as f32 - 1.0).max(0.0) as usize;
            let angle = self.flow_field[row][col];
            let speed = 0.4 * d.2;
            d.0 += angle.cos() * speed;
            d.1 += angle.sin() * speed;
            d.4 += 0.003; // age

            // Wrap
            if d.0 < 0.0 { d.0 += width; }
            if d.0 > width { d.0 -= width; }
            if d.1 < 0.0 { d.1 += height; }
            if d.1 > height { d.1 -= height; }
        }

        self.cache.clear();
    }

    pub fn view(&self) -> Element<'_, ()> {
        Canvas::new(self)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }

    /// Sample the noise field to get an aurora intensity at a point
    fn aurora_intensity(&self, x: f64, y: f64, band_offset: f64) -> f64 {
        let n1 = self.noise.get([x * 0.003, y * 0.008 + band_offset, self.time * 0.2]);
        let n2 = self.noise.get([x * 0.006 + 50.0, y * 0.012 + band_offset, self.time * 0.35 + 50.0]);
        let n3 = self.noise.get([x * 0.015 + 100.0, y * 0.02 + band_offset, self.time * 0.5 + 100.0]);
        let raw = n1 * 0.5 + n2 * 0.35 + n3 * 0.15;
        // Sharpen into band shapes
        let shaped = (raw * 2.5).max(0.0).min(1.0);
        // Breathing pulse
        let breath = (self.time * 0.15).sin() * 0.12 + 0.88;
        shaped * breath
    }

    /// Interpolate between coral and cyan based on t (0=coral, 1=cyan)
    fn aurora_color(&self, t: f32, intensity: f32, palette: &OpenClawPalette) -> Color {
        let coral = palette.coral_bright;
        let cyan = palette.cyan_bright;
        // Smooth interpolation
        let t = t.clamp(0.0, 1.0);
        let r = coral.r * (1.0 - t) + cyan.r * t;
        let g = coral.g * (1.0 - t) + cyan.g * t;
        let b = coral.b * (1.0 - t) + cyan.b * t;

        let base_alpha = match self.theme_mode {
            ThemeMode::Dark => 0.18,
            ThemeMode::Light => 0.08,
        };
        Color::from_rgba(r, g, b, intensity * base_alpha)
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

            // === Layer 1: Aurora bands (background gradient curves) ===
            for band_idx in 0..AURORA_BANDS {
                let band_offset = band_idx as f64 * 3.7;
                let color_phase = (band_idx as f32 / AURORA_BANDS as f32
                    + (self.time as f32 * 0.05))
                    % 1.0;

                // Draw as horizontal strips with varying height
                let strip_h = h / AURORA_BANDS as f32;
                let base_y = band_idx as f32 * strip_h;

                // Sample multiple points across the band to create curves
                let segments = 40;
                for seg in 0..segments {
                    let x0 = (seg as f32 / segments as f32) * w;
                    let x1 = ((seg + 1) as f32 / segments as f32) * w;

                    let intensity0 = self.aurora_intensity(
                        x0 as f64,
                        base_y as f64,
                        band_offset,
                    ) as f32;
                    let intensity1 = self.aurora_intensity(
                        x1 as f64,
                        base_y as f64,
                        band_offset,
                    ) as f32;

                    let avg_intensity = (intensity0 + intensity1) * 0.5;
                    if avg_intensity < 0.02 {
                        continue;
                    }

                    // Noise-displaced y positions for organic feel
                    let ny0 = self.noise.get([
                        x0 as f64 * 0.005,
                        band_offset,
                        self.time * 0.25,
                    ]) as f32;
                    let ny1 = self.noise.get([
                        x1 as f64 * 0.005,
                        band_offset,
                        self.time * 0.25,
                    ]) as f32;

                    let y0 = base_y + ny0 * strip_h * 0.6;
                    let y1 = base_y + ny1 * strip_h * 0.6;

                    let color = self.aurora_color(color_phase, avg_intensity, &palette);

                    // Draw wide strokes to create band effect
                    let band_width = strip_h * 0.8 * avg_intensity;
                    let line = Path::line(
                        Point::new(x0, y0),
                        Point::new(x1, y1),
                    );
                    frame.stroke(
                        &line,
                        Stroke::default()
                            .with_color(color)
                            .with_width(band_width.max(2.0)),
                    );

                    // Secondary glow layer (wider, more transparent)
                    let glow_color = self.aurora_color(
                        color_phase,
                        avg_intensity * 0.4,
                        &palette,
                    );
                    frame.stroke(
                        &line,
                        Stroke::default()
                            .with_color(glow_color)
                            .with_width(band_width * 2.5),
                    );
                }
            }

            // === Layer 2: Flow field curves ===
            // Draw a subset of visible flow lines for texture
            let flow_lines = 12;
            for i in 0..flow_lines {
                let start_y = (i as f32 / flow_lines as f32) * h;
                let phase = (i as f64 * 2.3 + self.time * 0.3).sin() as f32 * 0.5 + 0.5;

                let mut points: Vec<Point> = Vec::with_capacity(BAND_POINTS);
                let mut cx = 0.0f32;
                let mut cy = start_y;

                for _ in 0..BAND_POINTS {
                    points.push(Point::new(cx, cy));
                    let col = ((cx / w.max(1.0)) * GRID_COLS as f32)
                        .min(GRID_COLS as f32 - 1.0)
                        .max(0.0) as usize;
                    let row = ((cy / h.max(1.0)) * GRID_ROWS as f32)
                        .min(GRID_ROWS as f32 - 1.0)
                        .max(0.0) as usize;
                    let angle = self.flow_field[row][col];
                    cx += angle.cos() * (w / BAND_POINTS as f32) * 1.2;
                    cy += angle.sin() * 8.0;
                    cy = cy.clamp(0.0, h);
                    if cx > w {
                        break;
                    }
                }

                // Draw connected curve segments
                let alpha_base = match self.theme_mode {
                    ThemeMode::Dark => 0.06,
                    ThemeMode::Light => 0.03,
                };
                for pair in points.windows(2) {
                    let p0 = pair[0];
                    let p1 = pair[1];
                    let t = p0.x / w.max(1.0);
                    let color = self.aurora_color(
                        (phase + t * 0.3) % 1.0,
                        alpha_base / 0.18_f32.max(0.01), // normalize
                        &palette,
                    );
                    // Override alpha directly
                    let line = Path::line(p0, p1);
                    frame.stroke(
                        &line,
                        Stroke::default()
                            .with_color(Color::from_rgba(color.r, color.g, color.b, alpha_base))
                            .with_width(1.5),
                    );
                }
            }

            // === Layer 3: Drift particles (foreground) ===
            for d in &self.drifters {
                // Pulsing alpha based on age
                let pulse = ((d.4 * std::f32::consts::TAU * 2.0).sin() * 0.5 + 0.5) * 0.4;
                let alpha = match self.theme_mode {
                    ThemeMode::Dark => 0.1 + pulse,
                    ThemeMode::Light => 0.05 + pulse * 0.5,
                };
                // Color based on position
                let color_t = (d.0 / w.max(1.0) + d.1 / h.max(1.0)) * 0.5;
                let color_t = (color_t + self.time as f32 * 0.02) % 1.0;
                let coral = palette.coral_bright;
                let cyan = palette.cyan_bright;
                let r = coral.r * (1.0 - color_t) + cyan.r * color_t;
                let g = coral.g * (1.0 - color_t) + cyan.g * color_t;
                let b = coral.b * (1.0 - color_t) + cyan.b * color_t;

                let circle = Path::circle(Point::new(d.0, d.1), d.3);
                frame.fill(&circle, Color::from_rgba(r, g, b, alpha));

                // Subtle glow
                let glow = Path::circle(Point::new(d.0, d.1), d.3 * 3.0);
                frame.fill(&glow, Color::from_rgba(r, g, b, alpha * 0.15));
            }

            // === Layer 4: Subtle vignette overlay ===
            // Darken edges for depth (dark mode) or lighten (light mode)
            let vignette_steps = 8;
            for step in 0..vignette_steps {
                let t = step as f32 / vignette_steps as f32;
                let inset = t * w.min(h) * 0.25;
                let alpha = match self.theme_mode {
                    ThemeMode::Dark => (1.0 - t) * 0.15,
                    ThemeMode::Light => (1.0 - t) * 0.05,
                };
                // Draw border rectangles for vignette effect
                let top = Path::line(Point::new(inset, inset), Point::new(w - inset, inset));
                let bottom = Path::line(
                    Point::new(inset, h - inset),
                    Point::new(w - inset, h - inset),
                );
                let left = Path::line(Point::new(inset, inset), Point::new(inset, h - inset));
                let right = Path::line(
                    Point::new(w - inset, inset),
                    Point::new(w - inset, h - inset),
                );

                let vignette_color = match self.theme_mode {
                    ThemeMode::Dark => Color::from_rgba(0.0, 0.0, 0.0, alpha),
                    ThemeMode::Light => Color::from_rgba(0.9, 0.92, 0.95, alpha),
                };
                let stroke_width = w.min(h) * 0.25 / vignette_steps as f32;

                for edge in [&top, &bottom, &left, &right] {
                    frame.stroke(
                        edge,
                        Stroke::default()
                            .with_color(vignette_color)
                            .with_width(stroke_width),
                    );
                }
            }
        });

        vec![geometry]
    }
}
