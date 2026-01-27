use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs;
use crate::menu::MenuConfig;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Config {
    pub server: ServerConfig,
    pub terminal: TerminalConfig,
    pub services: Vec<ServiceConfig>,
    #[serde(default)]
    pub auth: AuthConfig,
    #[serde(default)]
    pub connection: ConnectionConfig,
    pub email: Option<EmailConfig>,
    #[serde(default)]
    pub menu: MenuConfig,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct TerminalConfig {
    pub cols: u16,
    pub rows: u16,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ServiceConfig {
    pub name: String,
    pub enabled: bool,
    pub description: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct AuthConfig {
    #[serde(default = "default_max_login_attempts")]
    pub max_login_attempts: u32,
    #[serde(default = "default_lockout_minutes")]
    pub lockout_minutes: u32,
    #[serde(default = "default_session_duration_hours")]
    pub session_duration_hours: u32,
    #[serde(default = "default_verification_code_expiry_hours")]
    pub verification_code_expiry_hours: u32,
    #[serde(default)]
    pub sysop_handles: Vec<String>,
}

fn default_max_login_attempts() -> u32 {
    5
}
fn default_lockout_minutes() -> u32 {
    15
}
fn default_session_duration_hours() -> u32 {
    24
}
fn default_verification_code_expiry_hours() -> u32 {
    24
}

impl Default for AuthConfig {
    fn default() -> Self {
        Self {
            max_login_attempts: default_max_login_attempts(),
            lockout_minutes: default_lockout_minutes(),
            session_duration_hours: default_session_duration_hours(),
            verification_code_expiry_hours: default_verification_code_expiry_hours(),
            sysop_handles: Vec::new(),
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ConnectionConfig {
    #[serde(default = "default_max_nodes")]
    pub max_nodes: u16,
    #[serde(default = "default_baud_simulation_cps")]
    pub baud_simulation_cps: u32,
    #[serde(default = "default_line_busy_mode")]
    pub line_busy_mode: String,
    #[serde(default = "default_idle_timeout_minutes")]
    pub idle_timeout_minutes: u32,
    #[serde(default = "default_idle_warning_bell")]
    pub idle_warning_bell: bool,
    #[serde(default)]
    pub ceremony_skippable: bool,
    #[serde(default = "default_tagline")]
    pub tagline: String,
    #[serde(default = "default_show_last_callers")]
    pub show_last_callers: bool,
}

fn default_max_nodes() -> u16 {
    16
}
fn default_baud_simulation_cps() -> u32 {
    960
}
fn default_line_busy_mode() -> String {
    "disconnect".to_string()
}
fn default_idle_timeout_minutes() -> u32 {
    15
}
fn default_idle_warning_bell() -> bool {
    true
}
fn default_tagline() -> String {
    "Welcome to The Construct BBS".to_string()
}
fn default_show_last_callers() -> bool {
    true
}

impl Default for ConnectionConfig {
    fn default() -> Self {
        Self {
            max_nodes: default_max_nodes(),
            baud_simulation_cps: default_baud_simulation_cps(),
            line_busy_mode: default_line_busy_mode(),
            idle_timeout_minutes: default_idle_timeout_minutes(),
            idle_warning_bell: default_idle_warning_bell(),
            ceremony_skippable: false,
            tagline: default_tagline(),
            show_last_callers: default_show_last_callers(),
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct EmailConfig {
    pub smtp_host: String,
    #[serde(default = "default_smtp_port")]
    pub smtp_port: u16,
    pub smtp_username: String,
    pub smtp_password: String,
    pub from_address: String,
    #[serde(default = "default_from_name")]
    pub from_name: String,
}

fn default_smtp_port() -> u16 {
    587
}
fn default_from_name() -> String {
    "The Construct BBS".to_string()
}

impl Config {
    pub fn load(path: &str) -> Result<Self, Box<dyn Error>> {
        let content = fs::read_to_string(path)?;
        let config: Config = toml::from_str(&content)?;
        Ok(config)
    }
}
