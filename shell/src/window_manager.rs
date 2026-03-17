//! In-app window manager — draggable, resizable, z-ordered overlay windows.

use crate::terminal::{self, TerminalState};
use crate::theme::{self, OpenClawPalette, BORDER_RADIUS};
use crate::widgets::glass_card;
use iced::widget::{button, column, container, mouse_area, row, stack, text, Space};
use iced::{Alignment, Color, Element, Length, Padding};
use iced_fonts::{Bootstrap, BOOTSTRAP_FONT};
use std::collections::HashMap;

pub type WindowId = u64;

#[derive(Debug, Clone)]
pub enum WindowContent {
    Terminal,
    // Future: Settings, FileManager, etc.
}

pub struct Window {
    pub id: WindowId,
    pub title: String,
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub min_width: f32,
    pub min_height: f32,
    pub z_order: u32,
    pub content: WindowContent,
}

#[derive(Debug, Clone)]
pub enum WindowManagerMessage {
    // Mouse interaction
    TitleBarPressed(WindowId, f32, f32),
    ResizePressed(WindowId, f32, f32),
    MouseMoved(f32, f32),
    MouseReleased,
    // Window actions
    CloseWindow(WindowId),
    BringToFront(WindowId),
    // Content messages
    WindowContent(WindowId, WindowContentMessage),
}

#[derive(Debug, Clone)]
pub enum WindowContentMessage {
    TerminalInput(String),
    TerminalSubmit,
}

enum DragState {
    None,
    Moving {
        window_id: WindowId,
        offset_x: f32,
        offset_y: f32,
    },
    Resizing {
        window_id: WindowId,
        start_x: f32,
        start_y: f32,
        start_w: f32,
        start_h: f32,
    },
}

pub struct WindowManager {
    windows: Vec<Window>,
    next_id: WindowId,
    drag_state: DragState,
    next_z: u32,
    terminal_states: HashMap<WindowId, TerminalState>,
}

impl WindowManager {
    pub fn new() -> Self {
        Self {
            windows: Vec::new(),
            next_id: 1,
            drag_state: DragState::None,
            next_z: 1,
            terminal_states: HashMap::new(),
        }
    }

    pub fn open_window(
        &mut self,
        title: String,
        content: WindowContent,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
    ) -> WindowId {
        let id = self.next_id;
        self.next_id += 1;
        let z = self.next_z;
        self.next_z += 1;
        if matches!(content, WindowContent::Terminal) {
            self.terminal_states.insert(id, TerminalState::new());
        }
        self.windows.push(Window {
            id,
            title,
            x,
            y,
            width,
            height,
            min_width: 300.0,
            min_height: 200.0,
            z_order: z,
            content,
        });
        id
    }

    pub fn close_window(&mut self, id: WindowId) {
        self.windows.retain(|w| w.id != id);
        self.terminal_states.remove(&id);
    }

    pub fn bring_to_front(&mut self, id: WindowId) {
        let z = self.next_z;
        self.next_z += 1;
        if let Some(w) = self.windows.iter_mut().find(|w| w.id == id) {
            w.z_order = z;
        }
    }

    pub fn has_windows(&self) -> bool {
        !self.windows.is_empty()
    }

    pub fn windows_sorted(&self) -> Vec<&Window> {
        let mut sorted: Vec<&Window> = self.windows.iter().collect();
        sorted.sort_by_key(|w| w.z_order);
        sorted
    }

    pub fn update(&mut self, msg: WindowManagerMessage) {
        match msg {
            WindowManagerMessage::TitleBarPressed(id, mx, my) => {
                self.bring_to_front(id);
                if let Some(w) = self.windows.iter().find(|w| w.id == id) {
                    self.drag_state = DragState::Moving {
                        window_id: id,
                        offset_x: mx - w.x,
                        offset_y: my - w.y,
                    };
                }
            }
            WindowManagerMessage::ResizePressed(id, mx, my) => {
                self.bring_to_front(id);
                if let Some(w) = self.windows.iter().find(|w| w.id == id) {
                    self.drag_state = DragState::Resizing {
                        window_id: id,
                        start_x: mx,
                        start_y: my,
                        start_w: w.width,
                        start_h: w.height,
                    };
                }
            }
            WindowManagerMessage::MouseMoved(mx, my) => match &self.drag_state {
                DragState::Moving {
                    window_id,
                    offset_x,
                    offset_y,
                } => {
                    let wid = *window_id;
                    let ox = *offset_x;
                    let oy = *offset_y;
                    if let Some(w) = self.windows.iter_mut().find(|w| w.id == wid) {
                        w.x = (mx - ox).max(0.0);
                        w.y = (my - oy).max(0.0);
                    }
                }
                DragState::Resizing {
                    window_id,
                    start_x,
                    start_y,
                    start_w,
                    start_h,
                } => {
                    let wid = *window_id;
                    let dx = mx - start_x;
                    let dy = my - start_y;
                    let sw = *start_w;
                    let sh = *start_h;
                    if let Some(w) = self.windows.iter_mut().find(|w| w.id == wid) {
                        w.width = (sw + dx).max(w.min_width);
                        w.height = (sh + dy).max(w.min_height);
                    }
                }
                DragState::None => {}
            },
            WindowManagerMessage::MouseReleased => {
                self.drag_state = DragState::None;
            }
            WindowManagerMessage::CloseWindow(id) => {
                self.close_window(id);
            }
            WindowManagerMessage::BringToFront(id) => {
                self.bring_to_front(id);
            }
            WindowManagerMessage::WindowContent(id, msg) => match msg {
                WindowContentMessage::TerminalInput(val) => {
                    if let Some(state) = self.terminal_states.get_mut(&id) {
                        state.input = val;
                    }
                }
                WindowContentMessage::TerminalSubmit => {
                    if let Some(state) = self.terminal_states.get_mut(&id) {
                        state.execute_command();
                        if state.should_close {
                            self.close_window(id);
                        }
                    }
                }
            },
        }
    }
}

// ---------- View ----------

const TITLE_BAR_HEIGHT: f32 = 32.0;
const RESIZE_HANDLE_SIZE: f32 = 12.0;

fn bicon(i: Bootstrap, size: f32, color: Color) -> iced::widget::Text<'static> {
    text(i.to_string())
        .font(BOOTSTRAP_FONT)
        .size(size)
        .color(color)
}

fn content_icon(content: &WindowContent) -> Bootstrap {
    match content {
        WindowContent::Terminal => Bootstrap::Terminal,
    }
}

fn view_single_window<'a>(
    win: &'a Window,
    palette: &OpenClawPalette,
    terminal_states: &'a HashMap<WindowId, TerminalState>,
) -> Element<'a, WindowManagerMessage> {
    let p = *palette;
    let wid = win.id;

    // --- Title bar ---
    let icon = bicon(content_icon(&win.content), 14.0, p.text_secondary);
    let title_text = text(win.title.clone())
        .size(theme::FONT_CAPTION)
        .color(p.text_primary);
    let close_btn = button(bicon(Bootstrap::XLg, 12.0, p.text_muted))
        .on_press(WindowManagerMessage::CloseWindow(wid))
        .padding(Padding::from([2, 6]))
        .style(button::text);

    let title_content = row![
        Space::with_width(8),
        icon,
        Space::with_width(6),
        title_text,
        Space::with_width(Length::Fill),
        close_btn,
    ]
    .align_y(Alignment::Center)
    .height(TITLE_BAR_HEIGHT)
    .width(Length::Fill);

    let title_bar = container(title_content)
        .width(Length::Fill)
        .style(move |_: &_| container::Style {
            background: Some(iced::Background::Color(p.surface_card_strong)),
            border: iced::Border {
                color: p.border_subtle,
                width: 0.0,
                radius: 0.0.into(),
            },
            ..Default::default()
        });

    // Wrap title bar in mouse_area for drag detection
    let title_bar_interactive: Element<'a, WindowManagerMessage> = mouse_area(title_bar)
        .on_press(WindowManagerMessage::TitleBarPressed(wid, 0.0, 0.0))
        .into();

    // --- Content area ---
    let content_area: Element<'a, WindowManagerMessage> = match &win.content {
        WindowContent::Terminal => {
            if let Some(term_state) = terminal_states.get(&wid) {
                terminal::view_terminal(term_state, wid, &p)
            } else {
                container(
                    text("Terminal not initialized")
                        .size(theme::FONT_BODY)
                        .color(p.text_muted),
                )
                .width(Length::Fill)
                .height(Length::Fill)
                .padding(theme::GRID)
                .style(move |_: &_| container::Style {
                    background: Some(iced::Background::Color(p.bg_deep)),
                    ..Default::default()
                })
                .into()
            }
        }
    };

    // --- Resize handle (bottom-right) ---
    let resize_icon = bicon(Bootstrap::ArrowsAngleExpand, 10.0, p.text_muted);
    let resize_handle = container(resize_icon)
        .width(RESIZE_HANDLE_SIZE)
        .height(RESIZE_HANDLE_SIZE)
        .align_x(Alignment::End)
        .align_y(Alignment::End);

    let resize_interactive: Element<'a, WindowManagerMessage> = mouse_area(resize_handle)
        .on_press(WindowManagerMessage::ResizePressed(wid, 0.0, 0.0))
        .into();

    let resize_row = container(resize_interactive)
        .width(Length::Fill)
        .align_x(Alignment::End);

    // --- Combine into window widget ---
    let window_body = column![title_bar_interactive, content_area, resize_row]
        .width(win.width)
        .height(win.height);

    // Glass card border around the whole window
    let glass = glass_card::glass_container_with_palette(&p);
    let window_container = container(window_body)
        .style(move |_: &_| container::Style {
            background: glass.background,
            border: iced::Border {
                color: p.border_subtle,
                width: 1.0,
                radius: BORDER_RADIUS.into(),
            },
            shadow: glass.shadow,
            text_color: glass.text_color,
        })
        .clip(true);

    // Position the window using padding from top-left
    container(window_container)
        .padding(Padding {
            top: win.y,
            left: win.x,
            bottom: 0.0,
            right: 0.0,
        })
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

pub fn view_windows<'a>(
    wm: &'a WindowManager,
    palette: &OpenClawPalette,
) -> Element<'a, WindowManagerMessage> {
    let sorted = wm.windows_sorted();
    if sorted.is_empty() {
        return Space::new(0, 0).into();
    }

    let mut layers: Vec<Element<'a, WindowManagerMessage>> = Vec::new();
    for win in sorted {
        layers.push(view_single_window(win, palette, &wm.terminal_states));
    }

    // Stack all windows; lowest z first (painter's algorithm)
    let mut s = stack(layers);
    s = s.width(Length::Fill).height(Length::Fill);
    s.into()
}
