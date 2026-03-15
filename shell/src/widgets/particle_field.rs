use iced::mouse;
use iced::widget::canvas::{self, Cache, Canvas, Frame, Geometry, Path};
use iced::{Color, Element, Length, Point, Rectangle, Renderer, Theme};
use rand::Rng;

const PARTICLE_COUNT: usize = 80;

#[derive(Debug, Clone)]
pub struct Particle {
    pub x: f32,
    pub y: f32,
    pub vx: f32,
    pub vy: f32,
    pub radius: f32,
    pub alpha: f32,
}

impl Particle {
    pub fn random(width: f32, height: f32) -> Self {
        let mut rng = rand::thread_rng();
        Self {
            x: rng.gen_range(0.0..width.max(1.0)),
            y: rng.gen_range(0.0..height.max(1.0)),
            vx: rng.gen_range(-0.3..0.3),
            vy: rng.gen_range(-0.2..0.2),
            radius: rng.gen_range(1.0..3.0),
            alpha: rng.gen_range(0.1..0.4),
        }
    }
}

pub struct ParticleField {
    particles: Vec<Particle>,
    cache: Cache,
}

impl ParticleField {
    pub fn new() -> Self {
        Self {
            particles: Vec::new(),
            cache: Cache::new(),
        }
    }

    pub fn tick(&mut self, width: f32, height: f32) {
        // Initialize particles if empty or screen resized
        if self.particles.is_empty() {
            self.particles = (0..PARTICLE_COUNT)
                .map(|_| Particle::random(width, height))
                .collect();
        }

        for p in &mut self.particles {
            p.x += p.vx;
            p.y += p.vy;

            // Wrap around edges
            if p.x < 0.0 {
                p.x = width;
            } else if p.x > width {
                p.x = 0.0;
            }
            if p.y < 0.0 {
                p.y = height;
            } else if p.y > height {
                p.y = 0.0;
            }

            // Subtle alpha oscillation
            p.alpha += rand::thread_rng().gen_range(-0.005..0.005);
            p.alpha = p.alpha.clamp(0.05, 0.5);
        }

        self.cache.clear();
    }

    pub fn view(&self) -> Element<'_, ()> {
        Canvas::new(self)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
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
            for p in &self.particles {
                let circle = Path::circle(Point::new(p.x, p.y), p.radius);
                frame.fill(&circle, Color::from_rgba(1.0, 1.0, 1.0, p.alpha));
            }

            // Draw subtle connection lines between nearby particles
            for i in 0..self.particles.len() {
                for j in (i + 1)..self.particles.len() {
                    let a = &self.particles[i];
                    let b = &self.particles[j];
                    let dx = a.x - b.x;
                    let dy = a.y - b.y;
                    let dist = (dx * dx + dy * dy).sqrt();

                    if dist < 120.0 {
                        let alpha = (1.0 - dist / 120.0) * 0.08;
                        let line = Path::line(
                            Point::new(a.x, a.y),
                            Point::new(b.x, b.y),
                        );
                        frame.stroke(
                            &line,
                            canvas::Stroke::default()
                                .with_color(Color::from_rgba(1.0, 1.0, 1.0, alpha))
                                .with_width(0.5),
                        );
                    }
                }
            }
        });

        vec![geometry]
    }
}
