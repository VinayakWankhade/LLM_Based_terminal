use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessibilityConfig {
    pub screen_reader_support: bool,
    pub high_contrast_mode: bool,
    pub reduced_motion: bool,
    pub focus_indicators: bool,
    pub keyboard_navigation: bool,
    pub voice_announcements: bool,
    pub magnification_enabled: bool,
    pub magnification_level: f32,
    pub color_blind_support: ColorBlindSupport,
    pub font_settings: AccessibilityFontSettings,
    pub audio_cues: AudioCueSettings,
    pub alternative_text: bool,
    pub skip_links: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColorBlindSupport {
    pub enabled: bool,
    pub color_blind_type: ColorBlindType,
    pub color_adjustments: HashMap<String, String>, // color mappings
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ColorBlindType {
    None,
    Protanopia,    // Red-blind
    Deuteranopia,  // Green-blind
    Tritanopia,    // Blue-blind
    Protanomaly,   // Red-weak
    Deuteranomaly, // Green-weak
    Tritanomaly,   // Blue-weak
    Monochromacy,  // Complete color blindness
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessibilityFontSettings {
    pub dyslexia_friendly_font: bool,
    pub minimum_font_size: u16,
    pub line_height_multiplier: f32,
    pub letter_spacing: f32,
    pub word_spacing: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioCueSettings {
    pub enabled: bool,
    pub volume: f32,
    pub error_sounds: bool,
    pub success_sounds: bool,
    pub notification_sounds: bool,
    pub typing_sounds: bool,
    pub navigation_sounds: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyboardShortcut {
    pub id: String,
    pub name: String,
    pub description: String,
    pub keys: Vec<String>,
    pub context: ShortcutContext,
    pub action: String,
    pub enabled: bool,
    pub customizable: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ShortcutContext {
    Global,
    Terminal,
    Editor,
    FileExplorer,
    Settings,
    Debug,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FocusManager {
    pub current_focus: Option<String>,
    pub focus_history: Vec<String>,
    pub focus_trap_stack: Vec<String>,
    pub skip_links: Vec<SkipLink>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkipLink {
    pub id: String,
    pub label: String,
    pub target: String,
    pub visible: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScreenReaderAnnouncement {
    pub message: String,
    pub priority: AnnouncementPriority,
    pub interrupt: bool,
    pub timestamp: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AnnouncementPriority {
    Low,
    Medium,
    High,
    Emergency,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessibilityRule {
    pub rule_id: String,
    pub name: String,
    pub description: String,
    pub severity: RuleSeverity,
    pub wcag_level: WcagLevel,
    pub check_function: String, // JavaScript function name
    pub auto_fix: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RuleSeverity {
    Error,
    Warning,
    Info,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum WcagLevel {
    A,
    AA,
    AAA,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessibilityAuditResult {
    pub rule_id: String,
    pub element_id: Option<String>,
    pub severity: RuleSeverity,
    pub message: String,
    pub suggestion: Option<String>,
    pub auto_fixable: bool,
}

// Internationalization Types

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct I18nConfig {
    pub current_locale: String,
    pub fallback_locale: String,
    pub available_locales: Vec<LocaleInfo>,
    pub rtl_support: bool,
    pub date_format: String,
    pub time_format: String,
    pub number_format: NumberFormatSettings,
    pub currency_settings: CurrencySettings,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocaleInfo {
    pub code: String,     // e.g., "en-US", "fr-FR"
    pub name: String,     // e.g., "English (United States)"
    pub native_name: String, // e.g., "English", "Français"
    pub rtl: bool,
    pub region: String,
    pub language: String,
    pub completion: f32,  // Translation completion percentage
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NumberFormatSettings {
    pub decimal_separator: String,
    pub thousands_separator: String,
    pub grouping: Vec<u8>, // e.g., [3, 3] for 1,000,000
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurrencySettings {
    pub code: String,     // e.g., "USD", "EUR"
    pub symbol: String,   // e.g., "$", "€"
    pub position: CurrencyPosition,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum CurrencyPosition {
    Before, // $100
    After,  // 100$
    BeforeWithSpace, // $ 100
    AfterWithSpace,  // 100 $
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranslationKey {
    pub key: String,
    pub default_value: String,
    pub context: Option<String>,
    pub plural_forms: Option<Vec<String>>,
    pub interpolations: Vec<String>, // Variables like {name}, {count}
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Translation {
    pub key: String,
    pub locale: String,
    pub value: String,
    pub plural_forms: Option<HashMap<String, String>>, // one, few, many, other
    pub context: Option<String>,
    pub last_updated: u64,
    pub status: TranslationStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TranslationStatus {
    Missing,
    Incomplete,
    Complete,
    NeedsReview,
    Approved,
}

pub struct AccessibilityManager {
    config: Arc<Mutex<AccessibilityConfig>>,
    shortcuts: Arc<Mutex<HashMap<String, KeyboardShortcut>>>,
    focus_manager: Arc<Mutex<FocusManager>>,
    announcements: Arc<Mutex<Vec<ScreenReaderAnnouncement>>>,
    accessibility_rules: Arc<Mutex<Vec<AccessibilityRule>>>,
}

impl AccessibilityManager {
    pub fn new() -> Self {
        let default_config = AccessibilityConfig {
            screen_reader_support: false,
            high_contrast_mode: false,
            reduced_motion: false,
            focus_indicators: true,
            keyboard_navigation: true,
            voice_announcements: false,
            magnification_enabled: false,
            magnification_level: 1.0,
            color_blind_support: ColorBlindSupport {
                enabled: false,
                color_blind_type: ColorBlindType::None,
                color_adjustments: HashMap::new(),
            },
            font_settings: AccessibilityFontSettings {
                dyslexia_friendly_font: false,
                minimum_font_size: 12,
                line_height_multiplier: 1.2,
                letter_spacing: 0.0,
                word_spacing: 0.0,
            },
            audio_cues: AudioCueSettings {
                enabled: false,
                volume: 0.5,
                error_sounds: true,
                success_sounds: true,
                notification_sounds: true,
                typing_sounds: false,
                navigation_sounds: true,
            },
            alternative_text: true,
            skip_links: true,
        };

        let default_focus_manager = FocusManager {
            current_focus: None,
            focus_history: Vec::new(),
            focus_trap_stack: Vec::new(),
            skip_links: vec![
                SkipLink {
                    id: "skip-to-main".to_string(),
                    label: "Skip to main content".to_string(),
                    target: "main".to_string(),
                    visible: false,
                },
                SkipLink {
                    id: "skip-to-nav".to_string(),
                    label: "Skip to navigation".to_string(),
                    target: "navigation".to_string(),
                    visible: false,
                },
            ],
        };

        Self {
            config: Arc::new(Mutex::new(default_config)),
            shortcuts: Arc::new(Mutex::new(Self::create_default_shortcuts())),
            focus_manager: Arc::new(Mutex::new(default_focus_manager)),
            announcements: Arc::new(Mutex::new(Vec::new())),
            accessibility_rules: Arc::new(Mutex::new(Self::create_accessibility_rules())),
        }
    }

    fn create_default_shortcuts() -> HashMap<String, KeyboardShortcut> {
        let mut shortcuts = HashMap::new();

        shortcuts.insert("terminal.new_tab".to_string(), KeyboardShortcut {
            id: "terminal.new_tab".to_string(),
            name: "New Tab".to_string(),
            description: "Create a new terminal tab".to_string(),
            keys: vec!["Ctrl".to_string(), "T".to_string()],
            context: ShortcutContext::Terminal,
            action: "new_tab".to_string(),
            enabled: true,
            customizable: true,
        });

        shortcuts.insert("terminal.close_tab".to_string(), KeyboardShortcut {
            id: "terminal.close_tab".to_string(),
            name: "Close Tab".to_string(),
            description: "Close the current terminal tab".to_string(),
            keys: vec!["Ctrl".to_string(), "W".to_string()],
            context: ShortcutContext::Terminal,
            action: "close_tab".to_string(),
            enabled: true,
            customizable: true,
        });

        shortcuts.insert("accessibility.toggle_screen_reader".to_string(), KeyboardShortcut {
            id: "accessibility.toggle_screen_reader".to_string(),
            name: "Toggle Screen Reader".to_string(),
            description: "Enable or disable screen reader support".to_string(),
            keys: vec!["Ctrl".to_string(), "Alt".to_string(), "S".to_string()],
            context: ShortcutContext::Global,
            action: "toggle_screen_reader".to_string(),
            enabled: true,
            customizable: true,
        });

        shortcuts.insert("accessibility.increase_font_size".to_string(), KeyboardShortcut {
            id: "accessibility.increase_font_size".to_string(),
            name: "Increase Font Size".to_string(),
            description: "Make text larger".to_string(),
            keys: vec!["Ctrl".to_string(), "Plus".to_string()],
            context: ShortcutContext::Global,
            action: "increase_font_size".to_string(),
            enabled: true,
            customizable: true,
        });

        shortcuts.insert("accessibility.decrease_font_size".to_string(), KeyboardShortcut {
            id: "accessibility.decrease_font_size".to_string(),
            name: "Decrease Font Size".to_string(),
            description: "Make text smaller".to_string(),
            keys: vec!["Ctrl".to_string(), "Minus".to_string()],
            context: ShortcutContext::Global,
            action: "decrease_font_size".to_string(),
            enabled: true,
            customizable: true,
        });

        shortcuts.insert("accessibility.toggle_high_contrast".to_string(), KeyboardShortcut {
            id: "accessibility.toggle_high_contrast".to_string(),
            name: "Toggle High Contrast".to_string(),
            description: "Toggle high contrast mode".to_string(),
            keys: vec!["Ctrl".to_string(), "Alt".to_string(), "H".to_string()],
            context: ShortcutContext::Global,
            action: "toggle_high_contrast".to_string(),
            enabled: true,
            customizable: true,
        });

        shortcuts
    }

    fn create_accessibility_rules() -> Vec<AccessibilityRule> {
        vec![
            AccessibilityRule {
                rule_id: "missing_alt_text".to_string(),
                name: "Missing Alt Text".to_string(),
                description: "Images must have alternative text".to_string(),
                severity: RuleSeverity::Error,
                wcag_level: WcagLevel::A,
                check_function: "checkAltText".to_string(),
                auto_fix: Some("addAltText".to_string()),
            },
            AccessibilityRule {
                rule_id: "insufficient_color_contrast".to_string(),
                name: "Insufficient Color Contrast".to_string(),
                description: "Text must have sufficient color contrast".to_string(),
                severity: RuleSeverity::Error,
                wcag_level: WcagLevel::AA,
                check_function: "checkColorContrast".to_string(),
                auto_fix: None,
            },
            AccessibilityRule {
                rule_id: "missing_focus_indicator".to_string(),
                name: "Missing Focus Indicator".to_string(),
                description: "Interactive elements must have visible focus indicators".to_string(),
                severity: RuleSeverity::Warning,
                wcag_level: WcagLevel::AA,
                check_function: "checkFocusIndicator".to_string(),
                auto_fix: Some("addFocusIndicator".to_string()),
            },
            AccessibilityRule {
                rule_id: "missing_heading_structure".to_string(),
                name: "Missing Heading Structure".to_string(),
                description: "Page must have proper heading hierarchy".to_string(),
                severity: RuleSeverity::Warning,
                wcag_level: WcagLevel::AA,
                check_function: "checkHeadingStructure".to_string(),
                auto_fix: None,
            },
            AccessibilityRule {
                rule_id: "missing_skip_links".to_string(),
                name: "Missing Skip Links".to_string(),
                description: "Page should have skip navigation links".to_string(),
                severity: RuleSeverity::Info,
                wcag_level: WcagLevel::A,
                check_function: "checkSkipLinks".to_string(),
                auto_fix: Some("addSkipLinks".to_string()),
            },
        ]
    }

    // Configuration Management
    pub fn get_config(&self) -> AccessibilityConfig {
        let config = self.config.lock().unwrap();
        config.clone()
    }

    pub fn update_config(&self, new_config: AccessibilityConfig) {
        let mut config = self.config.lock().unwrap();
        *config = new_config;
    }

    pub fn enable_screen_reader_support(&self) {
        let mut config = self.config.lock().unwrap();
        config.screen_reader_support = true;
        config.voice_announcements = true;
        config.focus_indicators = true;
        config.keyboard_navigation = true;
    }

    pub fn disable_screen_reader_support(&self) {
        let mut config = self.config.lock().unwrap();
        config.screen_reader_support = false;
        config.voice_announcements = false;
    }

    pub fn toggle_high_contrast(&self) -> bool {
        let mut config = self.config.lock().unwrap();
        config.high_contrast_mode = !config.high_contrast_mode;
        config.high_contrast_mode
    }

    pub fn set_magnification(&self, level: f32) {
        let mut config = self.config.lock().unwrap();
        config.magnification_enabled = level > 1.0;
        config.magnification_level = level.max(0.5).min(5.0);
    }

    // Keyboard Shortcuts
    pub fn get_shortcuts(&self, context: Option<ShortcutContext>) -> Vec<KeyboardShortcut> {
        let shortcuts = self.shortcuts.lock().unwrap();
        
        if let Some(ctx) = context {
            shortcuts.values()
                .filter(|s| s.context == ctx || s.context == ShortcutContext::Global)
                .cloned()
                .collect()
        } else {
            shortcuts.values().cloned().collect()
        }
    }

    pub fn update_shortcut(&self, shortcut_id: &str, new_keys: Vec<String>) -> Result<(), String> {
        let mut shortcuts = self.shortcuts.lock().unwrap();
        
        if let Some(shortcut) = shortcuts.get_mut(shortcut_id) {
            if shortcut.customizable {
                shortcut.keys = new_keys;
                Ok(())
            } else {
                Err("This shortcut cannot be customized".to_string())
            }
        } else {
            Err(format!("Shortcut {} not found", shortcut_id))
        }
    }

    pub fn add_custom_shortcut(&self, shortcut: KeyboardShortcut) -> Result<(), String> {
        let mut shortcuts = self.shortcuts.lock().unwrap();
        
        // Check for conflicts
        for existing in shortcuts.values() {
            if existing.keys == shortcut.keys && 
               existing.context == shortcut.context &&
               existing.enabled {
                return Err(format!("Shortcut conflict with: {}", existing.name));
            }
        }

        shortcuts.insert(shortcut.id.clone(), shortcut);
        Ok(())
    }

    // Focus Management
    pub fn set_focus(&self, element_id: &str) {
        let mut focus_manager = self.focus_manager.lock().unwrap();
        
        let current_focus = focus_manager.current_focus.clone();
        if let Some(current) = current_focus {
            focus_manager.focus_history.push(current);
            
            // Limit history size
            if focus_manager.focus_history.len() > 50 {
                focus_manager.focus_history.remove(0);
            }
        }
        
        focus_manager.current_focus = Some(element_id.to_string());
    }

    pub fn get_current_focus(&self) -> Option<String> {
        let focus_manager = self.focus_manager.lock().unwrap();
        focus_manager.current_focus.clone()
    }

    pub fn focus_previous(&self) -> Option<String> {
        let mut focus_manager = self.focus_manager.lock().unwrap();
        
        if let Some(previous) = focus_manager.focus_history.pop() {
            focus_manager.current_focus = Some(previous.clone());
            Some(previous)
        } else {
            None
        }
    }

    pub fn create_focus_trap(&self, trap_id: &str) {
        let mut focus_manager = self.focus_manager.lock().unwrap();
        focus_manager.focus_trap_stack.push(trap_id.to_string());
    }

    pub fn release_focus_trap(&self) -> Option<String> {
        let mut focus_manager = self.focus_manager.lock().unwrap();
        focus_manager.focus_trap_stack.pop()
    }

    pub fn get_skip_links(&self) -> Vec<SkipLink> {
        let focus_manager = self.focus_manager.lock().unwrap();
        focus_manager.skip_links.clone()
    }

    // Screen Reader Announcements
    pub fn announce(&self, message: &str, priority: AnnouncementPriority, interrupt: bool) {
        use std::time::{SystemTime, UNIX_EPOCH};
        
        let announcement = ScreenReaderAnnouncement {
            message: message.to_string(),
            priority,
            interrupt,
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        };

        let mut announcements = self.announcements.lock().unwrap();
        
        // Clear previous announcements if this is an interrupting announcement
        if interrupt {
            announcements.clear();
        }
        
        announcements.push(announcement);
        
        // Limit queue size
        while announcements.len() > 10 {
            announcements.remove(0);
        }
    }

    pub fn get_pending_announcements(&self) -> Vec<ScreenReaderAnnouncement> {
        let mut announcements = self.announcements.lock().unwrap();
        let pending = announcements.clone();
        announcements.clear();
        pending
    }

    // Color Blind Support
    pub fn set_color_blind_support(&self, color_blind_type: ColorBlindType) {
        let mut config = self.config.lock().unwrap();
        config.color_blind_support.enabled = color_blind_type != ColorBlindType::None;
        config.color_blind_support.color_blind_type = color_blind_type.clone();
        
        // Set up color adjustments based on type
        config.color_blind_support.color_adjustments = match color_blind_type {
            ColorBlindType::Protanopia => {
                // Red-blind: adjust red colors
                [
                    ("#ff0000".to_string(), "#0066cc".to_string()), // Red -> Blue
                    ("#ff6600".to_string(), "#0099cc".to_string()), // Orange -> Light Blue
                ].into_iter().collect()
            },
            ColorBlindType::Deuteranopia => {
                // Green-blind: adjust green colors
                [
                    ("#00ff00".to_string(), "#ffff00".to_string()), // Green -> Yellow
                    ("#009900".to_string(), "#cc6600".to_string()), // Dark Green -> Orange
                ].into_iter().collect()
            },
            ColorBlindType::Tritanopia => {
                // Blue-blind: adjust blue colors
                [
                    ("#0000ff".to_string(), "#ff00ff".to_string()), // Blue -> Magenta
                    ("#0066cc".to_string(), "#cc0066".to_string()), // Light Blue -> Pink
                ].into_iter().collect()
            },
            _ => HashMap::new(),
        };
    }

    pub fn get_adjusted_color(&self, color: &str) -> String {
        let config = self.config.lock().unwrap();
        
        if config.color_blind_support.enabled {
            if let Some(adjusted) = config.color_blind_support.color_adjustments.get(color) {
                return adjusted.clone();
            }
        }
        
        color.to_string()
    }

    // Accessibility Auditing
    pub fn run_accessibility_audit(&self, element_data: &str) -> Vec<AccessibilityAuditResult> {
        let rules = self.accessibility_rules.lock().unwrap();
        let mut results = Vec::new();

        // This would typically involve running JavaScript functions to check elements
        // For this example, we'll create mock results
        for rule in rules.iter() {
            // Mock audit logic - in reality this would parse the element_data
            // and run the appropriate checks
            match rule.rule_id.as_str() {
                "missing_alt_text" => {
                    if element_data.contains("<img") && !element_data.contains("alt=") {
                        results.push(AccessibilityAuditResult {
                            rule_id: rule.rule_id.clone(),
                            element_id: Some("image-1".to_string()),
                            severity: rule.severity.clone(),
                            message: "Image is missing alt attribute".to_string(),
                            suggestion: Some("Add descriptive alt text to the image".to_string()),
                            auto_fixable: rule.auto_fix.is_some(),
                        });
                    }
                },
                "insufficient_color_contrast" => {
                    // Mock color contrast check
                    results.push(AccessibilityAuditResult {
                        rule_id: rule.rule_id.clone(),
                        element_id: Some("text-1".to_string()),
                        severity: rule.severity.clone(),
                        message: "Text has insufficient color contrast ratio (2.1:1)".to_string(),
                        suggestion: Some("Increase contrast ratio to at least 4.5:1".to_string()),
                        auto_fixable: false,
                    });
                },
                _ => {}
            }
        }

        results
    }

    // Utility Functions
    pub fn generate_accessibility_report(&self) -> String {
        let config = self.config.lock().unwrap();
        let shortcuts = self.shortcuts.lock().unwrap();
        
        format!("Accessibility Report\n\
                 ====================\n\
                 Screen Reader Support: {}\n\
                 High Contrast Mode: {}\n\
                 Keyboard Navigation: {}\n\
                 Voice Announcements: {}\n\
                 Color Blind Support: {} ({})\n\
                 Custom Shortcuts: {}\n\
                 Minimum Font Size: {}px\n",
                config.screen_reader_support,
                config.high_contrast_mode,
                config.keyboard_navigation,
                config.voice_announcements,
                config.color_blind_support.enabled,
                format!("{:?}", config.color_blind_support.color_blind_type),
                shortcuts.len(),
                config.font_settings.minimum_font_size)
    }

    pub fn get_wcag_compliance_level(&self) -> WcagLevel {
        // Simple compliance check - in reality this would be more complex
        let config = self.config.lock().unwrap();
        
        if config.screen_reader_support && 
           config.keyboard_navigation && 
           config.focus_indicators && 
           config.alternative_text {
            WcagLevel::AA
        } else if config.keyboard_navigation && config.alternative_text {
            WcagLevel::A
        } else {
            WcagLevel::A // Default to A level
        }
    }
}

pub struct I18nManager {
    config: Arc<Mutex<I18nConfig>>,
    translations: Arc<Mutex<HashMap<String, HashMap<String, Translation>>>>, // locale -> key -> translation
    translation_keys: Arc<Mutex<HashMap<String, TranslationKey>>>,
    missing_translations: Arc<Mutex<Vec<String>>>,
}

impl I18nManager {
    pub fn new() -> Self {
        let default_config = I18nConfig {
            current_locale: "en-US".to_string(),
            fallback_locale: "en".to_string(),
            available_locales: vec![
                LocaleInfo {
                    code: "en-US".to_string(),
                    name: "English (United States)".to_string(),
                    native_name: "English".to_string(),
                    rtl: false,
                    region: "US".to_string(),
                    language: "en".to_string(),
                    completion: 1.0,
                },
                LocaleInfo {
                    code: "es-ES".to_string(),
                    name: "Spanish (Spain)".to_string(),
                    native_name: "Español".to_string(),
                    rtl: false,
                    region: "ES".to_string(),
                    language: "es".to_string(),
                    completion: 0.85,
                },
                LocaleInfo {
                    code: "fr-FR".to_string(),
                    name: "French (France)".to_string(),
                    native_name: "Français".to_string(),
                    rtl: false,
                    region: "FR".to_string(),
                    language: "fr".to_string(),
                    completion: 0.72,
                },
                LocaleInfo {
                    code: "ar-SA".to_string(),
                    name: "Arabic (Saudi Arabia)".to_string(),
                    native_name: "العربية".to_string(),
                    rtl: true,
                    region: "SA".to_string(),
                    language: "ar".to_string(),
                    completion: 0.45,
                },
            ],
            rtl_support: true,
            date_format: "MM/DD/YYYY".to_string(),
            time_format: "12".to_string(),
            number_format: NumberFormatSettings {
                decimal_separator: ".".to_string(),
                thousands_separator: ",".to_string(),
                grouping: vec![3],
            },
            currency_settings: CurrencySettings {
                code: "USD".to_string(),
                symbol: "$".to_string(),
                position: CurrencyPosition::Before,
            },
        };

        Self {
            config: Arc::new(Mutex::new(default_config)),
            translations: Arc::new(Mutex::new(HashMap::new())),
            translation_keys: Arc::new(Mutex::new(HashMap::new())),
            missing_translations: Arc::new(Mutex::new(Vec::new())),
        }
    }

    // Configuration
    pub fn get_config(&self) -> I18nConfig {
        let config = self.config.lock().unwrap();
        config.clone()
    }

    pub fn set_locale(&self, locale: &str) -> Result<(), String> {
        let mut config = self.config.lock().unwrap();
        
        // Validate locale exists
        if !config.available_locales.iter().any(|l| l.code == locale) {
            return Err(format!("Locale {} not available", locale));
        }
        
        config.current_locale = locale.to_string();
        Ok(())
    }

    pub fn get_current_locale(&self) -> String {
        let config = self.config.lock().unwrap();
        config.current_locale.clone()
    }

    pub fn is_rtl(&self) -> bool {
        let config = self.config.lock().unwrap();
        config.available_locales.iter()
            .find(|l| l.code == config.current_locale)
            .map(|l| l.rtl)
            .unwrap_or(false)
    }

    // Translation Management
    pub fn register_translation_key(&self, key: TranslationKey) {
        let mut keys = self.translation_keys.lock().unwrap();
        keys.insert(key.key.clone(), key);
    }

    pub fn add_translation(&self, translation: Translation) {
        let mut translations = self.translations.lock().unwrap();
        
        let locale_translations = translations
            .entry(translation.locale.clone())
            .or_insert_with(HashMap::new);
        
        locale_translations.insert(translation.key.clone(), translation);
    }

    pub fn translate(&self, key: &str, interpolations: Option<HashMap<String, String>>) -> String {
        let config = self.config.lock().unwrap();
        let translations = self.translations.lock().unwrap();
        let keys = self.translation_keys.lock().unwrap();
        
        // Try current locale first
        let current_locale = &config.current_locale;
        if let Some(locale_translations) = translations.get(current_locale) {
            if let Some(translation) = locale_translations.get(key) {
                return self.interpolate_string(&translation.value, interpolations);
            }
        }
        
        // Try fallback locale
        let fallback_locale = &config.fallback_locale;
        if fallback_locale != current_locale {
            if let Some(locale_translations) = translations.get(fallback_locale) {
                if let Some(translation) = locale_translations.get(key) {
                    return self.interpolate_string(&translation.value, interpolations);
                }
            }
        }
        
        // Record missing translation
        {
            let mut missing = self.missing_translations.lock().unwrap();
            let missing_key = format!("{}:{}", current_locale, key);
            if !missing.contains(&missing_key) {
                missing.push(missing_key);
            }
        }
        
        // Return default value or key
        if let Some(translation_key) = keys.get(key) {
            self.interpolate_string(&translation_key.default_value, interpolations)
        } else {
            key.to_string()
        }
    }

    pub fn translate_plural(&self, key: &str, count: i32, interpolations: Option<HashMap<String, String>>) -> String {
        let config = self.config.lock().unwrap();
        let translations = self.translations.lock().unwrap();
        
        let current_locale = &config.current_locale;
        if let Some(locale_translations) = translations.get(current_locale) {
            if let Some(translation) = locale_translations.get(key) {
                if let Some(ref plural_forms) = translation.plural_forms {
                    let plural_rule = self.get_plural_rule(&config.current_locale, count);
                    if let Some(plural_value) = plural_forms.get(&plural_rule) {
                        return self.interpolate_string(plural_value, interpolations);
                    }
                }
            }
        }
        
        // Fallback to regular translation
        self.translate(key, interpolations)
    }

    fn interpolate_string(&self, template: &str, interpolations: Option<HashMap<String, String>>) -> String {
        if let Some(vars) = interpolations {
            let mut result = template.to_string();
            for (key, value) in vars {
                let placeholder = format!("{{{{{}}}}}", key);
                result = result.replace(&placeholder, &value);
            }
            result
        } else {
            template.to_string()
        }
    }

    fn get_plural_rule(&self, locale: &str, count: i32) -> String {
        // Simplified plural rules - real implementation would be more complex
        match locale {
            locale if locale.starts_with("en") => {
                if count == 1 { "one" } else { "other" }
            },
            locale if locale.starts_with("fr") => {
                if count <= 1 { "one" } else { "other" }
            },
            locale if locale.starts_with("ru") => {
                match count % 100 {
                    11..=14 => "many",
                    _ => match count % 10 {
                        1 => "one",
                        2..=4 => "few",
                        _ => "many",
                    }
                }
            },
            _ => if count == 1 { "one" } else { "other" }
        }.to_string()
    }

    // Formatting
    pub fn format_number(&self, number: f64) -> String {
        let config = self.config.lock().unwrap();
        let fmt = &config.number_format;
        
        let mut result = format!("{:.2}", number);
        
        // Replace decimal separator
        if fmt.decimal_separator != "." {
            result = result.replace('.', &fmt.decimal_separator);
        }
        
        // Add thousands separators
        // Simplified implementation
        result
    }

    pub fn format_currency(&self, amount: f64) -> String {
        let config = self.config.lock().unwrap();
        let currency = &config.currency_settings;
        let formatted_number = self.format_number(amount);
        
        match currency.position {
            CurrencyPosition::Before => format!("{}{}", currency.symbol, formatted_number),
            CurrencyPosition::After => format!("{}{}", formatted_number, currency.symbol),
            CurrencyPosition::BeforeWithSpace => format!("{} {}", currency.symbol, formatted_number),
            CurrencyPosition::AfterWithSpace => format!("{} {}", formatted_number, currency.symbol),
        }
    }

    pub fn format_date(&self, timestamp: u64) -> String {
        // Simplified date formatting - would use chrono or similar in real implementation
        format!("Date: {}", timestamp)
    }

    // Utilities
    pub fn get_missing_translations(&self) -> Vec<String> {
        let missing = self.missing_translations.lock().unwrap();
        missing.clone()
    }

    pub fn get_translation_completion(&self, locale: &str) -> f32 {
        let config = self.config.lock().unwrap();
        config.available_locales.iter()
            .find(|l| l.code == locale)
            .map(|l| l.completion)
            .unwrap_or(0.0)
    }

    pub fn export_translations(&self, locale: &str) -> Result<String, String> {
        let translations = self.translations.lock().unwrap();
        
        if let Some(locale_translations) = translations.get(locale) {
            serde_json::to_string_pretty(locale_translations)
                .map_err(|e| format!("Failed to serialize translations: {}", e))
        } else {
            Err(format!("No translations found for locale: {}", locale))
        }
    }

    pub fn import_translations(&self, locale: &str, json_data: &str) -> Result<usize, String> {
        let new_translations: HashMap<String, Translation> = serde_json::from_str(json_data)
            .map_err(|e| format!("Failed to parse translations: {}", e))?;
        
        let count = new_translations.len();
        
        {
            let mut translations = self.translations.lock().unwrap();
            let locale_translations = translations
                .entry(locale.to_string())
                .or_insert_with(HashMap::new);
            
            locale_translations.extend(new_translations);
        }
        
        Ok(count)
    }
}
