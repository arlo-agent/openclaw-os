//! Embedded terminal — real PTY-based shell inside WM windows.

use crate::theme::{self, OpenClawPalette};
use crate::window_manager::{WindowContentMessage, WindowId, WindowManagerMessage};
use iced::widget::{column, container, row, scrollable, text, text_input, Space};
use iced::{Alignment, Color, Element, Length, Padding};
use portable_pty::{CommandBuilder, PtySize, native_pty_system};
use std::io::{Read as IoRead, Write as IoWrite};
use std::sync::mpsc;

pub struct TerminalState {
    pub output_text: String,
    pub input: String,
    pub scrollback_limit: usize,
    pub should_close: bool,
    writer: Option<Box<dyn IoWrite + Send>>,
    output_rx: Option<mpsc::Receiver<String>>,
}

impl TerminalState {
    pub fn new() -> Self {
        let pty_system = native_pty_system();

        let pair = pty_system
            .openpty(PtySize {
                rows: 24,
                cols: 80,
                pixel_width: 0,
                pixel_height: 0,
            })
            .expect("Failed to open PTY");

        let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());

        let mut cmd = CommandBuilder::new("bash");
        cmd.env("TERM", "xterm-256color");
        cmd.env("PS1", "\\w$ ");
        cmd.cwd(&home);

        // Try bash, fall back to sh
        let child = match pair.slave.spawn_command(cmd) {
            Ok(child) => child,
            Err(_) => {
                let mut cmd = CommandBuilder::new("sh");
                cmd.env("TERM", "xterm-256color");
                cmd.cwd(&home);
                pair.slave.spawn_command(cmd)
                    .expect("Failed to spawn shell")
            }
        };

        // Drop slave — we only use the master side
        drop(pair.slave);

        let writer = pair.master.take_writer().expect("Failed to take PTY writer");

        let (tx, rx) = mpsc::channel::<String>();

        // Reader thread
        let mut reader = pair.master.try_clone_reader().expect("Failed to clone PTY reader");
        std::thread::spawn(move || {
            let mut buf = [0u8; 4096];
            loop {
                match reader.read(&mut buf) {
                    Ok(0) => break,
                    Ok(n) => {
                        let chunk = String::from_utf8_lossy(&buf[..n]).to_string();
                        if tx.send(chunk).is_err() {
                            break;
                        }
                    }
                    Err(_) => break,
                }
            }
        });

        Self {
            output_text: String::new(),
            input: String::new(),
            scrollback_limit: 100_000,
            should_close: false,
            writer: Some(writer),
            output_rx: Some(rx),
        }
    }

    pub fn tick(&mut self) {
        if let Some(rx) = &self.output_rx {
            let mut got_data = false;
            while let Ok(chunk) = rx.try_recv() {
                self.output_text.push_str(&chunk);
                got_data = true;
            }
            if got_data {
                self.output_text = strip_ansi(&self.output_text);
                // Trim scrollback
                if self.output_text.len() > self.scrollback_limit {
                    let excess = self.output_text.len() - self.scrollback_limit;
                    // Find a newline near the cut point to avoid splitting a line
                    let cut = if let Some(nl) = self.output_text[excess..].find('\n') {
                        excess + nl + 1
                    } else {
                        excess
                    };
                    self.output_text.drain(..cut);
                }
            }
        }
    }

    pub fn send_input(&mut self) {
        let input = std::mem::take(&mut self.input);
        if let Some(writer) = &mut self.writer {
            let data = format!("{}\n", input);
            let _ = writer.write_all(data.as_bytes());
            let _ = writer.flush();
        }
    }

    pub fn send_raw(&mut self, text: &str) {
        if let Some(writer) = &mut self.writer {
            let _ = writer.write_all(text.as_bytes());
            let _ = writer.flush();
        }
    }
}

fn strip_ansi(input: &str) -> String {
    let mut result = String::with_capacity(input.len());
    let mut chars = input.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch == '\x1b' {
            if let Some(&next) = chars.peek() {
                if next == '[' {
                    chars.next();
                    // CSI sequence — read until terminator letter or ~
                    while let Some(&c) = chars.peek() {
                        chars.next();
                        if c.is_ascii_alphabetic() || c == '~' {
                            break;
                        }
                    }
                } else if next == ']' {
                    // OSC sequence — skip until BEL or ST
                    chars.next();
                    while let Some(&c) = chars.peek() {
                        chars.next();
                        if c == '\x07' {
                            break;
                        }
                        if c == '\x1b' {
                            if chars.peek() == Some(&'\\') {
                                chars.next();
                                break;
                            }
                        }
                    }
                } else if next == '(' || next == ')' {
                    // Character set selection — skip 2 chars
                    chars.next();
                    chars.next();
                }
            }
        } else if ch == '\r' {
            // Skip carriage returns
            continue;
        } else {
            result.push(ch);
        }
    }
    result
}

pub fn view_terminal<'a>(
    state: &'a TerminalState,
    window_id: WindowId,
    palette: &OpenClawPalette,
) -> Element<'a, WindowManagerMessage> {
    let p = *palette;

    // --- Output area ---
    let mut output_col = column![].spacing(0).width(Length::Fill);

    for line in state.output_text.split('\n') {
        if line.is_empty() {
            output_col = output_col.push(Space::with_height(4));
        } else {
            let line_text = text(line.to_string())
                .size(theme::FONT_CAPTION)
                .color(p.text_primary);
            output_col = output_col.push(line_text);
        }
    }

    let output_scroll = scrollable(
        container(output_col)
            .padding(Padding::from([theme::GRID, theme::GRID]))
            .width(Length::Fill),
    )
    .height(Length::Fill)
    .width(Length::Fill)
    .anchor_bottom()
    .id(scrollable::Id::new(format!("terminal-scroll-{}", window_id)));

    // --- Input area ---
    let wid = window_id;
    let prompt_text = text("$ ")
        .size(theme::FONT_CAPTION)
        .color(p.coral_bright);

    let input = text_input("", &state.input)
        .on_input(move |val| {
            WindowManagerMessage::WindowContent(wid, WindowContentMessage::TerminalInput(val))
        })
        .on_submit(WindowManagerMessage::WindowContent(
            wid,
            WindowContentMessage::TerminalSubmit,
        ))
        .padding(Padding::from([4.0, 4.0]))
        .size(theme::FONT_CAPTION)
        .style(move |_theme, _status| text_input::Style {
            background: iced::Background::Color(Color::TRANSPARENT),
            border: iced::Border {
                width: 0.0,
                radius: 0.0.into(),
                color: Color::TRANSPARENT,
            },
            icon: p.text_muted,
            placeholder: p.text_muted,
            value: p.text_primary,
            selection: Color::from_rgba(p.coral_bright.r, p.coral_bright.g, p.coral_bright.b, 0.3),
        });

    let input_row = row![prompt_text, input]
        .align_y(Alignment::Center)
        .width(Length::Fill);

    let input_area = container(input_row)
        .padding(Padding::from([4.0, theme::GRID]))
        .width(Length::Fill)
        .style(move |_: &_| container::Style {
            background: Some(iced::Background::Color(Color::from_rgba(
                p.bg_deep.r,
                p.bg_deep.g,
                p.bg_deep.b,
                0.9,
            ))),
            border: iced::Border {
                width: 1.0,
                radius: 0.0.into(),
                color: p.border_subtle,
            },
            ..Default::default()
        });

    // --- Full terminal layout ---
    let terminal = column![output_scroll, input_area]
        .width(Length::Fill)
        .height(Length::Fill);

    container(terminal)
        .width(Length::Fill)
        .height(Length::Fill)
        .style(move |_: &_| container::Style {
            background: Some(iced::Background::Color(p.bg_deep)),
            ..Default::default()
        })
        .into()
}
