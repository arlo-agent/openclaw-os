use iced::Color;

/// Theme mode — Dark (default) or Light
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThemeMode {
    Dark,
    Light,
}

impl ThemeMode {
    pub fn toggle(self) -> Self {
        match self {
            ThemeMode::Dark => ThemeMode::Light,
            ThemeMode::Light => ThemeMode::Dark,
        }
    }
}

impl Default for ThemeMode {
    fn default() -> Self {
        ThemeMode::Dark
    }
}

/// The OpenClaw brand palette, derived from openclaw.ai CSS variables.
#[derive(Debug, Clone, Copy)]
pub struct OpenClawPalette {
    pub bg_deep: Color,
    pub bg_surface: Color,
    pub bg_elevated: Color,
    pub coral_bright: Color,
    pub coral_mid: Color,
    pub coral_dark: Color,
    pub cyan_bright: Color,
    pub cyan_mid: Color,
    pub text_primary: Color,
    pub text_secondary: Color,
    pub text_muted: Color,
    pub border_subtle: Color,
    pub border_accent: Color,
    pub surface_card: Color,
    pub surface_card_strong: Color,
    pub surface_overlay: Color,
    pub surface_interactive: Color,
    pub shadow_coral_soft: Color,
}

impl OpenClawPalette {
    pub fn from_mode(mode: ThemeMode) -> Self {
        match mode {
            ThemeMode::Dark => Self::dark(),
            ThemeMode::Light => Self::light(),
        }
    }

    pub fn dark() -> Self {
        Self {
            bg_deep: hex(0x05, 0x08, 0x10),
            bg_surface: hex(0x0a, 0x0f, 0x1a),
            bg_elevated: hex(0x11, 0x18, 0x27),
            coral_bright: hex(0xff, 0x4d, 0x4d),
            coral_mid: hex(0xe6, 0x39, 0x46),
            coral_dark: hex(0x99, 0x1b, 0x1b),
            cyan_bright: hex(0x00, 0xe5, 0xcc),
            cyan_mid: hex(0x14, 0xb8, 0xa6),
            text_primary: hex(0xf0, 0xf4, 0xff),
            text_secondary: hex(0x88, 0x92, 0xb0),
            text_muted: hex(0x5a, 0x64, 0x80),
            border_subtle: Color::from_rgba(136.0 / 255.0, 146.0 / 255.0, 176.0 / 255.0, 0.15),
            border_accent: Color::from_rgba(1.0, 77.0 / 255.0, 77.0 / 255.0, 0.3),
            surface_card: Color::from_rgba(10.0 / 255.0, 15.0 / 255.0, 26.0 / 255.0, 0.65),
            surface_card_strong: Color::from_rgba(10.0 / 255.0, 15.0 / 255.0, 26.0 / 255.0, 0.8),
            surface_overlay: Color::from_rgba(0.0, 0.0, 0.0, 0.3),
            surface_interactive: Color::from_rgba(1.0, 1.0, 1.0, 0.1),
            shadow_coral_soft: Color::from_rgba(1.0, 77.0 / 255.0, 77.0 / 255.0, 0.15),
        }
    }

    pub fn light() -> Self {
        Self {
            bg_deep: hex(0xfc, 0xfe, 0xff),
            bg_surface: hex(0xff, 0xff, 0xff),
            bg_elevated: hex(0xf5, 0xf9, 0xff),
            coral_bright: hex(0xef, 0x4b, 0x58),
            coral_mid: hex(0xde, 0x3f, 0x4d),
            coral_dark: hex(0xc4, 0x36, 0x45),
            cyan_bright: hex(0x00, 0x8f, 0x87),
            cyan_mid: hex(0x00, 0x76, 0x6e),
            text_primary: hex(0x0b, 0x12, 0x20),
            text_secondary: hex(0x2e, 0x40, 0x5c),
            text_muted: hex(0x5f, 0x72, 0x90),
            border_subtle: Color::from_rgba(15.0 / 255.0, 23.0 / 255.0, 42.0 / 255.0, 0.16),
            border_accent: Color::from_rgba(239.0 / 255.0, 75.0 / 255.0, 88.0 / 255.0, 0.34),
            surface_card: Color::from_rgba(1.0, 1.0, 1.0, 0.88),
            surface_card_strong: Color::from_rgba(1.0, 1.0, 1.0, 0.95),
            surface_overlay: Color::from_rgba(160.0 / 255.0, 174.0 / 255.0, 194.0 / 255.0, 0.26),
            surface_interactive: Color::from_rgba(15.0 / 255.0, 23.0 / 255.0, 42.0 / 255.0, 0.1),
            shadow_coral_soft: Color::from_rgba(239.0 / 255.0, 75.0 / 255.0, 88.0 / 255.0, 0.15),
        }
    }

    /// For iced's built-in Theme palette
    pub fn to_iced_palette(&self) -> iced::theme::Palette {
        iced::theme::Palette {
            background: self.bg_deep,
            text: self.text_primary,
            primary: self.coral_bright,
            success: self.cyan_bright,
            danger: self.coral_dark,
        }
    }
}

// Helper to build Color from RGB bytes (const-friendly)
const fn hex(r: u8, g: u8, b: u8) -> Color {
    Color::from_rgb(r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0)
}

// Spacing (8px grid)
pub const GRID: f32 = 8.0;
pub const BORDER_RADIUS: f32 = 16.0;

// Font sizes
pub const FONT_DISPLAY: f32 = 72.0;
pub const FONT_HEADING: f32 = 24.0;
pub const FONT_BODY: f32 = 16.0;
pub const FONT_CAPTION: f32 = 12.0;
