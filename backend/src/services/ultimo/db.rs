//! Ultimo database layer
//!
//! Manages player characters, world state, housing, economy, and multiplayer features.
//! Uses SQLite with WAL mode for concurrent access.

use sqlx::sqlite::{SqlitePool, SqlitePoolOptions};
use std::path::Path;

pub struct UltimoDb {
    pool: SqlitePool,
}

#[allow(dead_code)]
impl UltimoDb {
    pub async fn new(db_path: &Path) -> Result<Self, sqlx::Error> {
        // Create parent directory if needed
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent).ok();
        }

        let db_url = format!("sqlite:{}?mode=rwc", db_path.display());
        let pool = SqlitePoolOptions::new()
            .max_connections(10)
            .connect(&db_url)
            .await?;

        // Enable WAL mode for concurrency
        sqlx::query("PRAGMA journal_mode=WAL")
            .execute(&pool)
            .await?;

        let db = Self { pool };
        db.init_schema().await?;
        Ok(db)
    }

    async fn init_schema(&self) -> Result<(), sqlx::Error> {
        // ========================================================================
        // CORE CHARACTER DATA
        // ========================================================================

        // Main save table - stores serialized GameState
        sqlx::query(r#"
            CREATE TABLE IF NOT EXISTS saves (
                user_id INTEGER PRIMARY KEY,
                handle TEXT NOT NULL,
                char_name TEXT NOT NULL,
                level INTEGER NOT NULL DEFAULT 1,
                total_xp INTEGER NOT NULL DEFAULT 0,
                state_json TEXT NOT NULL,
                last_saved TEXT NOT NULL
            )
        "#).execute(&self.pool).await?;

        // Characters table for multiplayer visibility and PvP
        sqlx::query(r#"
            CREATE TABLE IF NOT EXISTS characters (
                user_id INTEGER PRIMARY KEY,
                handle TEXT NOT NULL,
                char_name TEXT NOT NULL,
                level INTEGER NOT NULL DEFAULT 1,
                total_xp INTEGER NOT NULL DEFAULT 0,
                hp_current INTEGER NOT NULL,
                hp_max INTEGER NOT NULL,
                mana_current INTEGER NOT NULL,
                mana_max INTEGER NOT NULL,
                stamina_current INTEGER NOT NULL,
                stamina_max INTEGER NOT NULL,
                strength INTEGER NOT NULL,
                dexterity INTEGER NOT NULL,
                intelligence INTEGER NOT NULL,
                gold_pocket INTEGER NOT NULL DEFAULT 0,
                gold_bank INTEGER NOT NULL DEFAULT 0,
                -- Position
                zone TEXT NOT NULL DEFAULT 'britain',
                pos_x INTEGER NOT NULL DEFAULT 30,
                pos_y INTEGER NOT NULL DEFAULT 20,
                -- Equipment (item keys)
                equipped_weapon TEXT,
                equipped_armor TEXT,
                equipped_shield TEXT,
                -- Stats
                kills INTEGER NOT NULL DEFAULT 0,
                deaths INTEGER NOT NULL DEFAULT 0,
                pvp_kills INTEGER NOT NULL DEFAULT 0,
                pvp_deaths INTEGER NOT NULL DEFAULT 0,
                -- Title
                title TEXT,
                guild_id INTEGER,
                -- Romance
                partner_user_id INTEGER,
                partner_name TEXT,
                -- Status
                is_dead INTEGER NOT NULL DEFAULT 0,
                is_online INTEGER NOT NULL DEFAULT 0,
                last_active TEXT NOT NULL,
                created_at TEXT NOT NULL
            )
        "#).execute(&self.pool).await?;

        // ========================================================================
        // SKILLS
        // ========================================================================

        // Character skills (separate table for efficient querying)
        sqlx::query(r#"
            CREATE TABLE IF NOT EXISTS character_skills (
                user_id INTEGER NOT NULL,
                skill_key TEXT NOT NULL,
                skill_level INTEGER NOT NULL DEFAULT 0,
                PRIMARY KEY (user_id, skill_key),
                FOREIGN KEY (user_id) REFERENCES characters(user_id) ON DELETE CASCADE
            )
        "#).execute(&self.pool).await?;

        // ========================================================================
        // HOUSING SYSTEM
        // ========================================================================

        // Player houses
        sqlx::query(r#"
            CREATE TABLE IF NOT EXISTS houses (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                owner_id INTEGER NOT NULL,
                owner_name TEXT NOT NULL,
                house_type TEXT NOT NULL,
                zone TEXT NOT NULL,
                x INTEGER NOT NULL,
                y INTEGER NOT NULL,
                name TEXT,
                is_public INTEGER NOT NULL DEFAULT 0,
                purchased_at TEXT NOT NULL,
                last_maintenance TEXT NOT NULL,
                FOREIGN KEY (owner_id) REFERENCES characters(user_id) ON DELETE CASCADE
            )
        "#).execute(&self.pool).await?;

        // House storage
        sqlx::query(r#"
            CREATE TABLE IF NOT EXISTS house_storage (
                house_id INTEGER NOT NULL,
                item_key TEXT NOT NULL,
                quantity INTEGER NOT NULL DEFAULT 1,
                PRIMARY KEY (house_id, item_key),
                FOREIGN KEY (house_id) REFERENCES houses(id) ON DELETE CASCADE
            )
        "#).execute(&self.pool).await?;

        // House friends/co-owners
        sqlx::query(r#"
            CREATE TABLE IF NOT EXISTS house_access (
                house_id INTEGER NOT NULL,
                user_id INTEGER NOT NULL,
                access_level TEXT NOT NULL, -- 'friend' or 'co_owner'
                PRIMARY KEY (house_id, user_id),
                FOREIGN KEY (house_id) REFERENCES houses(id) ON DELETE CASCADE
            )
        "#).execute(&self.pool).await?;

        // House decorations
        sqlx::query(r#"
            CREATE TABLE IF NOT EXISTS house_decorations (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                house_id INTEGER NOT NULL,
                item_key TEXT NOT NULL,
                x INTEGER NOT NULL,
                y INTEGER NOT NULL,
                FOREIGN KEY (house_id) REFERENCES houses(id) ON DELETE CASCADE
            )
        "#).execute(&self.pool).await?;

        // ========================================================================
        // ECONOMY / TRADING
        // ========================================================================

        // Player trade offers (market)
        sqlx::query(r#"
            CREATE TABLE IF NOT EXISTS trade_offers (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                seller_id INTEGER NOT NULL,
                seller_name TEXT NOT NULL,
                item_key TEXT NOT NULL,
                quantity INTEGER NOT NULL,
                price INTEGER NOT NULL,
                created_at TEXT NOT NULL,
                FOREIGN KEY (seller_id) REFERENCES characters(user_id) ON DELETE CASCADE
            )
        "#).execute(&self.pool).await?;

        // Transaction history
        sqlx::query(r#"
            CREATE TABLE IF NOT EXISTS transactions (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                buyer_id INTEGER NOT NULL,
                seller_id INTEGER NOT NULL,
                item_key TEXT NOT NULL,
                quantity INTEGER NOT NULL,
                price INTEGER NOT NULL,
                transaction_at TEXT NOT NULL
            )
        "#).execute(&self.pool).await?;

        // ========================================================================
        // ROMANCE SYSTEM
        // ========================================================================

        // Player relationships (same-sex supported)
        sqlx::query(r#"
            CREATE TABLE IF NOT EXISTS relationships (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                user1_id INTEGER NOT NULL,
                user2_id INTEGER NOT NULL,
                status TEXT NOT NULL, -- 'courting', 'engaged', 'married', 'divorced'
                started_at TEXT NOT NULL,
                engaged_at TEXT,
                married_at TEXT,
                divorced_at TEXT,
                UNIQUE(user1_id, user2_id)
            )
        "#).execute(&self.pool).await?;

        // ========================================================================
        // GUILDS
        // ========================================================================

        // Guilds
        sqlx::query(r#"
            CREATE TABLE IF NOT EXISTS guilds (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL UNIQUE,
                tag TEXT NOT NULL,
                leader_id INTEGER NOT NULL,
                gold_bank INTEGER NOT NULL DEFAULT 0,
                created_at TEXT NOT NULL
            )
        "#).execute(&self.pool).await?;

        // Guild members
        sqlx::query(r#"
            CREATE TABLE IF NOT EXISTS guild_members (
                guild_id INTEGER NOT NULL,
                user_id INTEGER NOT NULL,
                rank TEXT NOT NULL DEFAULT 'member', -- 'member', 'officer', 'leader'
                joined_at TEXT NOT NULL,
                PRIMARY KEY (guild_id, user_id),
                FOREIGN KEY (guild_id) REFERENCES guilds(id) ON DELETE CASCADE,
                FOREIGN KEY (user_id) REFERENCES characters(user_id) ON DELETE CASCADE
            )
        "#).execute(&self.pool).await?;

        // ========================================================================
        // QUESTS
        // ========================================================================

        // Quest completions (for tracking)
        sqlx::query(r#"
            CREATE TABLE IF NOT EXISTS quest_completions (
                user_id INTEGER NOT NULL,
                quest_key TEXT NOT NULL,
                completed_at TEXT NOT NULL,
                PRIMARY KEY (user_id, quest_key)
            )
        "#).execute(&self.pool).await?;

        // ========================================================================
        // LEADERBOARDS
        // ========================================================================

        // Skill leaderboard snapshots
        sqlx::query(r#"
            CREATE TABLE IF NOT EXISTS skill_rankings (
                user_id INTEGER NOT NULL,
                skill_key TEXT NOT NULL,
                skill_level INTEGER NOT NULL,
                recorded_at TEXT NOT NULL,
                PRIMARY KEY (user_id, skill_key)
            )
        "#).execute(&self.pool).await?;

        // Achievement tracking
        sqlx::query(r#"
            CREATE TABLE IF NOT EXISTS achievements (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                user_id INTEGER NOT NULL,
                achievement_key TEXT NOT NULL,
                achieved_at TEXT NOT NULL,
                UNIQUE(user_id, achievement_key)
            )
        "#).execute(&self.pool).await?;

        // ========================================================================
        // WORLD STATE
        // ========================================================================

        // World events (for persistent world state)
        sqlx::query(r#"
            CREATE TABLE IF NOT EXISTS world_events (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                event_type TEXT NOT NULL,
                event_data TEXT NOT NULL,
                zone TEXT,
                started_at TEXT NOT NULL,
                ended_at TEXT
            )
        "#).execute(&self.pool).await?;

        // Spawned monsters (persistent)
        sqlx::query(r#"
            CREATE TABLE IF NOT EXISTS spawned_monsters (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                template_key TEXT NOT NULL,
                zone TEXT NOT NULL,
                x INTEGER NOT NULL,
                y INTEGER NOT NULL,
                level INTEGER NOT NULL,
                hp_current INTEGER NOT NULL,
                hp_max INTEGER NOT NULL,
                spawned_at TEXT NOT NULL
            )
        "#).execute(&self.pool).await?;

        // ========================================================================
        // CHAT / MESSAGING
        // ========================================================================

        // Mail system
        sqlx::query(r#"
            CREATE TABLE IF NOT EXISTS mail (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                from_id INTEGER NOT NULL,
                from_name TEXT NOT NULL,
                to_id INTEGER NOT NULL,
                subject TEXT NOT NULL,
                body TEXT NOT NULL,
                gold_attached INTEGER NOT NULL DEFAULT 0,
                item_attached TEXT,
                item_quantity INTEGER,
                is_read INTEGER NOT NULL DEFAULT 0,
                sent_at TEXT NOT NULL
            )
        "#).execute(&self.pool).await?;

        // ========================================================================
        // INDEXES
        // ========================================================================

        sqlx::query(r#"
            CREATE INDEX IF NOT EXISTS idx_characters_zone
            ON characters(zone, is_online)
        "#).execute(&self.pool).await?;

        sqlx::query(r#"
            CREATE INDEX IF NOT EXISTS idx_characters_level
            ON characters(level DESC)
        "#).execute(&self.pool).await?;

        sqlx::query(r#"
            CREATE INDEX IF NOT EXISTS idx_trade_offers_item
            ON trade_offers(item_key, price)
        "#).execute(&self.pool).await?;

        sqlx::query(r#"
            CREATE INDEX IF NOT EXISTS idx_houses_zone
            ON houses(zone, x, y)
        "#).execute(&self.pool).await?;

        sqlx::query(r#"
            CREATE INDEX IF NOT EXISTS idx_skill_rankings
            ON skill_rankings(skill_key, skill_level DESC)
        "#).execute(&self.pool).await?;

        sqlx::query(r#"
            CREATE INDEX IF NOT EXISTS idx_mail_to
            ON mail(to_id, is_read)
        "#).execute(&self.pool).await?;

        Ok(())
    }

    // ========================================================================
    // SAVE/LOAD OPERATIONS
    // ========================================================================

    pub async fn save_game(&self, user_id: i64, handle: &str, char_name: &str, level: u32, total_xp: i64, state_json: &str) -> Result<(), sqlx::Error> {
        sqlx::query(
            "INSERT OR REPLACE INTO saves (user_id, handle, char_name, level, total_xp, state_json, last_saved)
             VALUES (?, ?, ?, ?, ?, ?, datetime('now', '-5 hours'))"
        )
        .bind(user_id)
        .bind(handle)
        .bind(char_name)
        .bind(level as i32)
        .bind(total_xp)
        .bind(state_json)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn load_game(&self, user_id: i64) -> Result<Option<String>, sqlx::Error> {
        let row: Option<(String,)> = sqlx::query_as(
            "SELECT state_json FROM saves WHERE user_id = ?"
        )
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await?;
        Ok(row.map(|r| r.0))
    }

    pub async fn delete_save(&self, user_id: i64) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM saves WHERE user_id = ?")
            .bind(user_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    #[allow(dead_code)]
    pub async fn has_save(&self, user_id: i64) -> Result<bool, sqlx::Error> {
        let row: Option<(i64,)> = sqlx::query_as(
            "SELECT 1 FROM saves WHERE user_id = ?"
        )
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await?;
        Ok(row.is_some())
    }

    // ========================================================================
    // CHARACTER SYNC (for multiplayer visibility)
    // ========================================================================

    #[allow(clippy::too_many_arguments)]
    pub async fn sync_character(
        &self,
        user_id: i64,
        handle: &str,
        char_name: &str,
        level: u32,
        total_xp: i64,
        hp_current: i32,
        hp_max: i32,
        mana_current: i32,
        mana_max: i32,
        stamina_current: i32,
        stamina_max: i32,
        strength: i32,
        dexterity: i32,
        intelligence: i32,
        gold_pocket: i64,
        gold_bank: i64,
        zone: &str,
        pos_x: i32,
        pos_y: i32,
        equipped_weapon: Option<&str>,
        equipped_armor: Option<&str>,
        equipped_shield: Option<&str>,
        kills: u32,
        deaths: u32,
        pvp_kills: u32,
        pvp_deaths: u32,
        title: Option<&str>,
        guild_id: Option<i64>,
        partner_user_id: Option<i64>,
        partner_name: Option<&str>,
        is_dead: bool,
    ) -> Result<(), sqlx::Error> {
        // Check if character exists
        let exists: Option<(i64,)> = sqlx::query_as(
            "SELECT 1 FROM characters WHERE user_id = ?"
        )
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await?;

        if exists.is_some() {
            sqlx::query(r#"
                UPDATE characters SET
                    handle = ?, char_name = ?, level = ?, total_xp = ?,
                    hp_current = ?, hp_max = ?, mana_current = ?, mana_max = ?,
                    stamina_current = ?, stamina_max = ?, strength = ?, dexterity = ?, intelligence = ?,
                    gold_pocket = ?, gold_bank = ?, zone = ?, pos_x = ?, pos_y = ?,
                    equipped_weapon = ?, equipped_armor = ?, equipped_shield = ?,
                    kills = ?, deaths = ?, pvp_kills = ?, pvp_deaths = ?,
                    title = ?, guild_id = ?, partner_user_id = ?, partner_name = ?,
                    is_dead = ?, is_online = 1, last_active = datetime('now', '-5 hours')
                WHERE user_id = ?
            "#)
            .bind(handle)
            .bind(char_name)
            .bind(level as i32)
            .bind(total_xp)
            .bind(hp_current)
            .bind(hp_max)
            .bind(mana_current)
            .bind(mana_max)
            .bind(stamina_current)
            .bind(stamina_max)
            .bind(strength)
            .bind(dexterity)
            .bind(intelligence)
            .bind(gold_pocket)
            .bind(gold_bank)
            .bind(zone)
            .bind(pos_x)
            .bind(pos_y)
            .bind(equipped_weapon)
            .bind(equipped_armor)
            .bind(equipped_shield)
            .bind(kills as i32)
            .bind(deaths as i32)
            .bind(pvp_kills as i32)
            .bind(pvp_deaths as i32)
            .bind(title)
            .bind(guild_id)
            .bind(partner_user_id)
            .bind(partner_name)
            .bind(is_dead as i32)
            .bind(user_id)
            .execute(&self.pool)
            .await?;
        } else {
            sqlx::query(r#"
                INSERT INTO characters (
                    user_id, handle, char_name, level, total_xp,
                    hp_current, hp_max, mana_current, mana_max,
                    stamina_current, stamina_max, strength, dexterity, intelligence,
                    gold_pocket, gold_bank, zone, pos_x, pos_y,
                    equipped_weapon, equipped_armor, equipped_shield,
                    kills, deaths, pvp_kills, pvp_deaths,
                    title, guild_id, partner_user_id, partner_name,
                    is_dead, is_online, last_active, created_at
                ) VALUES (
                    ?, ?, ?, ?, ?,
                    ?, ?, ?, ?,
                    ?, ?, ?, ?, ?,
                    ?, ?, ?, ?, ?,
                    ?, ?, ?,
                    ?, ?, ?, ?,
                    ?, ?, ?, ?,
                    ?, 1, datetime('now', '-5 hours'), datetime('now', '-5 hours')
                )
            "#)
            .bind(user_id)
            .bind(handle)
            .bind(char_name)
            .bind(level as i32)
            .bind(total_xp)
            .bind(hp_current)
            .bind(hp_max)
            .bind(mana_current)
            .bind(mana_max)
            .bind(stamina_current)
            .bind(stamina_max)
            .bind(strength)
            .bind(dexterity)
            .bind(intelligence)
            .bind(gold_pocket)
            .bind(gold_bank)
            .bind(zone)
            .bind(pos_x)
            .bind(pos_y)
            .bind(equipped_weapon)
            .bind(equipped_armor)
            .bind(equipped_shield)
            .bind(kills as i32)
            .bind(deaths as i32)
            .bind(pvp_kills as i32)
            .bind(pvp_deaths as i32)
            .bind(title)
            .bind(guild_id)
            .bind(partner_user_id)
            .bind(partner_name)
            .bind(is_dead as i32)
            .execute(&self.pool)
            .await?;
        }

        Ok(())
    }

    pub async fn set_offline(&self, user_id: i64) -> Result<(), sqlx::Error> {
        sqlx::query("UPDATE characters SET is_online = 0 WHERE user_id = ?")
            .bind(user_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    // ========================================================================
    // MULTIPLAYER VISIBILITY
    // ========================================================================

    /// Get players in a zone (for rendering nearby players)
    pub async fn get_players_in_zone(&self, zone: &str, exclude_user_id: i64) -> Result<Vec<VisiblePlayer>, sqlx::Error> {
        let rows: Vec<(String, i32, i32, i32, Option<String>)> = sqlx::query_as(
            r#"
            SELECT char_name, level, pos_x, pos_y,
                   (SELECT name FROM guilds WHERE id = characters.guild_id) as guild_name
            FROM characters
            WHERE zone = ? AND user_id != ? AND is_online = 1
            "#
        )
        .bind(zone)
        .bind(exclude_user_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|(name, level, x, y, guild)| {
            VisiblePlayer {
                name,
                level: level as u32,
                x,
                y,
                guild,
            }
        }).collect())
    }

    /// Get all online players
    pub async fn get_online_players(&self) -> Result<Vec<OnlinePlayer>, sqlx::Error> {
        let rows: Vec<(i64, String, String, i32, String)> = sqlx::query_as(
            r#"
            SELECT user_id, handle, char_name, level, zone
            FROM characters
            WHERE is_online = 1
            ORDER BY level DESC
            "#
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|(user_id, handle, char_name, level, zone)| {
            OnlinePlayer {
                user_id,
                handle,
                char_name,
                level: level as u32,
                zone,
            }
        }).collect())
    }

    // ========================================================================
    // LEADERBOARDS
    // ========================================================================

    pub async fn get_level_leaderboard(&self, limit: i64) -> Result<Vec<LeaderboardEntry>, sqlx::Error> {
        let rows: Vec<(i64, String, String, i32, i64, i32, i32)> = sqlx::query_as(
            r#"
            SELECT
                RANK() OVER (ORDER BY level DESC, total_xp DESC) as rank,
                handle,
                char_name,
                level,
                total_xp,
                kills,
                pvp_kills
            FROM characters
            ORDER BY level DESC, total_xp DESC
            LIMIT ?
            "#
        )
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|(rank, handle, char_name, level, xp, kills, pvp_kills)| {
            LeaderboardEntry {
                rank,
                handle,
                char_name,
                level: level as u32,
                total_xp: xp,
                kills: kills as u32,
                pvp_kills: pvp_kills as u32,
            }
        }).collect())
    }

    pub async fn get_pvp_leaderboard(&self, limit: i64) -> Result<Vec<LeaderboardEntry>, sqlx::Error> {
        let rows: Vec<(i64, String, String, i32, i64, i32, i32)> = sqlx::query_as(
            r#"
            SELECT
                RANK() OVER (ORDER BY pvp_kills DESC) as rank,
                handle,
                char_name,
                level,
                total_xp,
                kills,
                pvp_kills
            FROM characters
            WHERE pvp_kills > 0
            ORDER BY pvp_kills DESC
            LIMIT ?
            "#
        )
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|(rank, handle, char_name, level, xp, kills, pvp_kills)| {
            LeaderboardEntry {
                rank,
                handle,
                char_name,
                level: level as u32,
                total_xp: xp,
                kills: kills as u32,
                pvp_kills: pvp_kills as u32,
            }
        }).collect())
    }

    pub async fn get_skill_leaderboard(&self, skill_key: &str, limit: i64) -> Result<Vec<SkillLeaderboardEntry>, sqlx::Error> {
        let rows: Vec<(i64, String, String, i32)> = sqlx::query_as(
            r#"
            SELECT
                RANK() OVER (ORDER BY skill_level DESC) as rank,
                c.handle,
                c.char_name,
                cs.skill_level
            FROM character_skills cs
            JOIN characters c ON c.user_id = cs.user_id
            WHERE cs.skill_key = ?
            ORDER BY cs.skill_level DESC
            LIMIT ?
            "#
        )
        .bind(skill_key)
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|(rank, handle, char_name, level)| {
            SkillLeaderboardEntry {
                rank,
                handle,
                char_name,
                skill_level: level as u32,
            }
        }).collect())
    }

    // ========================================================================
    // HOUSING
    // ========================================================================

    #[allow(dead_code)]
    pub async fn create_house(
        &self,
        owner_id: i64,
        owner_name: &str,
        house_type: &str,
        zone: &str,
        x: i32,
        y: i32,
    ) -> Result<i64, sqlx::Error> {
        let result = sqlx::query(
            r#"
            INSERT INTO houses (owner_id, owner_name, house_type, zone, x, y, purchased_at, last_maintenance)
            VALUES (?, ?, ?, ?, ?, ?, datetime('now', '-5 hours'), datetime('now', '-5 hours'))
            "#
        )
        .bind(owner_id)
        .bind(owner_name)
        .bind(house_type)
        .bind(zone)
        .bind(x)
        .bind(y)
        .execute(&self.pool)
        .await?;

        Ok(result.last_insert_rowid())
    }

    #[allow(dead_code)]
    pub async fn get_houses_in_zone(&self, zone: &str) -> Result<Vec<HouseInfo>, sqlx::Error> {
        let rows: Vec<(i64, String, String, i32, i32, Option<String>, i32)> = sqlx::query_as(
            r#"
            SELECT id, owner_name, house_type, x, y, name, is_public
            FROM houses
            WHERE zone = ?
            "#
        )
        .bind(zone)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|(id, owner, house_type, x, y, name, is_public)| {
            HouseInfo {
                id,
                owner_name: owner,
                house_type,
                x,
                y,
                name,
                is_public: is_public != 0,
            }
        }).collect())
    }

    // ========================================================================
    // TRADING
    // ========================================================================

    #[allow(dead_code)]
    pub async fn create_trade_offer(
        &self,
        seller_id: i64,
        seller_name: &str,
        item_key: &str,
        quantity: u32,
        price: i64,
    ) -> Result<i64, sqlx::Error> {
        let result = sqlx::query(
            r#"
            INSERT INTO trade_offers (seller_id, seller_name, item_key, quantity, price, created_at)
            VALUES (?, ?, ?, ?, ?, datetime('now', '-5 hours'))
            "#
        )
        .bind(seller_id)
        .bind(seller_name)
        .bind(item_key)
        .bind(quantity as i32)
        .bind(price)
        .execute(&self.pool)
        .await?;

        Ok(result.last_insert_rowid())
    }

    #[allow(dead_code)]
    pub async fn get_trade_offers(&self, item_key: Option<&str>) -> Result<Vec<TradeOfferInfo>, sqlx::Error> {
        let rows: Vec<(i64, i64, String, String, i32, i64, String)> = if let Some(key) = item_key {
            sqlx::query_as(
                r#"
                SELECT id, seller_id, seller_name, item_key, quantity, price, created_at
                FROM trade_offers
                WHERE item_key = ?
                ORDER BY price ASC
                "#
            )
            .bind(key)
            .fetch_all(&self.pool)
            .await?
        } else {
            sqlx::query_as(
                r#"
                SELECT id, seller_id, seller_name, item_key, quantity, price, created_at
                FROM trade_offers
                ORDER BY created_at DESC
                LIMIT 100
                "#
            )
            .fetch_all(&self.pool)
            .await?
        };

        Ok(rows.into_iter().map(|(id, seller_id, seller_name, item_key, quantity, price, created_at)| {
            TradeOfferInfo {
                id,
                seller_id,
                seller_name,
                item_key,
                quantity: quantity as u32,
                price,
                created_at,
            }
        }).collect())
    }

    #[allow(dead_code)]
    pub async fn remove_trade_offer(&self, offer_id: i64) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM trade_offers WHERE id = ?")
            .bind(offer_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    // ========================================================================
    // MAIL
    // ========================================================================

    #[allow(dead_code)]
    pub async fn send_mail(
        &self,
        from_id: i64,
        from_name: &str,
        to_id: i64,
        subject: &str,
        body: &str,
        gold: i64,
        item: Option<(&str, u32)>,
    ) -> Result<i64, sqlx::Error> {
        let result = sqlx::query(
            r#"
            INSERT INTO mail (from_id, from_name, to_id, subject, body, gold_attached, item_attached, item_quantity, sent_at)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, datetime('now', '-5 hours'))
            "#
        )
        .bind(from_id)
        .bind(from_name)
        .bind(to_id)
        .bind(subject)
        .bind(body)
        .bind(gold)
        .bind(item.map(|(k, _)| k))
        .bind(item.map(|(_, q)| q as i32))
        .execute(&self.pool)
        .await?;

        Ok(result.last_insert_rowid())
    }

    #[allow(dead_code)]
    pub async fn get_unread_mail_count(&self, user_id: i64) -> Result<i64, sqlx::Error> {
        let row: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM mail WHERE to_id = ? AND is_read = 0"
        )
        .bind(user_id)
        .fetch_one(&self.pool)
        .await?;
        Ok(row.0)
    }

    // ========================================================================
    // SKILLS SYNC
    // ========================================================================

    pub async fn sync_skills(&self, user_id: i64, skills: &[(String, u32)]) -> Result<(), sqlx::Error> {
        // Delete existing and re-insert (simple approach)
        sqlx::query("DELETE FROM character_skills WHERE user_id = ?")
            .bind(user_id)
            .execute(&self.pool)
            .await?;

        for (skill_key, level) in skills {
            if *level > 0 {
                sqlx::query(
                    "INSERT INTO character_skills (user_id, skill_key, skill_level) VALUES (?, ?, ?)"
                )
                .bind(user_id)
                .bind(skill_key)
                .bind(*level as i32)
                .execute(&self.pool)
                .await?;
            }
        }

        Ok(())
    }
}

// ============================================================================
// DATA STRUCTURES
// ============================================================================

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct VisiblePlayer {
    pub name: String,
    pub level: u32,
    pub x: i32,
    pub y: i32,
    pub guild: Option<String>,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct OnlinePlayer {
    pub user_id: i64,
    pub handle: String,
    pub char_name: String,
    pub level: u32,
    pub zone: String,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct LeaderboardEntry {
    pub rank: i64,
    pub handle: String,
    pub char_name: String,
    pub level: u32,
    pub total_xp: i64,
    pub kills: u32,
    pub pvp_kills: u32,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct SkillLeaderboardEntry {
    pub rank: i64,
    pub handle: String,
    pub char_name: String,
    pub skill_level: u32,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct HouseInfo {
    pub id: i64,
    pub owner_name: String,
    pub house_type: String,
    pub x: i32,
    pub y: i32,
    pub name: Option<String>,
    pub is_public: bool,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct TradeOfferInfo {
    pub id: i64,
    pub seller_id: i64,
    pub seller_name: String,
    pub item_key: String,
    pub quantity: u32,
    pub price: i64,
    pub created_at: String,
}
