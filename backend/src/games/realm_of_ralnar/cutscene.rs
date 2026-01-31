//! Cutscene System for Realm of Ralnar
//!
//! Defines cutscenes, dialogue, and the player for story sequences.

use serde::{Deserialize, Serialize};

/// A complete cutscene with multiple scenes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Cutscene {
    /// Unique identifier
    pub id: String,
    /// List of scenes in order
    pub scenes: Vec<CutsceneScene>,
    /// Whether the player can skip this cutscene
    pub skippable: bool,
    /// Background music to play (if any)
    pub music: Option<String>,
}

impl Cutscene {
    /// Create a new cutscene
    pub fn new(id: &str) -> Self {
        Cutscene {
            id: id.to_string(),
            scenes: Vec::new(),
            skippable: true,
            music: None,
        }
    }

    /// Builder method to add a scene
    pub fn with_scene(mut self, scene: CutsceneScene) -> Self {
        self.scenes.push(scene);
        self
    }

    /// Builder method to make unskippable
    pub fn unskippable(mut self) -> Self {
        self.skippable = false;
        self
    }

    /// Builder method to set music
    pub fn with_music(mut self, music: &str) -> Self {
        self.music = Some(music.to_string());
        self
    }

    /// Get the total number of scenes
    pub fn scene_count(&self) -> usize {
        self.scenes.len()
    }

    /// Check if the cutscene is empty
    pub fn is_empty(&self) -> bool {
        self.scenes.is_empty()
    }
}

/// A single scene within a cutscene
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CutsceneScene {
    /// Optional background image/location
    pub background: Option<String>,
    /// Dialogue lines to display
    pub dialogue: Vec<CutsceneDialogue>,
    /// Actions to perform during this scene
    pub actions: Vec<CutsceneAction>,
    /// How long to display (ms) if no dialogue. None means wait for input
    pub duration_ms: Option<u32>,
}

impl CutsceneScene {
    /// Create an empty scene
    pub fn new() -> Self {
        CutsceneScene {
            background: None,
            dialogue: Vec::new(),
            actions: Vec::new(),
            duration_ms: None,
        }
    }

    /// Builder method to set background
    pub fn with_background(mut self, bg: &str) -> Self {
        self.background = Some(bg.to_string());
        self
    }

    /// Builder method to add dialogue
    pub fn with_dialogue(mut self, dialogue: CutsceneDialogue) -> Self {
        self.dialogue.push(dialogue);
        self
    }

    /// Builder method to add an action
    pub fn with_action(mut self, action: CutsceneAction) -> Self {
        self.actions.push(action);
        self
    }

    /// Builder method to set duration
    pub fn with_duration(mut self, ms: u32) -> Self {
        self.duration_ms = Some(ms);
        self
    }

    /// Check if this scene has dialogue
    pub fn has_dialogue(&self) -> bool {
        !self.dialogue.is_empty()
    }
}

impl Default for CutsceneScene {
    fn default() -> Self {
        Self::new()
    }
}

/// A single line of dialogue
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CutsceneDialogue {
    /// Who is speaking (None for narration)
    pub speaker: Option<String>,
    /// The text to display
    pub text: String,
    /// Emotional state/expression
    pub emotion: Option<String>,
}

impl CutsceneDialogue {
    /// Create a new dialogue line
    pub fn new(text: &str) -> Self {
        CutsceneDialogue {
            speaker: None,
            text: text.to_string(),
            emotion: None,
        }
    }

    /// Create dialogue with a speaker
    pub fn with_speaker(speaker: &str, text: &str) -> Self {
        CutsceneDialogue {
            speaker: Some(speaker.to_string()),
            text: text.to_string(),
            emotion: None,
        }
    }

    /// Builder method to set emotion
    pub fn with_emotion(mut self, emotion: &str) -> Self {
        self.emotion = Some(emotion.to_string());
        self
    }

    /// Check if this is narration (no speaker)
    pub fn is_narration(&self) -> bool {
        self.speaker.is_none()
    }
}

/// Actions that can occur during a cutscene
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum CutsceneAction {
    /// Fade the screen in over duration_ms
    FadeIn(u32),
    /// Fade the screen out over duration_ms
    FadeOut(u32),
    /// Shake the screen
    ShakeScreen { duration_ms: u32, intensity: f32 },
    /// Play a sound effect
    PlaySound(String),
    /// Change the background music
    PlayMusic(String),
    /// Stop background music
    StopMusic,
    /// Wait for a duration
    Wait(u32),
    /// Show an image overlay
    ShowImage(String),
    /// Hide an image overlay
    HideImage(String),
    /// Set a story flag
    SetFlag(String, bool),
    /// Flash the screen (for impacts, revelations)
    FlashScreen { color: String, duration_ms: u32 },
    /// Change the background
    ChangeBackground(String),
    /// Move a character sprite
    MoveCharacter {
        character: String,
        x: i32,
        y: i32,
        duration_ms: u32,
    },
    /// Show a character
    ShowCharacter { character: String, position: String },
    /// Hide a character
    HideCharacter(String),
}

/// Result of advancing a cutscene
#[derive(Debug, Clone, PartialEq)]
pub enum CutsceneResult {
    /// Continue - more content available
    Continue,
    /// Waiting for player input to advance
    WaitForInput,
    /// Cutscene is finished
    Finished,
    /// An action should be processed
    Action(CutsceneAction),
}

/// Player for cutscenes
#[derive(Debug, Clone)]
pub struct CutscenePlayer {
    /// The cutscene being played
    cutscene: Cutscene,
    /// Current scene index
    current_scene: usize,
    /// Current dialogue index within the scene
    current_dialogue: usize,
    /// Current action index within the scene
    current_action: usize,
    /// Whether we're waiting for player input
    waiting_for_input: bool,
    /// Whether actions for current scene have been processed
    actions_processed: bool,
}

impl CutscenePlayer {
    /// Create a new cutscene player
    pub fn new(cutscene: Cutscene) -> Self {
        CutscenePlayer {
            cutscene,
            current_scene: 0,
            current_dialogue: 0,
            current_action: 0,
            waiting_for_input: false,
            actions_processed: false,
        }
    }

    /// Get the current cutscene
    pub fn cutscene(&self) -> &Cutscene {
        &self.cutscene
    }

    /// Check if the cutscene is finished
    pub fn is_finished(&self) -> bool {
        self.current_scene >= self.cutscene.scenes.len()
    }

    /// Check if the cutscene is skippable
    pub fn is_skippable(&self) -> bool {
        self.cutscene.skippable
    }

    /// Get the current scene (if any)
    pub fn current_scene(&self) -> Option<&CutsceneScene> {
        self.cutscene.scenes.get(self.current_scene)
    }

    /// Get the current dialogue (if any)
    pub fn current_text(&self) -> Option<&CutsceneDialogue> {
        if let Some(scene) = self.current_scene() {
            scene.dialogue.get(self.current_dialogue)
        } else {
            None
        }
    }

    /// Get the current background (if any)
    pub fn current_background(&self) -> Option<&str> {
        self.current_scene()
            .and_then(|s| s.background.as_deref())
    }

    /// Advance the cutscene (returns what to do next)
    pub fn advance(&mut self) -> CutsceneResult {
        // If finished, stay finished
        if self.is_finished() {
            return CutsceneResult::Finished;
        }

        let scene = match self.cutscene.scenes.get(self.current_scene) {
            Some(s) => s,
            None => return CutsceneResult::Finished,
        };

        // First, process any pending actions
        if !self.actions_processed && self.current_action < scene.actions.len() {
            let action = scene.actions[self.current_action].clone();
            self.current_action += 1;

            // If we've processed all actions, mark it
            if self.current_action >= scene.actions.len() {
                self.actions_processed = true;
            }

            return CutsceneResult::Action(action);
        }

        // Mark actions as processed once we've gone through them all
        self.actions_processed = true;

        // Then, advance through dialogue
        if self.current_dialogue < scene.dialogue.len() {
            self.waiting_for_input = true;
            self.current_dialogue += 1;

            if self.current_dialogue < scene.dialogue.len() {
                return CutsceneResult::WaitForInput;
            }
        }

        // All dialogue done, move to next scene
        self.current_scene += 1;
        self.current_dialogue = 0;
        self.current_action = 0;
        self.actions_processed = false;
        self.waiting_for_input = false;

        if self.current_scene >= self.cutscene.scenes.len() {
            CutsceneResult::Finished
        } else {
            CutsceneResult::Continue
        }
    }

    /// Skip to the end of the cutscene
    pub fn skip(&mut self) {
        if self.cutscene.skippable {
            self.current_scene = self.cutscene.scenes.len();
        }
    }

    /// Get progress as (current, total)
    pub fn progress(&self) -> (usize, usize) {
        (self.current_scene + 1, self.cutscene.scenes.len())
    }

    /// Reset to the beginning
    pub fn reset(&mut self) {
        self.current_scene = 0;
        self.current_dialogue = 0;
        self.current_action = 0;
        self.waiting_for_input = false;
        self.actions_processed = false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cutscene_creation() {
        let cutscene = Cutscene::new("test_cutscene")
            .with_scene(CutsceneScene::new())
            .with_music("test_music")
            .unskippable();

        assert_eq!(cutscene.id, "test_cutscene");
        assert_eq!(cutscene.scene_count(), 1);
        assert!(!cutscene.skippable);
        assert_eq!(cutscene.music, Some("test_music".to_string()));
    }

    #[test]
    fn test_cutscene_scene_creation() {
        let scene = CutsceneScene::new()
            .with_background("castle_interior")
            .with_dialogue(CutsceneDialogue::with_speaker("Herbert", "Hello!"))
            .with_action(CutsceneAction::FadeIn(500))
            .with_duration(3000);

        assert_eq!(scene.background, Some("castle_interior".to_string()));
        assert_eq!(scene.dialogue.len(), 1);
        assert_eq!(scene.actions.len(), 1);
        assert_eq!(scene.duration_ms, Some(3000));
        assert!(scene.has_dialogue());
    }

    #[test]
    fn test_dialogue_creation() {
        let narration = CutsceneDialogue::new("The sun rose over the hills.");
        assert!(narration.is_narration());
        assert!(narration.speaker.is_none());

        let dialogue = CutsceneDialogue::with_speaker("Herbert", "Good morning!")
            .with_emotion("happy");
        assert!(!dialogue.is_narration());
        assert_eq!(dialogue.speaker, Some("Herbert".to_string()));
        assert_eq!(dialogue.emotion, Some("happy".to_string()));
    }

    #[test]
    fn test_cutscene_player_simple() {
        let cutscene = Cutscene::new("test")
            .with_scene(
                CutsceneScene::new()
                    .with_dialogue(CutsceneDialogue::with_speaker("A", "Line 1"))
                    .with_dialogue(CutsceneDialogue::with_speaker("B", "Line 2"))
            );

        let mut player = CutscenePlayer::new(cutscene);

        assert!(!player.is_finished());
        assert!(player.current_text().is_some());

        // Advance through first dialogue
        let result = player.advance();
        assert!(matches!(result, CutsceneResult::WaitForInput));

        // Advance through second dialogue
        let result = player.advance();
        assert!(matches!(result, CutsceneResult::Finished));

        assert!(player.is_finished());
    }

    #[test]
    fn test_cutscene_player_with_actions() {
        let cutscene = Cutscene::new("test")
            .with_scene(
                CutsceneScene::new()
                    .with_action(CutsceneAction::FadeIn(500))
                    .with_action(CutsceneAction::PlayMusic("theme".to_string()))
                    .with_dialogue(CutsceneDialogue::new("Text"))
            );

        let mut player = CutscenePlayer::new(cutscene);

        // First advance should return the FadeIn action
        let result = player.advance();
        assert!(matches!(result, CutsceneResult::Action(CutsceneAction::FadeIn(500))));

        // Second advance should return the PlayMusic action
        let result = player.advance();
        assert!(matches!(result, CutsceneResult::Action(CutsceneAction::PlayMusic(_))));

        // Third advance should be waiting for input (dialogue)
        let result = player.advance();
        assert!(matches!(result, CutsceneResult::Finished));
    }

    #[test]
    fn test_cutscene_player_multiple_scenes() {
        let cutscene = Cutscene::new("test")
            .with_scene(
                CutsceneScene::new()
                    .with_dialogue(CutsceneDialogue::new("Scene 1"))
            )
            .with_scene(
                CutsceneScene::new()
                    .with_dialogue(CutsceneDialogue::new("Scene 2"))
            );

        let mut player = CutscenePlayer::new(cutscene);

        // Advance through scene 1
        let result = player.advance();
        assert!(matches!(result, CutsceneResult::Continue));

        // Advance through scene 2
        let result = player.advance();
        assert!(matches!(result, CutsceneResult::Finished));
    }

    #[test]
    fn test_cutscene_skip() {
        let cutscene = Cutscene::new("skippable")
            .with_scene(CutsceneScene::new().with_dialogue(CutsceneDialogue::new("Line 1")))
            .with_scene(CutsceneScene::new().with_dialogue(CutsceneDialogue::new("Line 2")));

        let mut player = CutscenePlayer::new(cutscene);
        assert!(!player.is_finished());

        player.skip();
        assert!(player.is_finished());
    }

    #[test]
    fn test_cutscene_unskippable() {
        let cutscene = Cutscene::new("unskippable")
            .unskippable()
            .with_scene(CutsceneScene::new().with_dialogue(CutsceneDialogue::new("Line")));

        let mut player = CutscenePlayer::new(cutscene);
        assert!(!player.is_skippable());

        player.skip();
        assert!(!player.is_finished()); // Should NOT skip
    }

    #[test]
    fn test_cutscene_progress() {
        let cutscene = Cutscene::new("test")
            .with_scene(CutsceneScene::new().with_dialogue(CutsceneDialogue::new("1")))
            .with_scene(CutsceneScene::new().with_dialogue(CutsceneDialogue::new("2")))
            .with_scene(CutsceneScene::new().with_dialogue(CutsceneDialogue::new("3")));

        let mut player = CutscenePlayer::new(cutscene);

        assert_eq!(player.progress(), (1, 3));

        player.advance();
        assert_eq!(player.progress(), (2, 3));

        player.advance();
        assert_eq!(player.progress(), (3, 3));
    }

    #[test]
    fn test_cutscene_reset() {
        let cutscene = Cutscene::new("test")
            .with_scene(CutsceneScene::new().with_dialogue(CutsceneDialogue::new("Line")));

        let mut player = CutscenePlayer::new(cutscene);

        // Advance to the end
        player.advance();
        assert!(player.is_finished());

        // Reset
        player.reset();
        assert!(!player.is_finished());
        assert_eq!(player.progress(), (1, 1));
    }

    #[test]
    fn test_cutscene_current_background() {
        let cutscene = Cutscene::new("test")
            .with_scene(CutsceneScene::new().with_background("castle"));

        let player = CutscenePlayer::new(cutscene);
        assert_eq!(player.current_background(), Some("castle"));
    }

    #[test]
    fn test_empty_cutscene() {
        let cutscene = Cutscene::new("empty");
        assert!(cutscene.is_empty());

        let mut player = CutscenePlayer::new(cutscene);
        assert!(player.is_finished());
    }

    #[test]
    fn test_cutscene_actions() {
        // Test various action types can be created
        let actions = vec![
            CutsceneAction::FadeIn(500),
            CutsceneAction::FadeOut(500),
            CutsceneAction::ShakeScreen { duration_ms: 1000, intensity: 0.5 },
            CutsceneAction::PlaySound("boom".to_string()),
            CutsceneAction::PlayMusic("theme".to_string()),
            CutsceneAction::StopMusic,
            CutsceneAction::Wait(1000),
            CutsceneAction::ShowImage("logo".to_string()),
            CutsceneAction::HideImage("logo".to_string()),
            CutsceneAction::SetFlag("test".to_string(), true),
            CutsceneAction::FlashScreen { color: "white".to_string(), duration_ms: 100 },
            CutsceneAction::ChangeBackground("new_bg".to_string()),
            CutsceneAction::MoveCharacter {
                character: "herbert".to_string(),
                x: 100,
                y: 50,
                duration_ms: 500,
            },
            CutsceneAction::ShowCharacter {
                character: "sera".to_string(),
                position: "left".to_string(),
            },
            CutsceneAction::HideCharacter("sera".to_string()),
        ];

        // All actions should be comparable
        for (i, action) in actions.iter().enumerate() {
            assert_eq!(action, &actions[i]);
        }
    }

    #[test]
    fn test_scene_without_dialogue() {
        let scene = CutsceneScene::new()
            .with_action(CutsceneAction::FadeIn(500))
            .with_duration(2000);

        assert!(!scene.has_dialogue());
        assert!(scene.dialogue.is_empty());
    }

    #[test]
    fn test_cutscene_player_finished_state() {
        let cutscene = Cutscene::new("test")
            .with_scene(CutsceneScene::new().with_dialogue(CutsceneDialogue::new("Line")));

        let mut player = CutscenePlayer::new(cutscene);

        // Advance to completion
        while !player.is_finished() {
            player.advance();
        }

        // Further advances should still return Finished
        assert!(matches!(player.advance(), CutsceneResult::Finished));
        assert!(matches!(player.advance(), CutsceneResult::Finished));
    }
}
