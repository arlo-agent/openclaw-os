//! Embedded terminal — interactive command shell inside WM windows.

use crate::theme::{self, OpenClawPalette};
use crate::window_manager::{WindowContentMessage, WindowId, WindowManagerMessage};
use iced::widget::{column, container, row, scrollable, text, text_input, Space};
use iced::{Alignment, Color, Element, Length, Padding};

pub struct TerminalLine {
    pub text: String,
    pub is_command: bool,
}

pub struct TerminalState {
    pub input: String,
    pub output_lines: Vec<TerminalLine>,
    pub working_dir: String,
    pub history: Vec<String>,
    pub history_index: Option<usize>,
    pub scrollback_limit: usize,
    pub should_close: bool,
}

impl TerminalState {
    pub fn new() -> Self {
        let working_dir = std::env::var("HOME").unwrap_or_else(|_| "/home/openclaw".to_string());
        Self {
            input: String::new(),
            output_lines: vec![TerminalLine {
                text: "OpenClaw OS Terminal — Type 'help' for commands".to_string(),
                is_command: false,
            }],
            working_dir,
            history: Vec::new(),
            history_index: None,
            scrollback_limit: 1000,
            should_close: false,
        }
    }

    pub fn execute_command(&mut self) {
        let cmd = self.input.trim().to_string();
        if cmd.is_empty() {
            return;
        }

        // Add command line to output
        self.output_lines.push(TerminalLine {
            text: cmd.clone(),
            is_command: true,
        });

        // Push to history
        self.history.push(cmd.clone());
        self.history_index = None;

        // Clear input
        self.input.clear();

        // Parse and execute
        let parts: Vec<&str> = cmd.split_whitespace().collect();
        match parts.first().map(|s| *s) {
            Some("clear") => {
                self.output_lines.clear();
                return;
            }
            Some("exit") => {
                self.should_close = true;
                return;
            }
            Some("help") => {
                let help = [
                    "Available commands:",
                    "  cd <dir>    — Change directory",
                    "  clear       — Clear terminal",
                    "  help        — Show this help",
                    "  exit        — Close terminal window",
                    "  <any>       — Runs via /bin/sh",
                ];
                for line in help {
                    self.output_lines.push(TerminalLine {
                        text: line.to_string(),
                        is_command: false,
                    });
                }
            }
            Some("cd") => {
                let target = parts.get(1).copied().unwrap_or("~");
                let target = if target == "~" {
                    std::env::var("HOME").unwrap_or_else(|_| "/home/openclaw".to_string())
                } else if target.starts_with('~') {
                    let home =
                        std::env::var("HOME").unwrap_or_else(|_| "/home/openclaw".to_string());
                    format!("{}{}", home, &target[1..])
                } else if target.starts_with('/') {
                    target.to_string()
                } else {
                    format!("{}/{}", self.working_dir, target)
                };

                let path = std::path::Path::new(&target);
                if path.is_dir() {
                    // Canonicalize to resolve .. and .
                    match path.canonicalize() {
                        Ok(canon) => {
                            self.working_dir = canon.to_string_lossy().to_string();
                        }
                        Err(e) => {
                            self.output_lines.push(TerminalLine {
                                text: format!("cd: {}", e),
                                is_command: false,
                            });
                        }
                    }
                } else {
                    self.output_lines.push(TerminalLine {
                        text: format!("cd: {}: No such directory", target),
                        is_command: false,
                    });
                }
            }
            _ => {
                // Run via sh
                match std::process::Command::new("sh")
                    .arg("-c")
                    .arg(&cmd)
                    .current_dir(&self.working_dir)
                    .output()
                {
                    Ok(output) => {
                        let stdout = String::from_utf8_lossy(&output.stdout);
                        let stderr = String::from_utf8_lossy(&output.stderr);
                        for line in stdout.lines() {
                            self.output_lines.push(TerminalLine {
                                text: line.to_string(),
                                is_command: false,
                            });
                        }
                        for line in stderr.lines() {
                            self.output_lines.push(TerminalLine {
                                text: line.to_string(),
                                is_command: false,
                            });
                        }
                    }
                    Err(e) => {
                        self.output_lines.push(TerminalLine {
                            text: format!("error: {}", e),
                            is_command: false,
                        });
                    }
                }
            }
        }

        // Blank separator line
        self.output_lines.push(TerminalLine {
            text: String::new(),
            is_command: false,
        });

        // Trim to scrollback limit
        if self.output_lines.len() > self.scrollback_limit {
            let excess = self.output_lines.len() - self.scrollback_limit;
            self.output_lines.drain(..excess);
        }
    }
}

pub fn view_terminal<'a>(
    state: &'a TerminalState,
    window_id: WindowId,
    palette: &OpenClawPalette,
) -> Element<'a, WindowManagerMessage> {
    let p = *palette;

    // --- Output area ---
    let mut output_col = column![].spacing(1).width(Length::Fill);

    for line in &state.output_lines {
        if line.text.is_empty() {
            output_col = output_col.push(Space::with_height(4));
        } else if line.is_command {
            let prompt = text("$ ")
                .size(theme::FONT_CAPTION)
                .color(p.coral_bright);
            let cmd_text = text(line.text.as_str())
                .size(theme::FONT_CAPTION)
                .color(p.coral_bright);
            output_col = output_col.push(row![prompt, cmd_text]);
        } else {
            let line_text = text(line.text.as_str())
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
