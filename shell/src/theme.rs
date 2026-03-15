use iced::Color;

// Core palette from ui-design.md
pub const BACKGROUND: Color = Color::from_rgb(
    0x0A as f32 / 255.0,
    0x0A as f32 / 255.0,
    0x0F as f32 / 255.0,
);

pub const SURFACE: Color = Color::from_rgb(
    0x1A as f32 / 255.0,
    0x1A as f32 / 255.0,
    0x2E as f32 / 255.0,
);

pub const GLASS: Color = Color::from_rgba(1.0, 1.0, 1.0, 0.08);

pub const PRIMARY: Color = Color::from_rgb(
    0x6C as f32 / 255.0,
    0x63 as f32 / 255.0,
    0xFF as f32 / 255.0,
);

pub const TEXT_PRIMARY: Color = Color::WHITE;

pub const TEXT_SECONDARY: Color = Color::from_rgb(
    0x88 as f32 / 255.0,
    0x88 as f32 / 255.0,
    0xAA as f32 / 255.0,
);

pub const SUCCESS: Color = Color::from_rgb(
    0x4A as f32 / 255.0,
    0xDE as f32 / 255.0,
    0x80 as f32 / 255.0,
);

pub const ALERT: Color = Color::from_rgb(
    0xFB as f32 / 255.0,
    0x92 as f32 / 255.0,
    0x3C as f32 / 255.0,
);

pub const ERROR: Color = Color::from_rgb(
    0xF8 as f32 / 255.0,
    0x71 as f32 / 255.0,
    0x71 as f32 / 255.0,
);

// Spacing (8px grid)
pub const GRID: f32 = 8.0;
pub const BORDER_RADIUS: f32 = 16.0;

// Font sizes
pub const FONT_DISPLAY: f32 = 72.0;
pub const FONT_HEADING: f32 = 24.0;
pub const FONT_BODY: f32 = 16.0;
pub const FONT_CAPTION: f32 = 12.0;
