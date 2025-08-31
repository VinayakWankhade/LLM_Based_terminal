use serde::{Deserialize, Serialize};
use std::fmt;
use crate::terminal_types::{TerminalCapabilities, ColorSupport};
use base64::{Engine as _, engine::general_purpose};
use std::collections::HashMap;

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
    CursorNextLine(u16),
    CursorPrevLine(u16),
    CursorColumn(u16),
    CursorSave,
    CursorRestore,
    
    // Cursor styles
    SetCursorStyle(CursorStyle),
    ShowCursor,
    HideCursor,
    
    // Screen manipulation
    ClearScreen,
    ClearLine,
    ClearToEndOfLine,
    ClearToBeginningOfLine,
    ClearFromCursor,
    ClearToCursor,
    
    // Alternate screen
    EnterAlternateScreen,
    ExitAlternateScreen,
    
    // Text attributes
    SetGraphicsMode(Vec<u8>),
    
    // Scrolling
    ScrollUp(u16),
    ScrollDown(u16),
    SetScrollRegion(u16, u16),
    
    // Character/Text
    PrintText(String),
    InsertCharacters(u16),
    DeleteCharacters(u16),
    InsertLines(u16),
    DeleteLines(u16),
    
    // Mouse support
    EnableMouseReporting(MouseReportMode),
    DisableMouseReporting(MouseReportMode),
    
    // Bracketed paste
    EnableBracketedPaste,
    DisableBracketedPaste,
    
    // Focus events
    EnableFocusEvents,
    DisableFocusEvents,
    
    // Window manipulation
    SetWindowTitle(String),
    SetIconTitle(String),
    ResizeWindow(u16, u16),
    MoveWindow(i16, i16),
    MinimizeWindow,
    MaximizeWindow,
    RestoreWindow,
    ReportWindowSize,
    ReportWindowPosition,
    
    // Hyperlinks
    SetHyperlink(String, String), // URL, text
    
    // Images
    DisplayImage(ImageData),
    DisplaySixel(Vec<u8>),
    
    // Synchronized updates
    BeginSynchronizedUpdate,
    EndSynchronizedUpdate,
    
    // Device Control Strings
    DeviceControlString(String),
    
    // Bell variants
    Bell,
    VisualBell,
    
    // Unrecognized escape sequence
    Unknown(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CursorStyle {
    Block,
    Underline,
    Bar,
    BlinkingBlock,
    BlinkingUnderline,
    BlinkingBar,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MouseReportMode {
    X10,
    Normal,
    Button,
    Any,
    SGR,
    URXVT,
}

#[derive(Debug, Clone)]
pub struct ImageData {
    pub format: String,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub data: Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct HyperlinkParams {
    pub id: Option<String>,
    pub url: String,
}

#[derive(Debug)]
pub struct AnsiParser {
    buffer: String,
    in_escape: bool,
    escape_type: EscapeType,
    current_attributes: CharAttributes,
    capabilities: TerminalCapabilities,
    saved_cursor: Option<CursorPosition>,
    hyperlink_stack: Vec<HyperlinkParams>,
    in_synchronized_update: bool,
    osc_params: HashMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum EscapeType {
    None,
    CSI,       // Control Sequence Introducer \e[
    OSC,       // Operating System Command \e]
    DCS,       // Device Control String \e P
    PM,        // Privacy Message \e ^
    APC,       // Application Program Command \e _
    SS2,       // Single Shift Two \e N
    SS3,       // Single Shift Three \e O
}

impl AnsiParser {
    pub fn new() -> Self {
        Self::with_capabilities(TerminalCapabilities::default())
    }

    pub fn with_capabilities(capabilities: TerminalCapabilities) -> Self {
        AnsiParser {
            buffer: String::new(),
            in_escape: false,
            escape_type: EscapeType::None,
            current_attributes: CharAttributes::default(),
            capabilities,
            saved_cursor: None,
            hyperlink_stack: Vec::new(),
            in_synchronized_update: false,
            osc_params: HashMap::new(),
        }
    }

    pub fn parse(&mut self, input: &str) -> Vec<AnsiCommand> {
        let mut commands = Vec::new();
        let mut chars = input.chars().peekable();

        while let Some(ch) = chars.next() {
            match ch {
                '\x1b' => {
                    // Start of escape sequence
                    self.flush_buffer(&mut commands);
                    self.in_escape = true;
                    self.escape_type = EscapeType::None;
                    self.buffer.push(ch);
                }
                '[' if self.in_escape && self.escape_type == EscapeType::None => {
                    // CSI (Control Sequence Introducer)
                    self.escape_type = EscapeType::CSI;
                    self.buffer.push(ch);
                }
                ']' if self.in_escape && self.escape_type == EscapeType::None => {
                    // OSC (Operating System Command)
                    self.escape_type = EscapeType::OSC;
                    self.buffer.push(ch);
                }
                'P' if self.in_escape && self.escape_type == EscapeType::None => {
                    // DCS (Device Control String)
                    self.escape_type = EscapeType::DCS;
                    self.buffer.push(ch);
                }
                '^' if self.in_escape && self.escape_type == EscapeType::None => {
                    // PM (Privacy Message)
                    self.escape_type = EscapeType::PM;
                    self.buffer.push(ch);
                }
                '_' if self.in_escape && self.escape_type == EscapeType::None => {
                    // APC (Application Program Command)
                    self.escape_type = EscapeType::APC;
                    self.buffer.push(ch);
                }
                'N' if self.in_escape && self.escape_type == EscapeType::None => {
                    // SS2 (Single Shift Two)
                    self.escape_type = EscapeType::SS2;
                    self.buffer.push(ch);
                }
                'O' if self.in_escape && self.escape_type == EscapeType::None => {
                    // SS3 (Single Shift Three)
                    self.escape_type = EscapeType::SS3;
                    self.buffer.push(ch);
                }
                '\x07' if self.in_escape && matches!(self.escape_type, EscapeType::OSC | EscapeType::DCS | EscapeType::PM | EscapeType::APC) => {
                    // End of OSC/DCS/PM/APC sequence with BEL
                    if let Some(command) = self.parse_escape_sequence(&self.buffer) {
                        commands.push(command);
                    }
                    self.reset_escape_state();
                }
                '\x1b' if self.in_escape && chars.peek() == Some(&'\\') => {
                    // End of OSC/DCS/PM/APC sequence with ESC \
                    chars.next(); // consume the \\
                    if let Some(command) = self.parse_escape_sequence(&self.buffer) {
                        commands.push(command);
                    }
                    self.reset_escape_state();
                }
                'A'..='Z' | 'a'..='z' if self.in_escape && self.escape_type == EscapeType::CSI => {
                    // End of CSI sequence
                    self.buffer.push(ch);
                    if let Some(command) = self.parse_escape_sequence(&self.buffer) {
                        commands.push(command);
                    }
                    self.reset_escape_state();
                }
                _ if self.in_escape => {
                    self.buffer.push(ch);
                }
                '\r' => {
                    // Carriage return - move cursor to beginning of line
                    self.flush_buffer(&mut commands);
                    commands.push(AnsiCommand::CursorColumn(1));
                }
                '\n' => {
                    // Line feed - move cursor down one line
                    self.flush_buffer(&mut commands);
                    commands.push(AnsiCommand::CursorDown(1));
                }
                '\x07' => {
                    // Bell character
                    self.flush_buffer(&mut commands);
                    commands.push(AnsiCommand::Bell);
                }
                '\t' => {
                    // Tab character
                    self.flush_buffer(&mut commands);
                    commands.push(AnsiCommand::CursorRight(8)); // Simple tab implementation
                }
                '\x08' => {
                    // Backspace
                    self.flush_buffer(&mut commands);
                    commands.push(AnsiCommand::CursorLeft(1));
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

    fn flush_buffer(&mut self, commands: &mut Vec<AnsiCommand>) {
        if !self.buffer.is_empty() && !self.in_escape {
            commands.push(AnsiCommand::PrintText(self.buffer.clone()));
            self.buffer.clear();
        }
    }

    fn reset_escape_state(&mut self) {
        self.buffer.clear();
        self.in_escape = false;
        self.escape_type = EscapeType::None;
    }

    fn parse_escape_sequence(&self, seq: &str) -> Option<AnsiCommand> {
        if seq.len() < 2 {
            return Some(AnsiCommand::Unknown(seq.to_string()));
        }

        match &self.escape_type {
            EscapeType::CSI => self.parse_csi_sequence(seq),
            EscapeType::OSC => self.parse_osc_sequence(seq),
            EscapeType::DCS => self.parse_dcs_sequence(seq),
            _ => Some(AnsiCommand::Unknown(seq.to_string())),
        }
    }

    fn parse_csi_sequence(&self, seq: &str) -> Option<AnsiCommand> {
        if !seq.starts_with("\x1b[") {
            return Some(AnsiCommand::Unknown(seq.to_string()));
        }

        let command_char = seq.chars().last()?;
        let params_str = &seq[2..seq.len()-1];
        let params: Vec<u16> = params_str
            .split(';')
            .filter_map(|s| s.parse().ok())
            .collect();

        match command_char {
            // Cursor movement
            'A' => Some(AnsiCommand::CursorUp(params.get(0).copied().unwrap_or(1))),
            'B' => Some(AnsiCommand::CursorDown(params.get(0).copied().unwrap_or(1))),
            'C' => Some(AnsiCommand::CursorRight(params.get(0).copied().unwrap_or(1))),
            'D' => Some(AnsiCommand::CursorLeft(params.get(0).copied().unwrap_or(1))),
            'E' => Some(AnsiCommand::CursorNextLine(params.get(0).copied().unwrap_or(1))),
            'F' => Some(AnsiCommand::CursorPrevLine(params.get(0).copied().unwrap_or(1))),
            'G' => Some(AnsiCommand::CursorColumn(params.get(0).copied().unwrap_or(1))),
            'H' | 'f' => {
                let row = params.get(0).copied().unwrap_or(1);
                let col = params.get(1).copied().unwrap_or(1);
                Some(AnsiCommand::CursorPosition(row, col))
            }
            's' => Some(AnsiCommand::CursorSave),
            'u' => Some(AnsiCommand::CursorRestore),
            
            // Screen manipulation
            'J' => {
                match params.get(0).copied().unwrap_or(0) {
                    0 => Some(AnsiCommand::ClearFromCursor),
                    1 => Some(AnsiCommand::ClearToCursor),
                    2 => Some(AnsiCommand::ClearScreen),
                    3 => Some(AnsiCommand::ClearScreen), // Clear entire screen + scrollback
                    _ => Some(AnsiCommand::Unknown(seq.to_string())),
                }
            }
            'K' => {
                match params.get(0).copied().unwrap_or(0) {
                    0 => Some(AnsiCommand::ClearToEndOfLine),
                    1 => Some(AnsiCommand::ClearToBeginningOfLine),
                    2 => Some(AnsiCommand::ClearLine),
                    _ => Some(AnsiCommand::Unknown(seq.to_string())),
                }
            }
            
            // Text modification
            'L' => Some(AnsiCommand::InsertLines(params.get(0).copied().unwrap_or(1))),
            'M' => Some(AnsiCommand::DeleteLines(params.get(0).copied().unwrap_or(1))),
            'P' => Some(AnsiCommand::DeleteCharacters(params.get(0).copied().unwrap_or(1))),
            '@' => Some(AnsiCommand::InsertCharacters(params.get(0).copied().unwrap_or(1))),
            
            // Scrolling
            'S' => Some(AnsiCommand::ScrollUp(params.get(0).copied().unwrap_or(1))),
            'T' => Some(AnsiCommand::ScrollDown(params.get(0).copied().unwrap_or(1))),
            'r' => {
                let top = params.get(0).copied().unwrap_or(1);
                let bottom = params.get(1).copied().unwrap_or(24);
                Some(AnsiCommand::SetScrollRegion(top, bottom))
            }
            
            // Graphics mode
            'm' => {
                let params: Vec<u8> = params_str
                    .split(';')
                    .filter_map(|s| s.parse().ok())
                    .collect();
                Some(AnsiCommand::SetGraphicsMode(params))
            }
            
            // Cursor visibility and style
            'q' if params_str.starts_with(" ") => {
                // DECSCUSR - Set cursor style
                match params.get(0).copied().unwrap_or(1) {
                    0 | 1 => Some(AnsiCommand::SetCursorStyle(CursorStyle::BlinkingBlock)),
                    2 => Some(AnsiCommand::SetCursorStyle(CursorStyle::Block)),
                    3 => Some(AnsiCommand::SetCursorStyle(CursorStyle::BlinkingUnderline)),
                    4 => Some(AnsiCommand::SetCursorStyle(CursorStyle::Underline)),
                    5 => Some(AnsiCommand::SetCursorStyle(CursorStyle::BlinkingBar)),
                    6 => Some(AnsiCommand::SetCursorStyle(CursorStyle::Bar)),
                    _ => Some(AnsiCommand::Unknown(seq.to_string())),
                }
            }
            
            // Mouse reporting
            'h' if params_str == "?1000" => Some(AnsiCommand::EnableMouseReporting(MouseReportMode::Normal)),
            'l' if params_str == "?1000" => Some(AnsiCommand::DisableMouseReporting(MouseReportMode::Normal)),
            'h' if params_str == "?1002" => Some(AnsiCommand::EnableMouseReporting(MouseReportMode::Button)),
            'l' if params_str == "?1002" => Some(AnsiCommand::DisableMouseReporting(MouseReportMode::Button)),
            'h' if params_str == "?1003" => Some(AnsiCommand::EnableMouseReporting(MouseReportMode::Any)),
            'l' if params_str == "?1003" => Some(AnsiCommand::DisableMouseReporting(MouseReportMode::Any)),
            'h' if params_str == "?1006" => Some(AnsiCommand::EnableMouseReporting(MouseReportMode::SGR)),
            'l' if params_str == "?1006" => Some(AnsiCommand::DisableMouseReporting(MouseReportMode::SGR)),
            
            // Alternate screen
            'h' if params_str == "?1049" || params_str == "?47" => Some(AnsiCommand::EnterAlternateScreen),
            'l' if params_str == "?1049" || params_str == "?47" => Some(AnsiCommand::ExitAlternateScreen),
            
            // Cursor visibility
            'h' if params_str == "?25" => Some(AnsiCommand::ShowCursor),
            'l' if params_str == "?25" => Some(AnsiCommand::HideCursor),
            
            // Bracketed paste
            'h' if params_str == "?2004" => Some(AnsiCommand::EnableBracketedPaste),
            'l' if params_str == "?2004" => Some(AnsiCommand::DisableBracketedPaste),
            
            // Focus events
            'h' if params_str == "?1004" => Some(AnsiCommand::EnableFocusEvents),
            'l' if params_str == "?1004" => Some(AnsiCommand::DisableFocusEvents),
            
            // Synchronized updates
            'h' if params_str == "?2026" => Some(AnsiCommand::BeginSynchronizedUpdate),
            'l' if params_str == "?2026" => Some(AnsiCommand::EndSynchronizedUpdate),
            
            _ => Some(AnsiCommand::Unknown(seq.to_string())),
        }
    }
    
    fn parse_osc_sequence(&self, seq: &str) -> Option<AnsiCommand> {
        if !seq.starts_with("\x1b]") {
            return Some(AnsiCommand::Unknown(seq.to_string()));
        }
        
        let content = &seq[2..];
        let parts: Vec<&str> = content.splitn(2, ';').collect();
        
        if let Some(command_num) = parts[0].parse::<u16>().ok() {
            match command_num {
                0 | 2 => {
                    // Set window title
                    let title = parts.get(1).unwrap_or(&"").to_string();
                    Some(AnsiCommand::SetWindowTitle(title))
                }
                1 => {
                    // Set icon title
                    let title = parts.get(1).unwrap_or(&"").to_string();
                    Some(AnsiCommand::SetIconTitle(title))
                }
                8 => {
                    // Hyperlink
                    let hyperlink_parts: Vec<&str> = parts.get(1).unwrap_or(&"").splitn(2, ';').collect();
                    let url = hyperlink_parts.get(1).unwrap_or(&"").to_string();
                    let text = "".to_string(); // Text will be in subsequent print commands
                    Some(AnsiCommand::SetHyperlink(url, text))
                }
                1337 => {
                    // iTerm2 proprietary sequences
                    if let Some(data) = parts.get(1) {
                        if data.starts_with("File=") {
                            // Image display
                            if let Ok(decoded) = general_purpose::STANDARD.decode(data[5..].as_bytes()) {
                                Some(AnsiCommand::DisplayImage(ImageData {
                                    format: "png".to_string(),
                                    width: None,
                                    height: None,
                                    data: decoded,
                                }))
                            } else {
                                Some(AnsiCommand::Unknown(seq.to_string()))
                            }
                        } else {
                            Some(AnsiCommand::Unknown(seq.to_string()))
                        }
                    } else {
                        Some(AnsiCommand::Unknown(seq.to_string()))
                    }
                }
                _ => Some(AnsiCommand::Unknown(seq.to_string())),
            }
        } else {
            Some(AnsiCommand::Unknown(seq.to_string()))
        }
    }
    
    fn parse_dcs_sequence(&self, seq: &str) -> Option<AnsiCommand> {
        if !seq.starts_with("\x1bP") {
            return Some(AnsiCommand::Unknown(seq.to_string()));
        }
        
        let content = &seq[2..];
        
        // Check for Sixel graphics
        if content.starts_with("q") || content.contains("#") {
            Some(AnsiCommand::DisplaySixel(content.as_bytes().to_vec()))
        } else {
            Some(AnsiCommand::DeviceControlString(content.to_string()))
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
