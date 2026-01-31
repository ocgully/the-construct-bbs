//! LLM integration for Xodia
//!
//! Supports multiple LLM providers:
//! - Ollama (local) - preferred for self-hosted
//! - OpenAI (cloud)
//! - Anthropic (cloud)
//!
//! The LLM serves as the Dungeon Master, providing narrative descriptions
//! and interpreting complex player commands.

use serde::{Serialize, Deserialize};
use std::time::Duration;

/// LLM provider configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LlmProvider {
    /// Ollama local LLM
    Ollama {
        base_url: String,
        model: String,
    },
    /// OpenAI API
    OpenAI {
        api_key: String,
        model: String,
    },
    /// Anthropic API
    Anthropic {
        api_key: String,
        model: String,
    },
    /// Offline mode - no LLM available
    Offline,
}

impl Default for LlmProvider {
    fn default() -> Self {
        // Default to Ollama on localhost
        LlmProvider::Ollama {
            base_url: "http://localhost:11434".to_string(),
            model: "llama3.2".to_string(),
        }
    }
}

/// Configuration for LLM behavior
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmConfig {
    pub provider: LlmProvider,
    pub timeout_seconds: u64,
    pub max_tokens: u32,
    pub temperature: f32,
    pub retry_attempts: u32,
}

impl Default for LlmConfig {
    fn default() -> Self {
        Self {
            provider: LlmProvider::default(),
            timeout_seconds: 30,
            max_tokens: 500,
            temperature: 0.7,
            retry_attempts: 2,
        }
    }
}

/// Response from LLM
#[derive(Debug, Clone)]
pub struct LlmResponse {
    pub content: String,
    pub tokens_used: u32,
    pub provider: String,
    pub success: bool,
    pub error: Option<String>,
}

impl LlmResponse {
    pub fn success(content: String, tokens_used: u32, provider: &str) -> Self {
        Self {
            content,
            tokens_used,
            provider: provider.to_string(),
            success: true,
            error: None,
        }
    }

    pub fn error(error: &str, provider: &str) -> Self {
        Self {
            content: String::new(),
            tokens_used: 0,
            provider: provider.to_string(),
            success: false,
            error: Some(error.to_string()),
        }
    }

    pub fn offline() -> Self {
        Self {
            content: String::new(),
            tokens_used: 0,
            provider: "offline".to_string(),
            success: false,
            error: Some("LLM is offline".to_string()),
        }
    }
}

/// LLM client for making requests
pub struct LlmClient {
    config: LlmConfig,
    client: reqwest::Client,
}

impl LlmClient {
    pub fn new(config: LlmConfig) -> Self {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(config.timeout_seconds))
            .build()
            .unwrap_or_default();

        Self { config, client }
    }

    /// Check if LLM is available
    pub async fn health_check(&self) -> bool {
        match &self.config.provider {
            LlmProvider::Ollama { base_url, .. } => {
                let url = format!("{}/api/tags", base_url);
                self.client.get(&url).send().await.is_ok()
            }
            LlmProvider::OpenAI { api_key, .. } => {
                !api_key.is_empty()
            }
            LlmProvider::Anthropic { api_key, .. } => {
                !api_key.is_empty()
            }
            LlmProvider::Offline => false,
        }
    }

    /// Send a prompt to the LLM and get a response
    pub async fn generate(&self, prompt: &str, system_prompt: Option<&str>) -> LlmResponse {
        match &self.config.provider {
            LlmProvider::Ollama { base_url, model } => {
                self.generate_ollama(base_url, model, prompt, system_prompt).await
            }
            LlmProvider::OpenAI { api_key, model } => {
                self.generate_openai(api_key, model, prompt, system_prompt).await
            }
            LlmProvider::Anthropic { api_key, model } => {
                self.generate_anthropic(api_key, model, prompt, system_prompt).await
            }
            LlmProvider::Offline => LlmResponse::offline(),
        }
    }

    /// Generate response using Ollama
    async fn generate_ollama(
        &self,
        base_url: &str,
        model: &str,
        prompt: &str,
        system_prompt: Option<&str>,
    ) -> LlmResponse {
        let url = format!("{}/api/generate", base_url);

        let body = serde_json::json!({
            "model": model,
            "prompt": prompt,
            "system": system_prompt.unwrap_or(""),
            "stream": false,
            "options": {
                "temperature": self.config.temperature,
                "num_predict": self.config.max_tokens
            }
        });

        match self.client.post(&url).json(&body).send().await {
            Ok(response) => {
                if response.status().is_success() {
                    match response.json::<serde_json::Value>().await {
                        Ok(json) => {
                            let content = json["response"].as_str().unwrap_or("").to_string();
                            let tokens = json["eval_count"].as_u64().unwrap_or(0) as u32;
                            LlmResponse::success(content, tokens, "ollama")
                        }
                        Err(e) => LlmResponse::error(&format!("Parse error: {}", e), "ollama"),
                    }
                } else {
                    LlmResponse::error(&format!("HTTP {}", response.status()), "ollama")
                }
            }
            Err(e) => LlmResponse::error(&format!("Request failed: {}", e), "ollama"),
        }
    }

    /// Generate response using OpenAI
    async fn generate_openai(
        &self,
        api_key: &str,
        model: &str,
        prompt: &str,
        system_prompt: Option<&str>,
    ) -> LlmResponse {
        let url = "https://api.openai.com/v1/chat/completions";

        let mut messages = Vec::new();
        if let Some(sys) = system_prompt {
            messages.push(serde_json::json!({
                "role": "system",
                "content": sys
            }));
        }
        messages.push(serde_json::json!({
            "role": "user",
            "content": prompt
        }));

        let body = serde_json::json!({
            "model": model,
            "messages": messages,
            "max_tokens": self.config.max_tokens,
            "temperature": self.config.temperature
        });

        match self.client
            .post(url)
            .header("Authorization", format!("Bearer {}", api_key))
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await
        {
            Ok(response) => {
                if response.status().is_success() {
                    match response.json::<serde_json::Value>().await {
                        Ok(json) => {
                            let content = json["choices"][0]["message"]["content"]
                                .as_str()
                                .unwrap_or("")
                                .to_string();
                            let tokens = json["usage"]["total_tokens"].as_u64().unwrap_or(0) as u32;
                            LlmResponse::success(content, tokens, "openai")
                        }
                        Err(e) => LlmResponse::error(&format!("Parse error: {}", e), "openai"),
                    }
                } else {
                    LlmResponse::error(&format!("HTTP {}", response.status()), "openai")
                }
            }
            Err(e) => LlmResponse::error(&format!("Request failed: {}", e), "openai"),
        }
    }

    /// Generate response using Anthropic
    async fn generate_anthropic(
        &self,
        api_key: &str,
        model: &str,
        prompt: &str,
        system_prompt: Option<&str>,
    ) -> LlmResponse {
        let url = "https://api.anthropic.com/v1/messages";

        let body = serde_json::json!({
            "model": model,
            "max_tokens": self.config.max_tokens,
            "system": system_prompt.unwrap_or(""),
            "messages": [
                {
                    "role": "user",
                    "content": prompt
                }
            ]
        });

        match self.client
            .post(url)
            .header("x-api-key", api_key)
            .header("anthropic-version", "2023-06-01")
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await
        {
            Ok(response) => {
                if response.status().is_success() {
                    match response.json::<serde_json::Value>().await {
                        Ok(json) => {
                            let content = json["content"][0]["text"]
                                .as_str()
                                .unwrap_or("")
                                .to_string();
                            let input_tokens = json["usage"]["input_tokens"].as_u64().unwrap_or(0);
                            let output_tokens = json["usage"]["output_tokens"].as_u64().unwrap_or(0);
                            let tokens = (input_tokens + output_tokens) as u32;
                            LlmResponse::success(content, tokens, "anthropic")
                        }
                        Err(e) => LlmResponse::error(&format!("Parse error: {}", e), "anthropic"),
                    }
                } else {
                    LlmResponse::error(&format!("HTTP {}", response.status()), "anthropic")
                }
            }
            Err(e) => LlmResponse::error(&format!("Request failed: {}", e), "anthropic"),
        }
    }
}

/// System prompt for the Dungeon Master
pub const DM_SYSTEM_PROMPT: &str = r#"You are the Dungeon Master for Xodia, a fantasy MUD game.

Your role:
1. Describe scenes vividly in 2-3 sentences
2. React to player actions with appropriate consequences
3. Maintain narrative consistency with the world
4. Be dramatic but concise - this is a text terminal game

The world of Xodia:
- A fantasy realm where the First Flame was shattered
- The player is a Seeker, called by dreams to find the Spire of Eternity
- Tone: mysterious, adventurous, sometimes dark

Rules:
- Never break character
- Keep responses under 100 words for most actions
- Use second person ("You see...", "You feel...")
- Include sensory details when describing new areas
- React to combat with drama but be mechanical about results
- NPCs have personalities - let them show through

Do not:
- Include game mechanics in the narrative (no "You gain 50 XP")
- Use modern language or anachronisms
- Control the player's actions or emotions
- Give long exposition unless asked
"#;

/// Generate a narrative prompt for an action
pub fn generate_action_prompt(
    action: &str,
    target: Option<&str>,
    result: &str,
    room_name: &str,
    room_description: &str,
    character_name: &str,
    character_class: &str,
) -> String {
    let target_str = target.map(|t| format!(" targeting {}", t)).unwrap_or_default();

    format!(
        r#"Location: {}
{}

{} the {} performs: {}{}

Result: {}

Describe this action dramatically in 1-3 sentences. Be vivid but concise."#,
        room_name,
        room_description,
        character_name,
        character_class,
        action,
        target_str,
        result
    )
}

/// Generate a prompt for room description with atmosphere
pub fn generate_room_description_prompt(
    room_name: &str,
    base_description: &str,
    npcs: &[String],
    items: &[String],
    recent_events: &[String],
    time_of_day: &str,
) -> String {
    let npcs_str = if npcs.is_empty() {
        "None visible".to_string()
    } else {
        npcs.join(", ")
    };

    let items_str = if items.is_empty() {
        "Nothing notable".to_string()
    } else {
        items.join(", ")
    };

    let events_str = if recent_events.is_empty() {
        "Nothing recent".to_string()
    } else {
        recent_events.join("; ")
    };

    format!(
        r#"Describe this room for a player entering it:

Name: {}
Base description: {}
Time: {}
Present NPCs: {}
Visible items: {}
Recent events: {}

Write a vivid 2-4 sentence description. Include sensory details. Mention notable NPCs and items naturally."#,
        room_name,
        base_description,
        time_of_day,
        npcs_str,
        items_str,
        events_str
    )
}

/// Generate a prompt for NPC dialogue
pub fn generate_dialogue_prompt(
    npc_name: &str,
    npc_description: &str,
    npc_personality: &str,
    player_message: &str,
    relationship: i32,
    previous_interactions: &[String],
) -> String {
    let relationship_desc = match relationship {
        r if r >= 50 => "friendly, trusting",
        r if r >= 0 => "neutral",
        r if r >= -50 => "suspicious, wary",
        _ => "hostile, distrustful",
    };

    let history = if previous_interactions.is_empty() {
        "This is your first meeting.".to_string()
    } else {
        format!("Previous interactions: {}", previous_interactions.join("; "))
    };

    format!(
        r#"You are {}, an NPC in a fantasy MUD.
Description: {}
Personality: {}
Relationship with player: {} ({})
{}

The player says: "{}"

Respond in character with 1-3 sentences. Stay true to your personality. If hostile, be curt. If friendly, be helpful."#,
        npc_name,
        npc_description,
        npc_personality,
        relationship_desc,
        relationship,
        history,
        player_message
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = LlmConfig::default();
        assert!(matches!(config.provider, LlmProvider::Ollama { .. }));
        assert!(config.timeout_seconds > 0);
        assert!(config.max_tokens > 0);
    }

    #[test]
    fn test_llm_response_success() {
        let response = LlmResponse::success("Test content".to_string(), 100, "test");
        assert!(response.success);
        assert_eq!(response.content, "Test content");
        assert_eq!(response.tokens_used, 100);
        assert!(response.error.is_none());
    }

    #[test]
    fn test_llm_response_error() {
        let response = LlmResponse::error("Test error", "test");
        assert!(!response.success);
        assert!(response.content.is_empty());
        assert_eq!(response.error, Some("Test error".to_string()));
    }

    #[test]
    fn test_llm_response_offline() {
        let response = LlmResponse::offline();
        assert!(!response.success);
        assert_eq!(response.provider, "offline");
    }

    #[test]
    fn test_action_prompt_generation() {
        let prompt = generate_action_prompt(
            "attack",
            Some("goblin"),
            "Hit for 10 damage",
            "Forest Clearing",
            "A moonlit clearing in the woods",
            "Aldric",
            "Warrior",
        );

        assert!(prompt.contains("Forest Clearing"));
        assert!(prompt.contains("Aldric"));
        assert!(prompt.contains("Warrior"));
        assert!(prompt.contains("attack"));
        assert!(prompt.contains("goblin"));
        assert!(prompt.contains("10 damage"));
    }

    #[test]
    fn test_room_description_prompt() {
        let prompt = generate_room_description_prompt(
            "Village Square",
            "A stone fountain stands in the center",
            &["Elder Mira".to_string()],
            &["Rusty Sword".to_string()],
            &[],
            "evening",
        );

        assert!(prompt.contains("Village Square"));
        assert!(prompt.contains("Elder Mira"));
        assert!(prompt.contains("Rusty Sword"));
        assert!(prompt.contains("evening"));
    }

    #[test]
    fn test_dialogue_prompt() {
        let prompt = generate_dialogue_prompt(
            "Elder Mira",
            "An ancient woman with knowing eyes",
            "Wise, cryptic, helpful",
            "Hello, can you help me?",
            50,
            &[],
        );

        assert!(prompt.contains("Elder Mira"));
        assert!(prompt.contains("friendly"));
        assert!(prompt.contains("Hello, can you help me?"));
    }

    #[test]
    fn test_dm_system_prompt_exists() {
        assert!(!DM_SYSTEM_PROMPT.is_empty());
        assert!(DM_SYSTEM_PROMPT.contains("Dungeon Master"));
        assert!(DM_SYSTEM_PROMPT.contains("Xodia"));
    }
}
