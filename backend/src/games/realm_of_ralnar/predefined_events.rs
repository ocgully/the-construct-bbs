//! Predefined Story Events for Realm of Ralnar
//!
//! Contains all major story events, cutscenes, and encounter definitions.

use super::cutscene::{Cutscene, CutsceneAction, CutsceneDialogue, CutsceneScene};
use super::events::{EventCondition, EventEffect, GameEvent};
use super::story::{flags, WorldPhase};

// ============================================================================
// KEY CUTSCENES
// ============================================================================

/// The reveal cutscene - when Sera sees the Fire Guardian's true form
pub fn create_reveal_cutscene() -> Cutscene {
    Cutscene::new("the_reveal")
        .unskippable()
        .with_music("despair")
        .with_scene(
            CutsceneScene::new()
                .with_background("fire_shrine_interior")
                .with_action(CutsceneAction::PlayMusic("victory_fanfare".to_string()))
                .with_dialogue(CutsceneDialogue::with_speaker("Herbert", "It's done.")
                    .with_emotion("tired"))
                .with_dialogue(CutsceneDialogue::with_speaker("Valeran", "That was the toughest one yet."))
                .with_dialogue(CutsceneDialogue::with_speaker("Sera", "...")
                    .with_emotion("horrified"))
        )
        .with_scene(
            CutsceneScene::new()
                .with_background("fire_shrine_interior")
                .with_action(CutsceneAction::ShakeScreen { duration_ms: 2000, intensity: 0.5 })
                .with_action(CutsceneAction::StopMusic)
                .with_dialogue(CutsceneDialogue::with_speaker("Herbert", "The way out!"))
                .with_dialogue(CutsceneDialogue::with_speaker("Valeran", "We're trapped. We'll have to dig."))
                .with_dialogue(CutsceneDialogue::with_speaker("Sera", "Something's wrong.")
                    .with_emotion("confused"))
        )
        .with_scene(
            CutsceneScene::new()
                .with_background("fire_shrine_interior")
                .with_action(CutsceneAction::FlashScreen { color: "white".to_string(), duration_ms: 500 })
                .with_dialogue(CutsceneDialogue::with_speaker("Valeran", "My head feels... clearer. Like waking up."))
                .with_dialogue(CutsceneDialogue::with_speaker("Herbert", "Mine too."))
                .with_dialogue(CutsceneDialogue::with_speaker("Sera", "Come look at this. Now."))
        )
        .with_scene(
            CutsceneScene::new()
                .with_background("fire_guardian_dying")
                .with_action(CutsceneAction::PlayMusic("despair".to_string()))
                .with_dialogue(CutsceneDialogue::with_speaker("Sera", "No...")
                    .with_emotion("horrified"))
                .with_dialogue(CutsceneDialogue::with_speaker("Herbert", "What happened to it?"))
                .with_dialogue(CutsceneDialogue::with_speaker("Sera",
                    "That's not a monster. That's... that's Pyreth. The Fire Guardian."))
        )
        .with_scene(
            CutsceneScene::new()
                .with_background("fire_guardian_dying")
                .with_dialogue(CutsceneDialogue::with_speaker("Valeran", "That's impossible. We saw-"))
                .with_dialogue(CutsceneDialogue::with_speaker("Sera",
                    "YOU saw a monster. I saw a Guardian defending his shrine. From us."))
                .with_dialogue(CutsceneDialogue::with_speaker("Herbert",
                    "The blessing. Before every shrine, Dorl cast..."))
                .with_dialogue(CutsceneDialogue::with_speaker("Sera",
                    "He never blessed me. I refused. And I was never at the shrines..."))
                .with_dialogue(CutsceneDialogue::with_speaker("Valeran",
                    "Because something always came up. Something always kept you away."))
        )
        .with_scene(
            CutsceneScene::new()
                .with_background("fire_guardian_dying")
                .with_action(CutsceneAction::Wait(2000))
                .with_dialogue(CutsceneDialogue::with_speaker("Herbert",
                    "Dorl. It was Dorl. The whole time."))
        )
        .with_scene(
            CutsceneScene::new()
                .with_background("sky_tears_open")
                .with_action(CutsceneAction::ShakeScreen { duration_ms: 3000, intensity: 0.8 })
                .with_dialogue(CutsceneDialogue::with_speaker("Pyreth", "Find... the echoes... seal... the Rift...")
                    .with_emotion("dying"))
                .with_dialogue(CutsceneDialogue::new("The Guardian's flame extinguishes."))
        )
        .with_scene(
            CutsceneScene::new()
                .with_background("fire_shrine_interior")
                .with_dialogue(CutsceneDialogue::with_speaker("Sera",
                    "I swore to protect them. My whole life. And I wasn't there. Not once.")
                    .with_emotion("crying"))
                .with_dialogue(CutsceneDialogue::with_speaker("Herbert",
                    "He made sure of that. This isn't your fault."))
                .with_dialogue(CutsceneDialogue::with_speaker("Sera",
                    "It doesn't matter whose fault it is. They're dead. All five of them. Because of us."))
        )
        .with_scene(
            CutsceneScene::new()
                .with_background("fire_shrine_interior")
                .with_dialogue(CutsceneDialogue::with_speaker("Valeran", "What do we do now?"))
                .with_dialogue(CutsceneDialogue::with_speaker("Sera",
                    "The Guardian said 'find the echoes.' When Guardians die, fragments of their power linger. We might be able to..."))
                .with_dialogue(CutsceneDialogue::with_speaker("Herbert", "To what?"))
                .with_dialogue(CutsceneDialogue::with_speaker("Sera",
                    "I don't know. But it's all we have."))
                .with_dialogue(CutsceneDialogue::with_speaker("Herbert",
                    "Then we dig out. And we start fixing this."))
        )
        .with_scene(
            CutsceneScene::new()
                .with_action(CutsceneAction::FadeOut(2000))
                .with_action(CutsceneAction::SetFlag(flags::THE_REVEAL.to_string(), true))
                .with_dialogue(CutsceneDialogue::new("THE RIFT OPENS"))
        )
}

/// Dorl's introduction cutscene
pub fn create_dorl_intro_cutscene() -> Cutscene {
    Cutscene::new("dorl_intro")
        .with_music("mysterious")
        .with_scene(
            CutsceneScene::new()
                .with_background("thornwick_square")
                .with_dialogue(CutsceneDialogue::new(
                    "An old man approaches - DORL, in his first appearance."))
                .with_dialogue(CutsceneDialogue::with_speaker("Dorl",
                    "Forgive an old man's intrusion. I couldn't help but notice your skill in battle."))
                .with_dialogue(CutsceneDialogue::with_speaker("Herbert", "Who are you?"))
                .with_dialogue(CutsceneDialogue::with_speaker("Dorl",
                    "A traveler. A scholar. Someone who has seen these dark times building for years... and believes he knows how to stop them."))
        )
        .with_scene(
            CutsceneScene::new()
                .with_background("thornwick_square")
                .with_dialogue(CutsceneDialogue::with_speaker("Valeran", "How?"))
                .with_dialogue(CutsceneDialogue::with_speaker("Dorl",
                    "The shrines. The ancient guardians who protected this land have fallen silent. Or worse... been corrupted. Something has twisted them into sources of darkness rather than light."))
                .with_dialogue(CutsceneDialogue::with_speaker("Herbert", "And you think we can fix that?"))
                .with_dialogue(CutsceneDialogue::with_speaker("Dorl",
                    "I think you have potential. And I think... I might know what happened to your parents.")
                    .with_emotion("warm"))
        )
        .with_scene(
            CutsceneScene::new()
                .with_background("thornwick_square")
                .with_dialogue(CutsceneDialogue::new("The brothers freeze."))
                .with_dialogue(CutsceneDialogue::with_speaker("Herbert", "What did you say?"))
                .with_dialogue(CutsceneDialogue::with_speaker("Dorl",
                    "The Dimming. Ten years ago. When the sky went dark and people vanished. Your parents among them, yes?"))
                .with_dialogue(CutsceneDialogue::with_speaker("Valeran", "How do you know about that?"))
                .with_dialogue(CutsceneDialogue::with_speaker("Dorl",
                    "Because I've been searching for answers too. And I believe they're connected. The shrines. The monsters. The Dimming. Help me investigate the first shrine, and I'll share what I know."))
        )
        .with_scene(
            CutsceneScene::new()
                .with_background("thornwick_square")
                .with_dialogue(CutsceneDialogue::with_speaker("Herbert",
                    "We were just going to get supplies and go home."))
                .with_dialogue(CutsceneDialogue::with_speaker("Valeran",
                    "Herbert... it's mom and dad."))
                .with_dialogue(CutsceneDialogue::with_speaker("Herbert",
                    "...One shrine. We check one shrine. Then we decide."))
                .with_dialogue(CutsceneDialogue::with_speaker("Dorl",
                    "That's all I ask. Come, let me give you something first. The shrines are dangerous places. A small protection...")
                    .with_emotion("warm"))
        )
        .with_scene(
            CutsceneScene::new()
                .with_background("dorl_blessing")
                .with_action(CutsceneAction::FlashScreen { color: "gold".to_string(), duration_ms: 1000 })
                .with_dialogue(CutsceneDialogue::new("Dorl's hands glow with golden light - THE BLESSING"))
                .with_dialogue(CutsceneDialogue::with_speaker("Dorl",
                    "There. This will shield your minds from any corruption you might encounter."))
                .with_dialogue(CutsceneDialogue::with_speaker("Valeran",
                    "It feels... warm. Safe."))
                .with_dialogue(CutsceneDialogue::with_speaker("Dorl",
                    "As it should. Now, shall we begin?"))
        )
        .with_scene(
            CutsceneScene::new()
                .with_action(CutsceneAction::FadeOut(1000))
                .with_action(CutsceneAction::SetFlag(flags::MET_DORL.to_string(), true))
                .with_action(CutsceneAction::SetFlag(flags::RECEIVED_BLESSING.to_string(), true))
                .with_dialogue(CutsceneDialogue::new("They set out together..."))
        )
}

/// Sera's introduction cutscene
pub fn create_sera_intro_cutscene() -> Cutscene {
    Cutscene::new("sera_intro")
        .with_music("gentle_sorrow")
        .with_scene(
            CutsceneScene::new()
                .with_background("port_valdris_refugee_camp")
                .with_dialogue(CutsceneDialogue::new(
                    "Among the refugees, a young woman tends to the sick and wounded."))
                .with_dialogue(CutsceneDialogue::with_speaker("Sera",
                    "There, there. The fever will break soon. Rest now."))
                .with_dialogue(CutsceneDialogue::with_speaker("Valeran",
                    "You're a healer?"))
                .with_dialogue(CutsceneDialogue::with_speaker("Sera",
                    "A priestess. Of the old faith - the Five Guardians. Though lately, my prayers feel... unanswered."))
        )
        .with_scene(
            CutsceneScene::new()
                .with_background("port_valdris_refugee_camp")
                .with_dialogue(CutsceneDialogue::with_speaker("Herbert",
                    "The Five Guardians? We're investigating the shrines."))
                .with_dialogue(CutsceneDialogue::with_speaker("Sera",
                    "The shrines?")
                    .with_emotion("surprised"))
                .with_dialogue(CutsceneDialogue::with_speaker("Sera",
                    "My temple... it fell when the monsters came. I alone survived. I've been searching for answers, trying to understand why the Guardians seem silent."))
                .with_dialogue(CutsceneDialogue::with_speaker("Herbert",
                    "We cleansed one shrine already. Dorl - our guide - says there are more."))
        )
        .with_scene(
            CutsceneScene::new()
                .with_background("port_valdris_refugee_camp")
                .with_dialogue(CutsceneDialogue::with_speaker("Sera",
                    "May I... may I join you? I need to see the shrines for myself. To understand what's happened."))
                .with_dialogue(CutsceneDialogue::with_speaker("Valeran",
                    "We could use a healer!"))
                .with_dialogue(CutsceneDialogue::with_speaker("Herbert",
                    "Welcome aboard, priestess."))
                .with_dialogue(CutsceneDialogue::with_speaker("Sera",
                    "Sera. Just Sera.")
                    .with_emotion("grateful"))
        )
        .with_scene(
            CutsceneScene::new()
                .with_action(CutsceneAction::SetFlag(flags::SERA_JOINED.to_string(), true))
                .with_dialogue(CutsceneDialogue::new("Sera has joined the party!"))
        )
}

/// Zanth's return cutscene (post-reveal)
pub fn create_zanth_return_cutscene() -> Cutscene {
    Cutscene::new("zanth_return")
        .with_music("hope_returns")
        .with_scene(
            CutsceneScene::new()
                .with_background("ruined_waystation")
                .with_dialogue(CutsceneDialogue::new(
                    "A familiar figure appears on the road ahead."))
                .with_dialogue(CutsceneDialogue::with_speaker("Zanth",
                    "I was two hundred miles away when the sky went dark and every shrine I've ever blessed went cold at once."))
                .with_dialogue(CutsceneDialogue::with_speaker("Zanth",
                    "I came as fast as I could."))
        )
        .with_scene(
            CutsceneScene::new()
                .with_background("ruined_waystation")
                .with_dialogue(CutsceneDialogue::with_speaker("Zanth",
                    "Tell me what happened... no. Tell me later."))
                .with_dialogue(CutsceneDialogue::with_speaker("Zanth",
                    "First, let me hold you.")
                    .with_emotion("motherly"))
                .with_dialogue(CutsceneDialogue::new(
                    "She embraces Herbert, then Valeran, her eyes wet with tears."))
        )
        .with_scene(
            CutsceneScene::new()
                .with_background("ruined_waystation")
                .with_dialogue(CutsceneDialogue::with_speaker("Herbert",
                    "We... we made a terrible mistake, Zanth."))
                .with_dialogue(CutsceneDialogue::with_speaker("Zanth",
                    "Cry, child. Cry. The spirits hear our tears. They know you didn't mean it.")
                    .with_emotion("gentle"))
                .with_dialogue(CutsceneDialogue::with_speaker("Zanth",
                    "I've been a mother, a grandmother, a wanderer, and a fool. But I've never been a murderer... until now. We all carry this weight together."))
        )
        .with_scene(
            CutsceneScene::new()
                .with_action(CutsceneAction::SetFlag(flags::ZANTH_RETURNED.to_string(), true))
                .with_dialogue(CutsceneDialogue::new("Zanth has rejoined the party!"))
        )
}

// ============================================================================
// KEY STORY EVENTS
// ============================================================================

/// Dorl's first meeting event
pub fn create_dorl_meeting_event() -> GameEvent {
    GameEvent::new("dorl_first_meeting", "Meeting the Old Man")
        .with_condition(EventCondition::NotFlag(flags::MET_DORL.to_string()))
        .with_condition(EventCondition::AtLocation {
            map: "thornwick".to_string(),
            x: 15,
            y: 10,
        })
        .with_effect(EventEffect::SetFlag(flags::MET_DORL.to_string(), true))
        .with_effect(EventEffect::PlayCutscene("dorl_intro".to_string()))
        .with_effect(EventEffect::ActivateBlessing)
        .with_cutscene("dorl_intro")
        .with_priority(100)
}

/// Sera joins the party event
pub fn create_sera_joins_event() -> GameEvent {
    GameEvent::new("sera_joins", "The Wayward Cleric")
        .with_condition(EventCondition::Flag(flags::SHRINE_1_COMPLETE.to_string()))
        .with_condition(EventCondition::NotFlag(flags::SERA_JOINED.to_string()))
        .with_condition(EventCondition::AtLocation {
            map: "port_valdris".to_string(),
            x: 20,
            y: 8,
        })
        .with_effect(EventEffect::PlayCutscene("sera_intro".to_string()))
        .with_effect(EventEffect::JoinParty("sera".to_string()))
        .with_effect(EventEffect::SetFlag(flags::SERA_JOINED.to_string(), true))
        .with_cutscene("sera_intro")
        .with_priority(90)
}

/// Zanth joins event
pub fn create_zanth_joins_event() -> GameEvent {
    GameEvent::new("zanth_joins", "The Wandering Mystic")
        .with_condition(EventCondition::Flag(flags::SHRINE_1_COMPLETE.to_string()))
        .with_condition(EventCondition::NotFlag(flags::ZANTH_JOINED.to_string()))
        .with_condition(EventCondition::OnMap("roadside_shrine".to_string()))
        .with_effect(EventEffect::JoinParty("zanth".to_string()))
        .with_effect(EventEffect::SetFlag(flags::ZANTH_JOINED.to_string(), true))
        .with_effect(EventEffect::ShowMessage("Zanth has joined the party!".to_string()))
        .with_priority(85)
}

/// Zanth leaves event
pub fn create_zanth_leaves_event() -> GameEvent {
    GameEvent::new("zanth_leaves", "A Healer's Calling")
        .with_condition(EventCondition::Flag(flags::ZANTH_JOINED.to_string()))
        .with_condition(EventCondition::NotFlag(flags::ZANTH_LEFT.to_string()))
        .with_condition(EventCondition::Flag(flags::SHRINE_3_COMPLETE.to_string()))
        .with_condition(EventCondition::OnMap("briarwood_path".to_string()))
        .with_effect(EventEffect::LeaveParty("zanth".to_string()))
        .with_effect(EventEffect::SetFlag(flags::ZANTH_LEFT.to_string(), true))
        .with_effect(EventEffect::ShowMessage(
            "Zanth: 'There's a village to the south - Briarwood. Children dying. I... I have to go.'".to_string()))
        .with_priority(80)
}

/// Zanth returns post-reveal
pub fn create_zanth_returns_event() -> GameEvent {
    GameEvent::new("zanth_returns", "A Friend in Dark Times")
        .with_condition(EventCondition::Flag(flags::THE_REVEAL.to_string()))
        .with_condition(EventCondition::Flag(flags::ZANTH_LEFT.to_string()))
        .with_condition(EventCondition::NotFlag(flags::ZANTH_RETURNED.to_string()))
        .with_effect(EventEffect::PlayCutscene("zanth_return".to_string()))
        .with_effect(EventEffect::JoinParty("zanth".to_string()))
        .with_effect(EventEffect::SetFlag(flags::ZANTH_RETURNED.to_string(), true))
        .with_cutscene("zanth_return")
        .with_priority(95)
}

/// The reveal event - triggered after defeating the fifth guardian
pub fn create_reveal_event() -> GameEvent {
    GameEvent::new("the_reveal", "The Truth")
        .with_condition(EventCondition::ShrineDestroyed(4))
        .with_condition(EventCondition::NotFlag(flags::THE_REVEAL.to_string()))
        .with_condition(EventCondition::PartyHas("sera".to_string()))
        .with_effect(EventEffect::PlayCutscene("the_reveal".to_string()))
        .with_effect(EventEffect::TriggerReveal)
        .with_effect(EventEffect::DeactivateBlessing)
        .with_cutscene("the_reveal")
        .with_priority(100)
}

/// Create all shrine events (1-5)
pub fn create_shrine_events() -> Vec<GameEvent> {
    let shrine_names = ["Earth", "Water", "Light", "Wind", "Fire"];
    let shrine_maps = [
        "earth_shrine",
        "water_shrine",
        "light_shrine",
        "wind_shrine",
        "fire_shrine",
    ];
    let guardian_names = ["Terreth", "Aqualis", "Luminos", "Ventus", "Pyreth"];

    let mut events = Vec::new();

    for i in 0..5 {
        // Pre-shrine blessing renewal
        if i > 0 {
            let blessing_event = GameEvent::new(
                &format!("blessing_renewal_{}", i + 1),
                &format!("Dorl's Blessing Before {} Shrine", shrine_names[i]),
            )
            .with_condition(EventCondition::ShrineDestroyed(i - 1))
            .with_condition(EventCondition::ShrineNotDestroyed(i))
            .with_condition(EventCondition::NotFlag(format!("blessing_renewed_{}", i + 1)))
            .with_condition(EventCondition::OnMap(format!("{}_entrance", shrine_maps[i])))
            .with_effect(EventEffect::SetFlag(format!("blessing_renewed_{}", i + 1), true))
            .with_effect(EventEffect::ActivateBlessing)
            .with_effect(EventEffect::ShowMessage(
                "Dorl: 'Let me renew my blessing before you enter. The corruption here is strong.'".to_string()))
            .with_priority(90);

            events.push(blessing_event);
        }

        // Shrine boss encounter
        let boss_event = GameEvent::new(
            &format!("shrine_{}_boss", i + 1),
            &format!("{} Guardian Battle", shrine_names[i]),
        )
        .with_condition(EventCondition::OnMap(shrine_maps[i].to_string()))
        .with_condition(EventCondition::AtLocation {
            map: shrine_maps[i].to_string(),
            x: 10, // Shrine center
            y: 10,
        })
        .with_condition(EventCondition::ShrineNotDestroyed(i))
        .with_effect(EventEffect::StartBattle {
            enemies: vec![format!("{}_guardian", guardian_names[i].to_lowercase())],
            boss: true,
            guardian: Some(i),
        })
        .with_priority(100);

        events.push(boss_event);

        // Post-boss shrine destruction
        let shrine_complete = GameEvent::new(
            &format!("shrine_{}_complete", i + 1),
            &format!("{} Shrine Cleansed", shrine_names[i]),
        )
        .with_condition(EventCondition::Flag(format!("guardian_{}_defeated", i + 1)))
        .with_condition(EventCondition::ShrineNotDestroyed(i))
        .with_effect(EventEffect::DestroyShrine(i))
        .with_effect(EventEffect::ShowMessage(
            format!("The {} Shrine falls silent. {} is no more.",
                shrine_names[i], guardian_names[i])))
        .with_priority(95);

        events.push(shrine_complete);
    }

    events
}

/// Create post-reveal events for gathering echoes
pub fn create_echo_events() -> Vec<GameEvent> {
    let echo_names = ["Terreth", "Aqualis", "Luminos", "Ventus", "Pyreth"];
    let echo_maps = [
        "earth_shrine_ruins",
        "water_shrine_ruins",
        "light_shrine_ruins",
        "wind_shrine_ruins",
        "fire_shrine_ruins",
    ];

    let mut events = Vec::new();

    for i in 0..5 {
        let echo_event = GameEvent::new(
            &format!("echo_{}_collection", i + 1),
            &format!("Echo of {}", echo_names[i]),
        )
        .with_condition(EventCondition::Flag(flags::THE_REVEAL.to_string()))
        .with_condition(EventCondition::WorldPhase(WorldPhase::Redemption))
        .with_condition(EventCondition::OnMap(echo_maps[i].to_string()))
        .with_condition(EventCondition::NotFlag(format!("echo_{}", echo_names[i].to_lowercase())))
        .with_effect(EventEffect::CollectEcho(i))
        .with_effect(EventEffect::ShowMessage(
            format!("A fragment of {}'s power flows into you. '...forgive...'", echo_names[i])))
        .with_priority(85);

        events.push(echo_event);
    }

    // All echoes collected event
    let all_echoes = GameEvent::new("all_echoes_collected", "The Power to Seal")
        .with_condition(EventCondition::Flag(flags::ECHO_TERRETH.to_string()))
        .with_condition(EventCondition::Flag(flags::ECHO_AQUALIS.to_string()))
        .with_condition(EventCondition::Flag(flags::ECHO_LUMINOS.to_string()))
        .with_condition(EventCondition::Flag(flags::ECHO_VENTUS.to_string()))
        .with_condition(EventCondition::Flag(flags::ECHO_PYRETH.to_string()))
        .with_condition(EventCondition::NotFlag(flags::ECHOES_COLLECTED.to_string()))
        .with_effect(EventEffect::SetFlag(flags::ECHOES_COLLECTED.to_string(), true))
        .with_effect(EventEffect::ShowMessage(
            "All five echoes resonate as one. You feel the power to seal the Rift.".to_string()))
        .with_priority(100);

    events.push(all_echoes);

    events
}

/// Create ending events
pub fn create_ending_events() -> Vec<GameEvent> {
    vec![
        // Final boss unlocked
        GameEvent::new("final_boss_unlocked", "Confronting Dorl")
            .with_condition(EventCondition::Flag(flags::ECHOES_COLLECTED.to_string()))
            .with_condition(EventCondition::OnMap("obsidian_spire".to_string()))
            .with_effect(EventEffect::ShowMessage(
                "With the Guardians' echoes, you can now face Dorl and seal him away forever.".to_string()))
            .with_priority(100),

        // Final boss defeated
        GameEvent::new("final_boss_defeated", "The Deceiver Falls")
            .with_condition(EventCondition::Flag("dorl_defeated".to_string()))
            .with_condition(EventCondition::NotFlag(flags::FINAL_BOSS_DEFEATED.to_string()))
            .with_effect(EventEffect::SetFlag(flags::FINAL_BOSS_DEFEATED.to_string(), true))
            .with_effect(EventEffect::PlayCutscene("ending".to_string()))
            .with_priority(100),
    ]
}

/// Get all predefined events
pub fn get_all_events() -> Vec<GameEvent> {
    let mut events = Vec::new();

    // Core story events
    events.push(create_dorl_meeting_event());
    events.push(create_sera_joins_event());
    events.push(create_zanth_joins_event());
    events.push(create_zanth_leaves_event());
    events.push(create_zanth_returns_event());
    events.push(create_reveal_event());

    // Shrine events
    events.extend(create_shrine_events());

    // Echo events
    events.extend(create_echo_events());

    // Ending events
    events.extend(create_ending_events());

    events
}

/// Get all predefined cutscenes
pub fn get_all_cutscenes() -> Vec<Cutscene> {
    vec![
        create_reveal_cutscene(),
        create_dorl_intro_cutscene(),
        create_sera_intro_cutscene(),
        create_zanth_return_cutscene(),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reveal_cutscene_structure() {
        let cutscene = create_reveal_cutscene();
        assert_eq!(cutscene.id, "the_reveal");
        assert!(!cutscene.skippable);
        assert!(!cutscene.scenes.is_empty());

        // Should have multiple scenes
        assert!(cutscene.scene_count() >= 5);
    }

    #[test]
    fn test_dorl_intro_cutscene() {
        let cutscene = create_dorl_intro_cutscene();
        assert_eq!(cutscene.id, "dorl_intro");
        assert!(!cutscene.scenes.is_empty());

        // Should mention Dorl setting flag
        let last_scene = cutscene.scenes.last().unwrap();
        assert!(last_scene.actions.iter().any(|a| {
            matches!(a, CutsceneAction::SetFlag(f, true) if f == flags::MET_DORL)
        }));
    }

    #[test]
    fn test_dorl_meeting_event() {
        let event = create_dorl_meeting_event();
        assert_eq!(event.id, "dorl_first_meeting");
        assert!(event.one_time);

        // Should require NOT having met Dorl
        assert!(event.trigger_conditions.iter().any(|c| {
            matches!(c, EventCondition::NotFlag(f) if f == flags::MET_DORL)
        }));

        // Should set met_dorl flag
        assert!(event.effects.iter().any(|e| {
            matches!(e, EventEffect::SetFlag(f, true) if f == flags::MET_DORL)
        }));
    }

    #[test]
    fn test_sera_joins_event() {
        let event = create_sera_joins_event();
        assert_eq!(event.id, "sera_joins");

        // Should require shrine 1 complete
        assert!(event.trigger_conditions.iter().any(|c| {
            matches!(c, EventCondition::Flag(f) if f == flags::SHRINE_1_COMPLETE)
        }));

        // Should add sera to party
        assert!(event.effects.iter().any(|e| {
            matches!(e, EventEffect::JoinParty(s) if s == "sera")
        }));
    }

    #[test]
    fn test_reveal_event_conditions() {
        let event = create_reveal_event();

        // Should require 5th shrine destroyed
        assert!(event.trigger_conditions.iter().any(|c| {
            matches!(c, EventCondition::ShrineDestroyed(4))
        }));

        // Should require Sera in party
        assert!(event.trigger_conditions.iter().any(|c| {
            matches!(c, EventCondition::PartyHas(s) if s == "sera")
        }));

        // Should trigger reveal
        assert!(event.effects.iter().any(|e| {
            matches!(e, EventEffect::TriggerReveal)
        }));
    }

    #[test]
    fn test_shrine_events_complete() {
        let events = create_shrine_events();

        // Should have events for all 5 shrines
        // At least: 4 blessing renewals (shrines 2-5), 5 boss encounters, 5 completions
        assert!(events.len() >= 14);

        // Check that all shrine indices are covered
        let mut has_shrine_complete = [false; 5];
        for event in &events {
            if event.id.starts_with("shrine_") && event.id.ends_with("_complete") {
                let idx = event.id.chars().nth(7).unwrap().to_digit(10).unwrap() as usize - 1;
                has_shrine_complete[idx] = true;
            }
        }
        assert!(has_shrine_complete.iter().all(|&b| b));
    }

    #[test]
    fn test_echo_events_complete() {
        let events = create_echo_events();

        // Should have 5 echo collection events + 1 all echoes event
        assert_eq!(events.len(), 6);

        // All echo events should require THE_REVEAL
        for event in &events[0..5] {
            assert!(event.trigger_conditions.iter().any(|c| {
                matches!(c, EventCondition::Flag(f) if f == flags::THE_REVEAL)
            }));
        }
    }

    #[test]
    fn test_get_all_events() {
        let events = get_all_events();
        assert!(!events.is_empty());

        // Should have unique IDs
        let ids: Vec<_> = events.iter().map(|e| &e.id).collect();
        let unique_ids: std::collections::HashSet<_> = ids.iter().collect();
        assert_eq!(ids.len(), unique_ids.len(), "Event IDs should be unique");
    }

    #[test]
    fn test_get_all_cutscenes() {
        let cutscenes = get_all_cutscenes();
        assert!(!cutscenes.is_empty());

        // Should have unique IDs
        let ids: Vec<_> = cutscenes.iter().map(|c| &c.id).collect();
        let unique_ids: std::collections::HashSet<_> = ids.iter().collect();
        assert_eq!(ids.len(), unique_ids.len(), "Cutscene IDs should be unique");
    }

    #[test]
    fn test_zanth_story_arc() {
        let joins = create_zanth_joins_event();
        let leaves = create_zanth_leaves_event();
        let returns = create_zanth_returns_event();

        // Joins sets ZANTH_JOINED
        assert!(joins.effects.iter().any(|e| {
            matches!(e, EventEffect::SetFlag(f, true) if f == flags::ZANTH_JOINED)
        }));

        // Leaves requires ZANTH_JOINED and sets ZANTH_LEFT
        assert!(leaves.trigger_conditions.iter().any(|c| {
            matches!(c, EventCondition::Flag(f) if f == flags::ZANTH_JOINED)
        }));
        assert!(leaves.effects.iter().any(|e| {
            matches!(e, EventEffect::SetFlag(f, true) if f == flags::ZANTH_LEFT)
        }));

        // Returns requires ZANTH_LEFT and THE_REVEAL
        assert!(returns.trigger_conditions.iter().any(|c| {
            matches!(c, EventCondition::Flag(f) if f == flags::ZANTH_LEFT)
        }));
        assert!(returns.trigger_conditions.iter().any(|c| {
            matches!(c, EventCondition::Flag(f) if f == flags::THE_REVEAL)
        }));
    }
}
