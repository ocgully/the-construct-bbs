//! Scoring system for Acromania
//!
//! Handles point calculation for submissions, votes, and bonuses.

/// Points awarded per vote received
pub const POINTS_PER_VOTE: i64 = 100;

/// Participation points for submitting (even if no votes)
pub const PARTICIPATION_POINTS: i64 = 10;

/// Bonus for unanimous vote (all players voted for you)
pub const UNANIMOUS_BONUS: i64 = 200;

/// Maximum speed bonus points
pub const MAX_SPEED_BONUS: i64 = 50;

/// Face-off winner points
pub const FACEOFF_WINNER: i64 = 500;

/// Face-off loser points
pub const FACEOFF_LOSER: i64 = 100;

/// Speed bonus calculation result
#[derive(Debug, Clone)]
pub struct SpeedBonus {
    pub points: i64,
    #[allow(dead_code)]
    pub submission_seconds: u64,
    #[allow(dead_code)]
    pub time_limit_seconds: u64,
}

impl SpeedBonus {
    /// Calculate speed bonus based on submission time
    /// Earlier submission = more bonus points (up to MAX_SPEED_BONUS)
    pub fn calculate(submission_seconds: u64, time_limit_seconds: u64) -> Self {
        let points = if submission_seconds >= time_limit_seconds {
            0
        } else {
            // Linear scale: submit at 0 seconds = MAX_SPEED_BONUS
            // Submit at time_limit = 0 bonus
            let remaining = time_limit_seconds - submission_seconds;
            let ratio = remaining as f64 / time_limit_seconds as f64;
            (ratio * MAX_SPEED_BONUS as f64) as i64
        };

        Self {
            points,
            submission_seconds,
            time_limit_seconds,
        }
    }
}

/// Calculate total score for a submission
#[allow(dead_code)]
pub fn calculate_score(
    votes_received: u32,
    total_voters: u32,
    speed_bonus: &SpeedBonus,
    is_faceoff: bool,
    is_winner: bool,
) -> i64 {
    let mut total = 0i64;

    // Base points from votes
    total += (votes_received as i64) * POINTS_PER_VOTE;

    // Participation bonus (if they submitted something)
    total += PARTICIPATION_POINTS;

    // Speed bonus
    total += speed_bonus.points;

    // Unanimous bonus (everyone voted for this submission)
    // Requires at least 2 voters to prevent solo-game abuse
    if votes_received > 0 && votes_received == total_voters && total_voters >= 2 {
        total += UNANIMOUS_BONUS;
    }

    // Face-off round special scoring
    if is_faceoff {
        if is_winner {
            total += FACEOFF_WINNER;
        } else {
            total += FACEOFF_LOSER;
        }
    }

    total
}

/// Result of scoring a round
#[derive(Debug, Clone)]
pub struct RoundResult {
    pub user_id: i64,
    pub handle: String,
    pub submission_text: String,
    pub votes_received: u32,
    #[allow(dead_code)]
    pub base_points: i64,
    pub speed_bonus: i64,
    pub unanimous_bonus: i64,
    pub total_points: i64,
    pub is_winner: bool,
}

/// Calculate all results for a round
pub fn calculate_round_results(
    submissions: &[(i64, String, String, u32, u64)], // (user_id, handle, text, votes, submission_seconds)
    time_limit_seconds: u64,
    total_voters: u32,
    is_faceoff: bool,
) -> Vec<RoundResult> {
    // Find max votes to determine winner(s)
    let max_votes = submissions.iter().map(|(_, _, _, v, _)| *v).max().unwrap_or(0);

    let mut results: Vec<RoundResult> = submissions
        .iter()
        .map(|(user_id, handle, text, votes, seconds)| {
            let speed = SpeedBonus::calculate(*seconds, time_limit_seconds);
            let is_winner = *votes == max_votes && max_votes > 0;

            let base_points = (*votes as i64) * POINTS_PER_VOTE + PARTICIPATION_POINTS;
            let unanimous = if *votes > 0 && *votes == total_voters && total_voters >= 2 {
                UNANIMOUS_BONUS
            } else {
                0
            };

            let faceoff_bonus = if is_faceoff {
                if is_winner { FACEOFF_WINNER } else { FACEOFF_LOSER }
            } else {
                0
            };

            let total = base_points + speed.points + unanimous + faceoff_bonus;

            RoundResult {
                user_id: *user_id,
                handle: handle.clone(),
                submission_text: text.clone(),
                votes_received: *votes,
                base_points,
                speed_bonus: speed.points,
                unanimous_bonus: unanimous,
                total_points: total,
                is_winner,
            }
        })
        .collect();

    // Sort by votes (descending), then by speed bonus (descending)
    results.sort_by(|a, b| {
        b.votes_received.cmp(&a.votes_received)
            .then(b.speed_bonus.cmp(&a.speed_bonus))
    });

    results
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_speed_bonus_max() {
        let bonus = SpeedBonus::calculate(0, 60);
        assert_eq!(bonus.points, MAX_SPEED_BONUS);
    }

    #[test]
    fn test_speed_bonus_zero_at_limit() {
        let bonus = SpeedBonus::calculate(60, 60);
        assert_eq!(bonus.points, 0);
    }

    #[test]
    fn test_speed_bonus_half() {
        let bonus = SpeedBonus::calculate(30, 60);
        assert_eq!(bonus.points, MAX_SPEED_BONUS / 2);
    }

    #[test]
    fn test_calculate_score_basic() {
        let speed = SpeedBonus::calculate(30, 60);
        let score = calculate_score(3, 5, &speed, false, false);

        // 3 votes * 100 = 300 + 10 participation + 25 speed = 335
        assert_eq!(score, 335);
    }

    #[test]
    fn test_calculate_score_unanimous() {
        let speed = SpeedBonus::calculate(60, 60); // No speed bonus
        let score = calculate_score(3, 3, &speed, false, false);

        // 3 votes * 100 = 300 + 10 participation + 0 speed + 200 unanimous = 510
        assert_eq!(score, 510);
    }

    #[test]
    fn test_calculate_score_faceoff_winner() {
        let speed = SpeedBonus::calculate(60, 60);
        let score = calculate_score(2, 3, &speed, true, true);

        // 2 * 100 + 10 + 0 + 500 = 710
        assert_eq!(score, 710);
    }

    #[test]
    fn test_calculate_score_faceoff_loser() {
        let speed = SpeedBonus::calculate(60, 60);
        let score = calculate_score(1, 3, &speed, true, false);

        // 1 * 100 + 10 + 0 + 100 = 210
        assert_eq!(score, 210);
    }

    #[test]
    fn test_round_results_sorted() {
        let submissions = vec![
            (1, "Alice".to_string(), "A B C".to_string(), 1, 30),
            (2, "Bob".to_string(), "A B C".to_string(), 3, 45),
            (3, "Carol".to_string(), "A B C".to_string(), 2, 20),
        ];

        let results = calculate_round_results(&submissions, 60, 3, false);

        // Should be sorted by votes: Bob (3), Carol (2), Alice (1)
        assert_eq!(results[0].handle, "Bob");
        assert_eq!(results[1].handle, "Carol");
        assert_eq!(results[2].handle, "Alice");
        assert!(results[0].is_winner);
        assert!(!results[1].is_winner);
        assert!(!results[2].is_winner);
    }

    #[test]
    fn test_unanimous_requires_two_voters() {
        let speed = SpeedBonus::calculate(60, 60);

        // Solo game - no unanimous bonus
        let score_solo = calculate_score(1, 1, &speed, false, false);
        assert_eq!(score_solo, 110); // 100 + 10, no unanimous

        // Two voters - gets unanimous
        let score_duo = calculate_score(2, 2, &speed, false, false);
        assert_eq!(score_duo, 410); // 200 + 10 + 200
    }
}
