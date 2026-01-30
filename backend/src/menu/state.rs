use crate::menu::config::{MenuConfig, MenuItem};
use std::collections::VecDeque;

#[derive(Debug, Clone)]
pub enum MenuState {
    MainMenu,
    Submenu { submenu_key: String },
}

#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
pub enum MenuAction {
    Redraw,                          // Redraw current menu (Enter, invalid input)
    EnterSubmenu(String),            // Navigate to submenu by key
    BackToMain,                      // Return to main menu from submenu
    LaunchService(String),           // Launch a service by service_name
    ExecuteCommand(String),          // Execute a command ("quit", "profile")
    ShowHelp,                        // Show help text
    Buffered,                        // Input was buffered (during transition)
    None,                            // No action (ignored input)
}

#[allow(dead_code)]
pub struct TypeAheadBuffer {
    buffer: VecDeque<char>,
    max_size: usize,
}

#[allow(dead_code)]
impl TypeAheadBuffer {
    pub fn new() -> Self {
        Self {
            buffer: VecDeque::with_capacity(16),
            max_size: 16,
        }
    }

    pub fn push(&mut self, ch: char) {
        if self.buffer.len() >= self.max_size {
            self.buffer.pop_front();
        }
        self.buffer.push_back(ch);
    }

    pub fn pop(&mut self) -> Option<char> {
        self.buffer.pop_front()
    }

    pub fn is_empty(&self) -> bool {
        self.buffer.is_empty()
    }

    pub fn clear(&mut self) {
        self.buffer.clear();
    }
}

pub struct MenuSession {
    state: MenuState,
    typeahead: TypeAheadBuffer,
    user_level: u8,
}

impl MenuSession {
    pub fn new(user_level: u8) -> Self {
        Self {
            state: MenuState::MainMenu,
            typeahead: TypeAheadBuffer::new(),
            user_level,
        }
    }

    pub fn state(&self) -> &MenuState {
        &self.state
    }

    pub fn user_level(&self) -> u8 {
        self.user_level
    }

    pub fn process_key(&mut self, key: char, config: &MenuConfig) -> MenuAction {
        // Universal actions
        if key == '?' {
            return MenuAction::ShowHelp;
        }
        if key == '\r' || key == '\n' {
            return MenuAction::Redraw;
        }

        match &self.state {
            MenuState::MainMenu => {
                let items = config.main_items(self.user_level);

                for item in items {
                    if item.matches_key(key) {
                        match item {
                            MenuItem::Submenu { submenu_key, .. } => {
                                self.state = MenuState::Submenu {
                                    submenu_key: submenu_key.clone(),
                                };
                                return MenuAction::EnterSubmenu(submenu_key.clone());
                            }
                            MenuItem::Service { service_name, .. } => {
                                return MenuAction::LaunchService(service_name.clone());
                            }
                            MenuItem::Command { command, .. } => {
                                return MenuAction::ExecuteCommand(command.clone());
                            }
                        }
                    }
                }

                MenuAction::Redraw
            }
            MenuState::Submenu { submenu_key } => {
                // Q/q always returns to main menu from submenu
                if key.eq_ignore_ascii_case(&'Q') {
                    self.state = MenuState::MainMenu;
                    return MenuAction::BackToMain;
                }

                let items = config.submenu_items(submenu_key, self.user_level);

                for item in items {
                    if item.matches_key(key) {
                        match item {
                            MenuItem::Service { service_name, .. } => {
                                return MenuAction::LaunchService(service_name.clone());
                            }
                            MenuItem::Command { command, .. } => {
                                return MenuAction::ExecuteCommand(command.clone());
                            }
                            MenuItem::Submenu { .. } => {
                                // Submenus shouldn't contain nested submenus, but handle gracefully
                                return MenuAction::Redraw;
                            }
                        }
                    }
                }

                MenuAction::Redraw
            }
        }
    }

    #[allow(dead_code)]
    pub fn buffer_key(&mut self, ch: char) {
        self.typeahead.push(ch);
    }

    pub fn drain_buffer(&mut self, config: &MenuConfig) -> Vec<MenuAction> {
        let mut actions = Vec::new();

        while let Some(ch) = self.typeahead.pop() {
            let action = self.process_key(ch, config);
            actions.push(action.clone());

            // Stop processing if we hit a terminal action (service launch or command)
            if matches!(action, MenuAction::LaunchService(_) | MenuAction::ExecuteCommand(_)) {
                break;
            }
        }

        actions
    }

    pub fn reset_to_main(&mut self) {
        self.state = MenuState::MainMenu;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_config() -> MenuConfig {
        MenuConfig {
            main: vec![
                MenuItem::Command {
                    hotkey: "Q".to_string(),
                    name: "Quit".to_string(),
                    command: "quit".to_string(),
                    min_level: 0,
                    order: 99,
                },
                MenuItem::Command {
                    hotkey: "P".to_string(),
                    name: "Profile".to_string(),
                    command: "profile".to_string(),
                    min_level: 0,
                    order: 98,
                },
                MenuItem::Submenu {
                    hotkey: "G".to_string(),
                    name: "Games".to_string(),
                    submenu_key: "games".to_string(),
                    min_level: 0,
                    order: 1,
                },
            ],
            games: vec![
                MenuItem::Service {
                    hotkey: "1".to_string(),
                    name: "Test Game".to_string(),
                    service_name: "test_game".to_string(),
                    min_level: 0,
                    order: 1,
                },
            ],
            ..Default::default()
        }
    }

    #[test]
    fn test_process_key_quit_at_main_menu() {
        let config = create_test_config();
        let mut session = MenuSession::new(0);

        let action = session.process_key('Q', &config);
        assert_eq!(action, MenuAction::ExecuteCommand("quit".to_string()));
    }

    #[test]
    fn test_process_key_quit_at_submenu_returns_to_main() {
        let config = create_test_config();
        let mut session = MenuSession::new(0);

        // Enter submenu first
        session.state = MenuState::Submenu {
            submenu_key: "games".to_string(),
        };

        let action = session.process_key('Q', &config);
        assert_eq!(action, MenuAction::BackToMain);
        assert!(matches!(session.state, MenuState::MainMenu));
    }

    #[test]
    fn test_typeahead_buffer_fifo_ordering() {
        let mut buffer = TypeAheadBuffer::new();

        buffer.push('A');
        buffer.push('B');
        buffer.push('C');

        assert_eq!(buffer.pop(), Some('A'));
        assert_eq!(buffer.pop(), Some('B'));
        assert_eq!(buffer.pop(), Some('C'));
        assert_eq!(buffer.pop(), None);
    }

    #[test]
    fn test_typeahead_buffer_respects_max_size() {
        let mut buffer = TypeAheadBuffer::new();

        // Fill beyond max_size (16)
        for i in 0..20 {
            buffer.push(char::from_digit(i % 10, 10).unwrap());
        }

        // Should only have 16 items (last 16)
        let mut count = 0;
        while buffer.pop().is_some() {
            count += 1;
        }

        assert_eq!(count, 16);
    }

    #[test]
    fn test_drain_buffer_stops_at_launch_service() {
        let config = create_test_config();
        let mut session = MenuSession::new(0);

        // Buffer: G (enter games submenu) then 1 (launch game)
        session.buffer_key('G');
        session.buffer_key('1');

        let actions = session.drain_buffer(&config);

        // Should have two actions: EnterSubmenu and LaunchService
        assert_eq!(actions.len(), 2);
        assert!(matches!(actions[0], MenuAction::EnterSubmenu(_)));
        assert!(matches!(actions[1], MenuAction::LaunchService(_)));

        // Buffer should now be empty
        assert!(session.typeahead.is_empty());
    }

    #[test]
    fn test_enter_key_returns_redraw() {
        let config = create_test_config();
        let mut session = MenuSession::new(0);

        assert_eq!(session.process_key('\r', &config), MenuAction::Redraw);
        assert_eq!(session.process_key('\n', &config), MenuAction::Redraw);
    }

    #[test]
    fn test_help_key_returns_show_help() {
        let config = create_test_config();
        let mut session = MenuSession::new(0);

        assert_eq!(session.process_key('?', &config), MenuAction::ShowHelp);
    }

    #[test]
    fn test_invalid_key_returns_redraw() {
        let config = create_test_config();
        let mut session = MenuSession::new(0);

        assert_eq!(session.process_key('X', &config), MenuAction::Redraw);
    }

    #[test]
    fn test_enter_submenu_changes_state() {
        let config = create_test_config();
        let mut session = MenuSession::new(0);

        let action = session.process_key('G', &config);

        assert_eq!(action, MenuAction::EnterSubmenu("games".to_string()));
        assert!(matches!(
            session.state,
            MenuState::Submenu { submenu_key } if submenu_key == "games"
        ));
    }

    #[test]
    fn test_reset_to_main() {
        let mut session = MenuSession::new(0);

        session.state = MenuState::Submenu {
            submenu_key: "games".to_string(),
        };

        session.reset_to_main();

        assert!(matches!(session.state, MenuState::MainMenu));
    }
}
