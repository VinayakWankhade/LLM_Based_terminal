use serde::{Deserialize, Serialize};
use chrono::Timelike;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::fs;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: f32,
}

impl Color {
    pub fn new(r: u8, g: u8, b: u8, a: f32) -> Self {
        Self { r, g, b, a }
    }

    pub fn from_hex(hex: &str) -> Result<Self, String> {
        let hex = hex.trim_start_matches('#');
        if hex.len() != 6 && hex.len() != 8 {
            return Err("Invalid hex color format".to_string());
        }

        let r = u8::from_str_radix(&hex[0..2], 16)
            .map_err(|_| "Invalid red component")?;
        let g = u8::from_str_radix(&hex[2..4], 16)
            .map_err(|_| "Invalid green component")?;
        let b = u8::from_str_radix(&hex[4..6], 16)
            .map_err(|_| "Invalid blue component")?;

        let a = if hex.len() == 8 {
            u8::from_str_radix(&hex[6..8], 16)
                .map_err(|_| "Invalid alpha component")? as f32 / 255.0
        } else {
            1.0
        };

        Ok(Self::new(r, g, b, a))
    }

    pub fn to_hex(&self) -> String {
        if self.a == 1.0 {
            format!("#{:02x}{:02x}{:02x}", self.r, self.g, self.b)
        } else {
            format!("#{:02x}{:02x}{:02x}{:02x}", 
                self.r, self.g, self.b, (self.a * 255.0) as u8)
        }
    }

    pub fn to_rgb(&self) -> String {
        format!("rgb({}, {}, {})", self.r, self.g, self.b)
    }

    pub fn to_rgba(&self) -> String {
        format!("rgba({}, {}, {}, {})", self.r, self.g, self.b, self.a)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FontConfig {
    pub family: String,
    pub size: u16,
    pub weight: FontWeight,
    pub style: FontStyle,
    pub line_height: f32,
    pub letter_spacing: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum FontWeight {
    Thin,
    ExtraLight,
    Light,
    Normal,
    Medium,
    SemiBold,
    Bold,
    ExtraBold,
    Black,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum FontStyle {
    Normal,
    Italic,
    Oblique,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColorScheme {
    pub name: String,
    pub is_dark: bool,
    pub foreground: Color,
    pub background: Color,
    pub cursor: Color,
    pub selection: Color,
    pub black: Color,
    pub red: Color,
    pub green: Color,
    pub yellow: Color,
    pub blue: Color,
    pub magenta: Color,
    pub cyan: Color,
    pub white: Color,
    pub bright_black: Color,
    pub bright_red: Color,
    pub bright_green: Color,
    pub bright_yellow: Color,
    pub bright_blue: Color,
    pub bright_magenta: Color,
    pub bright_cyan: Color,
    pub bright_white: Color,
    pub accent: Color,
    pub warning: Color,
    pub error: Color,
    pub success: Color,
    pub info: Color,
}

impl ColorScheme {
    pub fn default_dark() -> Self {
        Self {
            name: "Default Dark".to_string(),
            is_dark: true,
            foreground: Color::from_hex("#ffffff").unwrap(),
            background: Color::from_hex("#1e1e1e").unwrap(),
            cursor: Color::from_hex("#ffffff").unwrap(),
            selection: Color::new(255, 255, 255, 0.2),
            black: Color::from_hex("#000000").unwrap(),
            red: Color::from_hex("#f14c4c").unwrap(),
            green: Color::from_hex("#23d18b").unwrap(),
            yellow: Color::from_hex("#f5f543").unwrap(),
            blue: Color::from_hex("#3b8eea").unwrap(),
            magenta: Color::from_hex("#d670d6").unwrap(),
            cyan: Color::from_hex("#29b8db").unwrap(),
            white: Color::from_hex("#e5e5e5").unwrap(),
            bright_black: Color::from_hex("#666666").unwrap(),
            bright_red: Color::from_hex("#f14c4c").unwrap(),
            bright_green: Color::from_hex("#23d18b").unwrap(),
            bright_yellow: Color::from_hex("#f5f543").unwrap(),
            bright_blue: Color::from_hex("#3b8eea").unwrap(),
            bright_magenta: Color::from_hex("#d670d6").unwrap(),
            bright_cyan: Color::from_hex("#29b8db").unwrap(),
            bright_white: Color::from_hex("#ffffff").unwrap(),
            accent: Color::from_hex("#3b8eea").unwrap(),
            warning: Color::from_hex("#f5f543").unwrap(),
            error: Color::from_hex("#f14c4c").unwrap(),
            success: Color::from_hex("#23d18b").unwrap(),
            info: Color::from_hex("#3b8eea").unwrap(),
        }
    }

    pub fn default_light() -> Self {
        Self {
            name: "Default Light".to_string(),
            is_dark: false,
            foreground: Color::from_hex("#000000").unwrap(),
            background: Color::from_hex("#ffffff").unwrap(),
            cursor: Color::from_hex("#000000").unwrap(),
            selection: Color::new(0, 0, 0, 0.2),
            black: Color::from_hex("#000000").unwrap(),
            red: Color::from_hex("#cd3131").unwrap(),
            green: Color::from_hex("#00bc00").unwrap(),
            yellow: Color::from_hex("#949800").unwrap(),
            blue: Color::from_hex("#0451a5").unwrap(),
            magenta: Color::from_hex("#bc05bc").unwrap(),
            cyan: Color::from_hex("#0598bc").unwrap(),
            white: Color::from_hex("#555555").unwrap(),
            bright_black: Color::from_hex("#666666").unwrap(),
            bright_red: Color::from_hex("#cd3131").unwrap(),
            bright_green: Color::from_hex("#14ce14").unwrap(),
            bright_yellow: Color::from_hex("#b5ba00").unwrap(),
            bright_blue: Color::from_hex("#0451a5").unwrap(),
            bright_magenta: Color::from_hex("#bc05bc").unwrap(),
            bright_cyan: Color::from_hex("#0598bc").unwrap(),
            bright_white: Color::from_hex("#a5a5a5").unwrap(),
            accent: Color::from_hex("#0451a5").unwrap(),
            warning: Color::from_hex("#949800").unwrap(),
            error: Color::from_hex("#cd3131").unwrap(),
            success: Color::from_hex("#00bc00").unwrap(),
            info: Color::from_hex("#0451a5").unwrap(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Theme {
    pub id: String,
    pub name: String,
    pub description: String,
    pub author: String,
    pub version: String,
    pub color_scheme: ColorScheme,
    pub font: FontConfig,
    pub ui_colors: HashMap<String, Color>,
    pub ui_spacing: HashMap<String, f32>,
    pub ui_borders: HashMap<String, BorderConfig>,
    pub ui_shadows: HashMap<String, ShadowConfig>,
    pub animations: AnimationConfig,
    pub created_at: u64,
    pub updated_at: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BorderConfig {
    pub width: f32,
    pub color: Color,
    pub style: BorderStyle,
    pub radius: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum BorderStyle {
    Solid,
    Dashed,
    Dotted,
    None,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShadowConfig {
    pub offset_x: f32,
    pub offset_y: f32,
    pub blur_radius: f32,
    pub spread_radius: f32,
    pub color: Color,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnimationConfig {
    pub duration: f32,
    pub easing: EasingFunction,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum EasingFunction {
    Linear,
    EaseIn,
    EaseOut,
    EaseInOut,
    Cubic,
    Bounce,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeVariation {
    pub base_theme_id: String,
    pub name: String,
    pub color_overrides: HashMap<String, Color>,
    pub font_overrides: Option<FontConfig>,
    pub ui_overrides: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeCollection {
    pub name: String,
    pub description: String,
    pub themes: Vec<String>, // theme IDs
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemePreferences {
    pub current_theme_id: String,
    pub auto_switch_enabled: bool,
    pub light_theme_id: String,
    pub dark_theme_id: String,
    pub switch_time_dawn: String, // "06:00"
    pub switch_time_dusk: String, // "18:00"
    pub follow_system: bool,
    pub high_contrast: bool,
    pub reduce_motion: bool,
}

pub struct ThemeManager {
    themes: Arc<Mutex<HashMap<String, Theme>>>,
    variations: Arc<Mutex<HashMap<String, ThemeVariation>>>,
    collections: Arc<Mutex<HashMap<String, ThemeCollection>>>,
    preferences: Arc<Mutex<ThemePreferences>>,
    themes_directory: String,
    hot_reload_enabled: bool,
}

impl ThemeManager {
    pub fn new(themes_directory: String) -> Self {
        let mut themes = HashMap::new();
        
        // Add default themes
        let dark_theme = Self::create_default_dark_theme();
        let light_theme = Self::create_default_light_theme();
        
        themes.insert(dark_theme.id.clone(), dark_theme);
        themes.insert(light_theme.id.clone(), light_theme);

        let default_preferences = ThemePreferences {
            current_theme_id: "default_dark".to_string(),
            auto_switch_enabled: false,
            light_theme_id: "default_light".to_string(),
            dark_theme_id: "default_dark".to_string(),
            switch_time_dawn: "06:00".to_string(),
            switch_time_dusk: "18:00".to_string(),
            follow_system: true,
            high_contrast: false,
            reduce_motion: false,
        };

        Self {
            themes: Arc::new(Mutex::new(themes)),
            variations: Arc::new(Mutex::new(HashMap::new())),
            collections: Arc::new(Mutex::new(HashMap::new())),
            preferences: Arc::new(Mutex::new(default_preferences)),
            themes_directory,
            hot_reload_enabled: true,
        }
    }

    fn create_default_dark_theme() -> Theme {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let mut ui_colors = HashMap::new();
        ui_colors.insert("panel_background".to_string(), Color::from_hex("#2d2d30").unwrap());
        ui_colors.insert("border".to_string(), Color::from_hex("#3f3f46").unwrap());
        ui_colors.insert("hover".to_string(), Color::new(255, 255, 255, 0.1));
        ui_colors.insert("active".to_string(), Color::new(255, 255, 255, 0.2));

        let mut ui_spacing = HashMap::new();
        ui_spacing.insert("padding_small".to_string(), 4.0);
        ui_spacing.insert("padding_medium".to_string(), 8.0);
        ui_spacing.insert("padding_large".to_string(), 16.0);
        ui_spacing.insert("margin_small".to_string(), 4.0);
        ui_spacing.insert("margin_medium".to_string(), 8.0);
        ui_spacing.insert("margin_large".to_string(), 16.0);

        let mut ui_borders = HashMap::new();
        ui_borders.insert("default".to_string(), BorderConfig {
            width: 1.0,
            color: Color::from_hex("#3f3f46").unwrap(),
            style: BorderStyle::Solid,
            radius: 4.0,
        });

        let mut ui_shadows = HashMap::new();
        ui_shadows.insert("default".to_string(), ShadowConfig {
            offset_x: 0.0,
            offset_y: 2.0,
            blur_radius: 8.0,
            spread_radius: 0.0,
            color: Color::new(0, 0, 0, 0.2),
        });

        Theme {
            id: "default_dark".to_string(),
            name: "Default Dark".to_string(),
            description: "The default dark theme".to_string(),
            author: "Terminal Emulator".to_string(),
            version: "1.0.0".to_string(),
            color_scheme: ColorScheme::default_dark(),
            font: FontConfig {
                family: "Fira Code".to_string(),
                size: 14,
                weight: FontWeight::Normal,
                style: FontStyle::Normal,
                line_height: 1.2,
                letter_spacing: 0.0,
            },
            ui_colors,
            ui_spacing,
            ui_borders,
            ui_shadows,
            animations: AnimationConfig {
                duration: 0.2,
                easing: EasingFunction::EaseInOut,
                enabled: true,
            },
            created_at: timestamp,
            updated_at: timestamp,
        }
    }

    fn create_default_light_theme() -> Theme {
        let mut theme = Self::create_default_dark_theme();
        theme.id = "default_light".to_string();
        theme.name = "Default Light".to_string();
        theme.description = "The default light theme".to_string();
        theme.color_scheme = ColorScheme::default_light();
        
        // Update UI colors for light theme
        theme.ui_colors.insert("panel_background".to_string(), Color::from_hex("#f3f3f3").unwrap());
        theme.ui_colors.insert("border".to_string(), Color::from_hex("#e1e1e1").unwrap());
        theme.ui_colors.insert("hover".to_string(), Color::new(0, 0, 0, 0.1));
        theme.ui_colors.insert("active".to_string(), Color::new(0, 0, 0, 0.2));

        theme
    }

    pub async fn load_themes_from_directory(&self) -> Result<usize, String> {
        let mut loaded_count = 0;
        let mut entries = fs::read_dir(&self.themes_directory).await
            .map_err(|e| format!("Failed to read themes directory: {}", e))?;

        while let Some(entry) = entries.next_entry().await
            .map_err(|e| format!("Failed to read directory entry: {}", e))? {
            
            let path = entry.path();
            if path.extension().map_or(false, |ext| ext == "json") {
                match self.load_theme_from_file(&path.to_string_lossy()).await {
                    Ok(_) => loaded_count += 1,
                    Err(e) => eprintln!("Failed to load theme from {:?}: {}", path, e),
                }
            }
        }

        Ok(loaded_count)
    }

    pub async fn load_theme_from_file(&self, file_path: &str) -> Result<String, String> {
        let content = fs::read_to_string(file_path).await
            .map_err(|e| format!("Failed to read theme file: {}", e))?;

        let theme: Theme = serde_json::from_str(&content)
            .map_err(|e| format!("Failed to parse theme JSON: {}", e))?;

        let theme_id = theme.id.clone();
        
        {
            let mut themes = self.themes.lock().unwrap();
            themes.insert(theme_id.clone(), theme);
        }

        Ok(theme_id)
    }

    pub async fn save_theme_to_file(&self, theme_id: &str, file_path: &str) -> Result<(), String> {
        let theme = {
            let themes = self.themes.lock().unwrap();
            themes.get(theme_id).cloned()
                .ok_or_else(|| format!("Theme {} not found", theme_id))?
        };

        let json = serde_json::to_string_pretty(&theme)
            .map_err(|e| format!("Failed to serialize theme: {}", e))?;

        fs::write(file_path, json).await
            .map_err(|e| format!("Failed to write theme file: {}", e))?;

        Ok(())
    }

    pub fn get_all_themes(&self) -> Vec<Theme> {
        let themes = self.themes.lock().unwrap();
        themes.values().cloned().collect()
    }

    pub fn get_theme(&self, theme_id: &str) -> Option<Theme> {
        let themes = self.themes.lock().unwrap();
        themes.get(theme_id).cloned()
    }

    pub fn get_current_theme(&self) -> Option<Theme> {
        let theme_id = {
            let preferences = self.preferences.lock().unwrap();
            preferences.current_theme_id.clone()
        };
        self.get_theme(&theme_id)
    }

    pub fn set_current_theme(&self, theme_id: String) -> Result<(), String> {
        {
            let themes = self.themes.lock().unwrap();
            if !themes.contains_key(&theme_id) {
                return Err(format!("Theme {} not found", theme_id));
            }
        }

        {
            let mut preferences = self.preferences.lock().unwrap();
            preferences.current_theme_id = theme_id;
        }

        Ok(())
    }

    pub fn add_theme(&self, mut theme: Theme) -> Result<String, String> {
        // Ensure unique ID
        let mut counter = 1;
        let original_id = theme.id.clone();
        
        {
            let themes = self.themes.lock().unwrap();
            while themes.contains_key(&theme.id) {
                theme.id = format!("{}_{}", original_id, counter);
                counter += 1;
            }
        }

        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        theme.created_at = timestamp;
        theme.updated_at = timestamp;
        
        let theme_id = theme.id.clone();
        
        {
            let mut themes = self.themes.lock().unwrap();
            themes.insert(theme_id.clone(), theme);
        }

        Ok(theme_id)
    }

    pub fn update_theme(&self, theme_id: &str, mut updated_theme: Theme) -> Result<(), String> {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        updated_theme.id = theme_id.to_string();
        updated_theme.updated_at = timestamp;

        {
            let mut themes = self.themes.lock().unwrap();
            if !themes.contains_key(theme_id) {
                return Err(format!("Theme {} not found", theme_id));
            }
            themes.insert(theme_id.to_string(), updated_theme);
        }

        Ok(())
    }

    pub fn remove_theme(&self, theme_id: &str) -> Result<(), String> {
        {
            let mut themes = self.themes.lock().unwrap();
            if !themes.contains_key(theme_id) {
                return Err(format!("Theme {} not found", theme_id));
            }
            themes.remove(theme_id);
        }

        // Update preferences if this was the current theme
        {
            let mut preferences = self.preferences.lock().unwrap();
            if preferences.current_theme_id == theme_id {
                preferences.current_theme_id = "default_dark".to_string();
            }
        }

        Ok(())
    }

    pub fn create_variation(&self, base_theme_id: &str, variation: ThemeVariation) -> Result<String, String> {
        {
            let themes = self.themes.lock().unwrap();
            if !themes.contains_key(base_theme_id) {
                return Err(format!("Base theme {} not found", base_theme_id));
            }
        }

        let variation_id = format!("{}_{}", base_theme_id, variation.name.to_lowercase().replace(' ', "_"));
        
        {
            let mut variations = self.variations.lock().unwrap();
            variations.insert(variation_id.clone(), variation);
        }

        Ok(variation_id)
    }

    pub fn apply_variation(&self, base_theme_id: &str, variation_id: &str) -> Result<Theme, String> {
        let base_theme = self.get_theme(base_theme_id)
            .ok_or_else(|| format!("Base theme {} not found", base_theme_id))?;

        let variation = {
            let variations = self.variations.lock().unwrap();
            variations.get(variation_id).cloned()
                .ok_or_else(|| format!("Variation {} not found", variation_id))?
        };

        let mut theme = base_theme;
        theme.id = format!("{}_{}", base_theme_id, variation_id);
        theme.name = format!("{} - {}", theme.name, variation.name);

        // Apply color overrides
        for (key, color) in variation.color_overrides {
            match key.as_str() {
                "foreground" => theme.color_scheme.foreground = color,
                "background" => theme.color_scheme.background = color,
                "cursor" => theme.color_scheme.cursor = color,
                "selection" => theme.color_scheme.selection = color,
                "accent" => theme.color_scheme.accent = color,
                _ => {
                    theme.ui_colors.insert(key, color);
                }
            }
        }

        // Apply font overrides
        if let Some(font_overrides) = variation.font_overrides {
            theme.font = font_overrides;
        }

        Ok(theme)
    }

    pub fn get_preferences(&self) -> ThemePreferences {
        let preferences = self.preferences.lock().unwrap();
        preferences.clone()
    }

    pub fn update_preferences(&self, new_preferences: ThemePreferences) {
        let mut preferences = self.preferences.lock().unwrap();
        *preferences = new_preferences;
    }

    pub fn should_auto_switch_theme(&self) -> Option<String> {
        let preferences = self.preferences.lock().unwrap();
        
        if !preferences.auto_switch_enabled {
            return None;
        }

        // Simple time-based switching (in a real implementation, you'd use proper time libraries)
        let current_hour = chrono::Utc::now().hour();
        let dawn_hour = preferences.switch_time_dawn
            .split(':')
            .next()
            .and_then(|h| h.parse::<u32>().ok())
            .unwrap_or(6);
        let dusk_hour = preferences.switch_time_dusk
            .split(':')
            .next()
            .and_then(|h| h.parse::<u32>().ok())
            .unwrap_or(18);

        if current_hour >= dawn_hour && current_hour < dusk_hour {
            Some(preferences.light_theme_id.clone())
        } else {
            Some(preferences.dark_theme_id.clone())
        }
    }

    pub fn export_theme(&self, theme_id: &str) -> Result<String, String> {
        let theme = self.get_theme(theme_id)
            .ok_or_else(|| format!("Theme {} not found", theme_id))?;

        serde_json::to_string_pretty(&theme)
            .map_err(|e| format!("Failed to serialize theme: {}", e))
    }

    pub fn import_theme(&self, json_data: &str) -> Result<String, String> {
        let theme: Theme = serde_json::from_str(json_data)
            .map_err(|e| format!("Failed to parse theme JSON: {}", e))?;

        self.add_theme(theme)
    }

    pub fn get_theme_preview(&self, theme_id: &str) -> Option<HashMap<String, String>> {
        let theme = self.get_theme(theme_id)?;
        
        let mut preview = HashMap::new();
        preview.insert("background".to_string(), theme.color_scheme.background.to_hex());
        preview.insert("foreground".to_string(), theme.color_scheme.foreground.to_hex());
        preview.insert("accent".to_string(), theme.color_scheme.accent.to_hex());
        preview.insert("red".to_string(), theme.color_scheme.red.to_hex());
        preview.insert("green".to_string(), theme.color_scheme.green.to_hex());
        preview.insert("blue".to_string(), theme.color_scheme.blue.to_hex());
        preview.insert("yellow".to_string(), theme.color_scheme.yellow.to_hex());
        
        Some(preview)
    }

    pub fn search_themes(&self, query: &str, tags: Option<Vec<String>>) -> Vec<Theme> {
        let themes = self.themes.lock().unwrap();
        let query_lower = query.to_lowercase();
        
        themes.values()
            .filter(|theme| {
                let name_match = theme.name.to_lowercase().contains(&query_lower);
                let desc_match = theme.description.to_lowercase().contains(&query_lower);
                let author_match = theme.author.to_lowercase().contains(&query_lower);
                
                name_match || desc_match || author_match
            })
            .cloned()
            .collect()
    }

    pub fn get_css_variables(&self, theme_id: &str) -> Result<String, String> {
        let theme = self.get_theme(theme_id)
            .ok_or_else(|| format!("Theme {} not found", theme_id))?;

        let mut css = String::from(":root {\n");
        
        // Color scheme variables
        css.push_str(&format!("  --color-foreground: {};\n", theme.color_scheme.foreground.to_hex()));
        css.push_str(&format!("  --color-background: {};\n", theme.color_scheme.background.to_hex()));
        css.push_str(&format!("  --color-cursor: {};\n", theme.color_scheme.cursor.to_hex()));
        css.push_str(&format!("  --color-selection: {};\n", theme.color_scheme.selection.to_rgba()));
        css.push_str(&format!("  --color-accent: {};\n", theme.color_scheme.accent.to_hex()));
        css.push_str(&format!("  --color-error: {};\n", theme.color_scheme.error.to_hex()));
        css.push_str(&format!("  --color-warning: {};\n", theme.color_scheme.warning.to_hex()));
        css.push_str(&format!("  --color-success: {};\n", theme.color_scheme.success.to_hex()));
        css.push_str(&format!("  --color-info: {};\n", theme.color_scheme.info.to_hex()));

        // ANSI colors
        css.push_str(&format!("  --ansi-black: {};\n", theme.color_scheme.black.to_hex()));
        css.push_str(&format!("  --ansi-red: {};\n", theme.color_scheme.red.to_hex()));
        css.push_str(&format!("  --ansi-green: {};\n", theme.color_scheme.green.to_hex()));
        css.push_str(&format!("  --ansi-yellow: {};\n", theme.color_scheme.yellow.to_hex()));
        css.push_str(&format!("  --ansi-blue: {};\n", theme.color_scheme.blue.to_hex()));
        css.push_str(&format!("  --ansi-magenta: {};\n", theme.color_scheme.magenta.to_hex()));
        css.push_str(&format!("  --ansi-cyan: {};\n", theme.color_scheme.cyan.to_hex()));
        css.push_str(&format!("  --ansi-white: {};\n", theme.color_scheme.white.to_hex()));

        // Font variables
        css.push_str(&format!("  --font-family: '{}';\n", theme.font.family));
        css.push_str(&format!("  --font-size: {}px;\n", theme.font.size));
        css.push_str(&format!("  --font-weight: {:?};\n", theme.font.weight));
        css.push_str(&format!("  --line-height: {};\n", theme.font.line_height));
        css.push_str(&format!("  --letter-spacing: {}px;\n", theme.font.letter_spacing));

        // UI color variables
        for (key, color) in &theme.ui_colors {
            css.push_str(&format!("  --ui-{}: {};\n", key.replace('_', "-"), color.to_hex()));
        }

        // Spacing variables
        for (key, value) in &theme.ui_spacing {
            css.push_str(&format!("  --spacing-{}: {}px;\n", key.replace('_', "-"), value));
        }

        // Animation variables
        css.push_str(&format!("  --animation-duration: {}s;\n", theme.animations.duration));
        css.push_str(&format!("  --animation-enabled: {};\n", if theme.animations.enabled { "1" } else { "0" }));

        css.push_str("}\n");
        
        Ok(css)
    }
}
