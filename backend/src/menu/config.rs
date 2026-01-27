use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum MenuItem {
    Service {
        hotkey: String,
        name: String,
        service_name: String,
        #[serde(default)]
        min_level: u8,
        #[serde(default)]
        order: i32,
    },
    Submenu {
        hotkey: String,
        name: String,
        submenu_key: String,
        #[serde(default)]
        min_level: u8,
        #[serde(default)]
        order: i32,
    },
    Command {
        hotkey: String,
        name: String,
        command: String,
        #[serde(default)]
        min_level: u8,
        #[serde(default)]
        order: i32,
    },
}

impl MenuItem {
    pub fn hotkey(&self) -> &str {
        match self {
            MenuItem::Service { hotkey, .. } => hotkey,
            MenuItem::Submenu { hotkey, .. } => hotkey,
            MenuItem::Command { hotkey, .. } => hotkey,
        }
    }

    pub fn name(&self) -> &str {
        match self {
            MenuItem::Service { name, .. } => name,
            MenuItem::Submenu { name, .. } => name,
            MenuItem::Command { name, .. } => name,
        }
    }

    pub fn order(&self) -> i32 {
        match self {
            MenuItem::Service { order, .. } => *order,
            MenuItem::Submenu { order, .. } => *order,
            MenuItem::Command { order, .. } => *order,
        }
    }

    pub fn min_level(&self) -> u8 {
        match self {
            MenuItem::Service { min_level, .. } => *min_level,
            MenuItem::Submenu { min_level, .. } => *min_level,
            MenuItem::Command { min_level, .. } => *min_level,
        }
    }

    pub fn matches_key(&self, key: char) -> bool {
        let hotkey = self.hotkey();
        if hotkey.len() != 1 {
            return false;
        }
        hotkey.chars().next().unwrap().eq_ignore_ascii_case(&key)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MenuConfig {
    #[serde(default)]
    pub main: Vec<MenuItem>,
    #[serde(default)]
    pub games: Vec<MenuItem>,
    #[serde(default)]
    pub mail: Vec<MenuItem>,
    #[serde(default)]
    pub chat: Vec<MenuItem>,
    #[serde(default)]
    pub news: Vec<MenuItem>,
}

impl MenuConfig {
    /// Returns main menu items filtered by user level and sorted by order
    pub fn main_items(&self, user_level: u8) -> Vec<&MenuItem> {
        let mut items: Vec<&MenuItem> = self
            .main
            .iter()
            .filter(|item| item.min_level() <= user_level)
            .collect();
        items.sort_by_key(|item| item.order());
        items
    }

    /// Returns submenu items filtered by user level and sorted by order
    pub fn submenu_items(&self, key: &str, user_level: u8) -> Vec<&MenuItem> {
        let items = match key {
            "games" => &self.games,
            "mail" => &self.mail,
            "chat" => &self.chat,
            "news" => &self.news,
            _ => return Vec::new(),
        };

        let mut filtered: Vec<&MenuItem> = items
            .iter()
            .filter(|item| item.min_level() <= user_level)
            .collect();
        filtered.sort_by_key(|item| item.order());
        filtered
    }

    /// Returns human-readable name for submenu key
    pub fn submenu_name(&self, key: &str) -> &str {
        match key {
            "games" => "Games",
            "mail" => "Mail",
            "chat" => "Chat",
            "news" => "News & Bulletins",
            _ => "Unknown",
        }
    }
}
