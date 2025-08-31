use std::collections::HashMap;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TerminalType {
    XTerm,
    XTerm256Color,
    XTermTrueColor,
    Screen,
    Screen256Color,
    Tmux,
    Tmux256Color,
    VT100,
    VT220,
    VT320,
    VT420,
    VT520,
    LinuxConsole,
    WindowsConsole,
    ANSI,
    Dumb,
    Unknown(String),
}

impl TerminalType {
    pub fn from_env() -> Self {
        let term = std::env::var("TERM").unwrap_or_else(|_| "xterm".to_string());
        Self::from_string(&term)
    }

    pub fn from_string(term: &str) -> Self {
        match term.to_lowercase().as_str() {
            "xterm" => Self::XTerm,
            "xterm-256color" => Self::XTerm256Color,
            "xterm-truecolor" | "xterm-24bit" => Self::XTermTrueColor,
            "screen" => Self::Screen,
            "screen-256color" => Self::Screen256Color,
            "tmux" => Self::Tmux,
            "tmux-256color" => Self::Tmux256Color,
            "vt100" => Self::VT100,
            "vt220" => Self::VT220,
            "vt320" => Self::VT320,
            "vt420" => Self::VT420,
            "vt520" => Self::VT520,
            "linux" | "console" => Self::LinuxConsole,
            "cygwin" | "msys" => Self::WindowsConsole,
            "ansi" => Self::ANSI,
            "dumb" => Self::Dumb,
            _ => Self::Unknown(term.to_string()),
        }
    }

    pub fn to_string(&self) -> String {
        match self {
            Self::XTerm => "xterm".to_string(),
            Self::XTerm256Color => "xterm-256color".to_string(),
            Self::XTermTrueColor => "xterm-truecolor".to_string(),
            Self::Screen => "screen".to_string(),
            Self::Screen256Color => "screen-256color".to_string(),
            Self::Tmux => "tmux".to_string(),
            Self::Tmux256Color => "tmux-256color".to_string(),
            Self::VT100 => "vt100".to_string(),
            Self::VT220 => "vt220".to_string(),
            Self::VT320 => "vt320".to_string(),
            Self::VT420 => "vt420".to_string(),
            Self::VT520 => "vt520".to_string(),
            Self::LinuxConsole => "linux".to_string(),
            Self::WindowsConsole => "cygwin".to_string(),
            Self::ANSI => "ansi".to_string(),
            Self::Dumb => "dumb".to_string(),
            Self::Unknown(s) => s.clone(),
        }
    }

    pub fn capabilities(&self) -> TerminalCapabilities {
        match self {
            Self::XTermTrueColor => TerminalCapabilities {
                colors: ColorSupport::TrueColor,
                cursor_styles: true,
                mouse_support: true,
                bracketed_paste: true,
                alternate_screen: true,
                title_setting: true,
                focus_events: true,
                unicode_support: true,
                sixel_graphics: false,
                iterm2_images: true,
                hyperlinks: true,
                synchronized_updates: true,
            },
            Self::XTerm256Color | Self::Screen256Color | Self::Tmux256Color => TerminalCapabilities {
                colors: ColorSupport::Color256,
                cursor_styles: true,
                mouse_support: true,
                bracketed_paste: true,
                alternate_screen: true,
                title_setting: true,
                focus_events: true,
                unicode_support: true,
                sixel_graphics: false,
                iterm2_images: false,
                hyperlinks: false,
                synchronized_updates: false,
            },
            Self::XTerm | Self::Screen | Self::Tmux => TerminalCapabilities {
                colors: ColorSupport::Color16,
                cursor_styles: true,
                mouse_support: true,
                bracketed_paste: true,
                alternate_screen: true,
                title_setting: true,
                focus_events: false,
                unicode_support: true,
                sixel_graphics: false,
                iterm2_images: false,
                hyperlinks: false,
                synchronized_updates: false,
            },
            Self::VT520 => TerminalCapabilities {
                colors: ColorSupport::Color16,
                cursor_styles: true,
                mouse_support: false,
                bracketed_paste: false,
                alternate_screen: true,
                title_setting: false,
                focus_events: false,
                unicode_support: false,
                sixel_graphics: true,
                iterm2_images: false,
                hyperlinks: false,
                synchronized_updates: false,
            },
            Self::VT220 | Self::VT320 | Self::VT420 => TerminalCapabilities {
                colors: ColorSupport::Monochrome,
                cursor_styles: true,
                mouse_support: false,
                bracketed_paste: false,
                alternate_screen: true,
                title_setting: false,
                focus_events: false,
                unicode_support: false,
                sixel_graphics: false,
                iterm2_images: false,
                hyperlinks: false,
                synchronized_updates: false,
            },
            Self::VT100 => TerminalCapabilities {
                colors: ColorSupport::Monochrome,
                cursor_styles: false,
                mouse_support: false,
                bracketed_paste: false,
                alternate_screen: false,
                title_setting: false,
                focus_events: false,
                unicode_support: false,
                sixel_graphics: false,
                iterm2_images: false,
                hyperlinks: false,
                synchronized_updates: false,
            },
            Self::LinuxConsole => TerminalCapabilities {
                colors: ColorSupport::Color16,
                cursor_styles: false,
                mouse_support: false,
                bracketed_paste: false,
                alternate_screen: false,
                title_setting: false,
                focus_events: false,
                unicode_support: true,
                sixel_graphics: false,
                iterm2_images: false,
                hyperlinks: false,
                synchronized_updates: false,
            },
            Self::WindowsConsole => TerminalCapabilities {
                colors: ColorSupport::Color16,
                cursor_styles: true,
                mouse_support: true,
                bracketed_paste: true,
                alternate_screen: true,
                title_setting: true,
                focus_events: false,
                unicode_support: true,
                sixel_graphics: false,
                iterm2_images: false,
                hyperlinks: false,
                synchronized_updates: false,
            },
            Self::ANSI => TerminalCapabilities {
                colors: ColorSupport::Color16,
                cursor_styles: false,
                mouse_support: false,
                bracketed_paste: false,
                alternate_screen: false,
                title_setting: false,
                focus_events: false,
                unicode_support: false,
                sixel_graphics: false,
                iterm2_images: false,
                hyperlinks: false,
                synchronized_updates: false,
            },
            Self::Dumb => TerminalCapabilities::minimal(),
            Self::Unknown(_) => TerminalCapabilities::default(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ColorSupport {
    Monochrome,
    Color16,
    Color256,
    TrueColor,
}

#[derive(Debug, Clone)]
pub struct TerminalCapabilities {
    pub colors: ColorSupport,
    pub cursor_styles: bool,
    pub mouse_support: bool,
    pub bracketed_paste: bool,
    pub alternate_screen: bool,
    pub title_setting: bool,
    pub focus_events: bool,
    pub unicode_support: bool,
    pub sixel_graphics: bool,
    pub iterm2_images: bool,
    pub hyperlinks: bool,
    pub synchronized_updates: bool,
}

impl TerminalCapabilities {
    pub fn minimal() -> Self {
        Self {
            colors: ColorSupport::Monochrome,
            cursor_styles: false,
            mouse_support: false,
            bracketed_paste: false,
            alternate_screen: false,
            title_setting: false,
            focus_events: false,
            unicode_support: false,
            sixel_graphics: false,
            iterm2_images: false,
            hyperlinks: false,
            synchronized_updates: false,
        }
    }
}

impl Default for TerminalCapabilities {
    fn default() -> Self {
        Self {
            colors: ColorSupport::Color16,
            cursor_styles: true,
            mouse_support: true,
            bracketed_paste: true,
            alternate_screen: true,
            title_setting: true,
            focus_events: false,
            unicode_support: true,
            sixel_graphics: false,
            iterm2_images: false,
            hyperlinks: false,
            synchronized_updates: false,
        }
    }
}

/// Terminal information database (terminfo-like)
pub struct TerminalDatabase {
    capabilities: HashMap<String, TerminalCapabilities>,
}

impl TerminalDatabase {
    pub fn new() -> Self {
        let mut db = Self {
            capabilities: HashMap::new(),
        };
        db.populate_defaults();
        db
    }

    fn populate_defaults(&mut self) {
        // Add common terminal definitions
        for term_type in [
            TerminalType::XTerm,
            TerminalType::XTerm256Color,
            TerminalType::XTermTrueColor,
            TerminalType::Screen,
            TerminalType::Screen256Color,
            TerminalType::Tmux,
            TerminalType::Tmux256Color,
            TerminalType::VT100,
            TerminalType::VT220,
            TerminalType::VT320,
            TerminalType::VT420,
            TerminalType::VT520,
            TerminalType::LinuxConsole,
            TerminalType::WindowsConsole,
            TerminalType::ANSI,
            TerminalType::Dumb,
        ] {
            self.capabilities.insert(term_type.to_string(), term_type.capabilities());
        }
    }

    pub fn get_capabilities(&self, term_name: &str) -> TerminalCapabilities {
        self.capabilities
            .get(term_name)
            .cloned()
            .unwrap_or_else(TerminalCapabilities::default)
    }

    pub fn supports_feature(&self, term_name: &str, feature: &str) -> bool {
        let caps = self.get_capabilities(term_name);
        match feature {
            "colors" => !matches!(caps.colors, ColorSupport::Monochrome),
            "256colors" => matches!(caps.colors, ColorSupport::Color256 | ColorSupport::TrueColor),
            "truecolor" => matches!(caps.colors, ColorSupport::TrueColor),
            "cursor_styles" => caps.cursor_styles,
            "mouse" => caps.mouse_support,
            "bracketed_paste" => caps.bracketed_paste,
            "alternate_screen" => caps.alternate_screen,
            "title" => caps.title_setting,
            "focus_events" => caps.focus_events,
            "unicode" => caps.unicode_support,
            "sixel" => caps.sixel_graphics,
            "images" => caps.iterm2_images,
            "hyperlinks" => caps.hyperlinks,
            "synchronized_updates" => caps.synchronized_updates,
            _ => false,
        }
    }
}
