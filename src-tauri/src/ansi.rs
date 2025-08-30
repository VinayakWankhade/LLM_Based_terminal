use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Color {
    pub fn new(r: u8, g: u8, b: u8) -> Self {
        Color { r, g, b, a: 255 }
    }

    pub fn black() -> Self { Color::new(0, 0, 0) }
    pub fn red() -> Self { Color::new(255, 0, 0) }
    pub fn green() -> Self { Color::new(0, 255, 0) }
    pub fn yellow() -> Self { Color::new(255, 255, 0) }
    pub fn blue() -> Self { Color::new(0, 0, 255) }
    pub fn magenta() -> Self { Color::new(255, 0, 255) }
    pub fn cyan() -> Self { Color::new(0, 255, 255) }
    pub fn white() -> Self { Color::new(255, 255, 255) }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CharAttributes {
    pub bold: bool,
    pub italic: bool,
    pub underline: bool,
    pub strikethrough: bool,
    pub reverse: bool,
    pub fg_color: Option<Color>,
    pub bg_color: Option<Color>,
}

impl Default for CharAttributes {
    fn default() -> Self {
        CharAttributes {
            bold: false,
            italic: false,
            underline: false,
            strikethrough: false,
            reverse: false,
            fg_color: None,
            bg_color: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CursorPosition {
    pub row: u16,
    pub col: u16,
}

#[derive(Debug, Clone)]
pub enum AnsiCommand {
    // Cursor movement
    CursorUp(u16),
    CursorDown(u16),
    CursorLeft(u16),
    CursorRight(u16),
    CursorPosition(u16, u16),
    CursorHome,
    
    // Screen manipulation
    ClearScreen,
    ClearLine,
    ClearToEndOfLine,
    ClearToBeginningOfLine,
    
    // Text attributes
    SetGraphicsMode(Vec<u8>),
    
    // Scrolling
    ScrollUp(u16),
    ScrollDown(u16),
    
    // Character/Text
    PrintText(String),
    
    // Bell
    Bell,
    
    // Unrecognized escape sequence
    Unknown(String),
}

#[derive(Debug)]
pub struct AnsiParser {
    buffer: String,
    in_escape: bool,
    current_attributes: CharAttributes,
}

impl AnsiParser {
    pub fn new() -> Self {
        AnsiParser {
            buffer: String::new(),
            in_escape: false,
            current_attributes: CharAttributes::default(),
        }
    }

    pub fn parse(&mut self, input: &str) -> Vec<AnsiCommand> {
        let mut commands = Vec::new();
        let mut chars = input.chars().peekable();

        while let Some(ch) = chars.next() {
            match ch {
                '\x1b' => {
                    // Start of escape sequence
                    if !self.buffer.is_empty() {
                        commands.push(AnsiCommand::PrintText(self.buffer.clone()));
                        self.buffer.clear();
                    }
                    self.in_escape = true;
                    self.buffer.push(ch);
                }
                '[' if self.in_escape && self.buffer == "\x1b" => {
                    // CSI (Control Sequence Introducer)
                    self.buffer.push(ch);
                }
                'A'..='Z' | 'a'..='z' if self.in_escape => {
                    // End of escape sequence
                    self.buffer.push(ch);
                    if let Some(command) = self.parse_escape_sequence(&self.buffer) {
                        commands.push(command);
                    }
                    self.buffer.clear();
                    self.in_escape = false;
                }
                _ if self.in_escape => {
                    self.buffer.push(ch);
                }
                '\r' => {
                    // Carriage return - move cursor to beginning of line
                    if !self.buffer.is_empty() {
                        commands.push(AnsiCommand::PrintText(self.buffer.clone()));
                        self.buffer.clear();
                    }
                    commands.push(AnsiCommand::CursorLeft(9999)); // Move to beginning
                }
                '\n' => {
                    // Line feed - move cursor down one line
                    if !self.buffer.is_empty() {
                        commands.push(AnsiCommand::PrintText(self.buffer.clone()));
                        self.buffer.clear();
                    }
                    commands.push(AnsiCommand::CursorDown(1));
                }
                '\x07' => {
                    // Bell character
                    if !self.buffer.is_empty() {
                        commands.push(AnsiCommand::PrintText(self.buffer.clone()));
                        self.buffer.clear();
                    }
                    commands.push(AnsiCommand::Bell);
                }
                _ => {
                    self.buffer.push(ch);
                }
            }
        }

        // If there's remaining text, add it as a print command
        if !self.buffer.is_empty() && !self.in_escape {
            commands.push(AnsiCommand::PrintText(self.buffer.clone()));
            self.buffer.clear();
        }

        commands
    }

    fn parse_escape_sequence(&self, seq: &str) -> Option<AnsiCommand> {
        if seq.len() < 3 {
            return Some(AnsiCommand::Unknown(seq.to_string()));
        }

        if seq.starts_with("\x1b[") {
            // CSI sequence
            let command_char = seq.chars().last()?;
            let params_str = &seq[2..seq.len()-1];
            let params: Vec<u16> = params_str
                .split(';')
                .filter_map(|s| s.parse().ok())
                .collect();

            match command_char {
                'A' => Some(AnsiCommand::CursorUp(params.get(0).copied().unwrap_or(1))),
                'B' => Some(AnsiCommand::CursorDown(params.get(0).copied().unwrap_or(1))),
                'C' => Some(AnsiCommand::CursorRight(params.get(0).copied().unwrap_or(1))),
                'D' => Some(AnsiCommand::CursorLeft(params.get(0).copied().unwrap_or(1))),
                'H' => {
                    let row = params.get(0).copied().unwrap_or(1);
                    let col = params.get(1).copied().unwrap_or(1);
                    Some(AnsiCommand::CursorPosition(row, col))
                }
                'J' => {
                    match params.get(0).copied().unwrap_or(0) {
                        0 => Some(AnsiCommand::ClearToEndOfLine),
                        1 => Some(AnsiCommand::ClearToBeginningOfLine),
                        2 => Some(AnsiCommand::ClearScreen),
                        _ => Some(AnsiCommand::Unknown(seq.to_string())),
                    }
                }
                'K' => Some(AnsiCommand::ClearLine),
                'm' => {
                    let params: Vec<u8> = params_str
                        .split(';')
                        .filter_map(|s| s.parse().ok())
                        .collect();
                    Some(AnsiCommand::SetGraphicsMode(params))
                }
                'S' => Some(AnsiCommand::ScrollUp(params.get(0).copied().unwrap_or(1))),
                'T' => Some(AnsiCommand::ScrollDown(params.get(0).copied().unwrap_or(1))),
                _ => Some(AnsiCommand::Unknown(seq.to_string())),
            }
        } else {
            Some(AnsiCommand::Unknown(seq.to_string()))
        }
    }

    pub fn apply_graphics_mode(&mut self, params: &[u8]) {
        for &param in params {
            match param {
                0 => self.current_attributes = CharAttributes::default(),
                1 => self.current_attributes.bold = true,
                3 => self.current_attributes.italic = true,
                4 => self.current_attributes.underline = true,
                7 => self.current_attributes.reverse = true,
                9 => self.current_attributes.strikethrough = true,
                22 => self.current_attributes.bold = false,
                23 => self.current_attributes.italic = false,
                24 => self.current_attributes.underline = false,
                27 => self.current_attributes.reverse = false,
                29 => self.current_attributes.strikethrough = false,
                30 => self.current_attributes.fg_color = Some(Color::black()),
                31 => self.current_attributes.fg_color = Some(Color::red()),
                32 => self.current_attributes.fg_color = Some(Color::green()),
                33 => self.current_attributes.fg_color = Some(Color::yellow()),
                34 => self.current_attributes.fg_color = Some(Color::blue()),
                35 => self.current_attributes.fg_color = Some(Color::magenta()),
                36 => self.current_attributes.fg_color = Some(Color::cyan()),
                37 => self.current_attributes.fg_color = Some(Color::white()),
                39 => self.current_attributes.fg_color = None,
                40 => self.current_attributes.bg_color = Some(Color::black()),
                41 => self.current_attributes.bg_color = Some(Color::red()),
                42 => self.current_attributes.bg_color = Some(Color::green()),
                43 => self.current_attributes.bg_color = Some(Color::yellow()),
                44 => self.current_attributes.bg_color = Some(Color::blue()),
                45 => self.current_attributes.bg_color = Some(Color::magenta()),
                46 => self.current_attributes.bg_color = Some(Color::cyan()),
                47 => self.current_attributes.bg_color = Some(Color::white()),
                49 => self.current_attributes.bg_color = None,
                _ => {} // Ignore unknown parameters
            }
        }
    }

    pub fn current_attributes(&self) -> &CharAttributes {
        &self.current_attributes
    }
}

impl fmt::Display for Color {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "rgba({}, {}, {}, {})", self.r, self.g, self.b, self.a)
    }
}
