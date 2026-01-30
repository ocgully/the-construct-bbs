use super::state::GameState;
use rand::prelude::*;

// ============================================================================
// LOAN SHARK
// ============================================================================

/// Borrow money from loan shark
/// Returns Ok(new_debt_total) or Err(message)
pub fn borrow_money(state: &mut GameState, amount: i64) -> Result<i64, String> {
    if amount <= 0 {
        return Err("Amount must be positive.".to_string());
    }

    // Max borrow is 2x current debt
    let max_borrow = state.debt * 2;
    if amount > max_borrow {
        return Err(format!(
            "The loan shark laughs. 'You? I'll give you {} max.'",
            super::render::format_money(max_borrow)
        ));
    }

    state.cash += amount;
    state.debt += amount;

    Ok(state.debt)
}

/// Pay back loan shark debt
/// Returns Ok(remaining_debt) or Err(message)
#[allow(dead_code)]
pub fn pay_debt(state: &mut GameState, amount: i64) -> Result<i64, String> {
    if amount <= 0 {
        return Err("Amount must be positive.".to_string());
    }

    if amount > state.cash {
        return Err("You don't have that much cash!".to_string());
    }

    // Can't pay more than owed
    let actual_payment = amount.min(state.debt);
    state.cash -= actual_payment;
    state.debt -= actual_payment;

    Ok(state.debt)
}

/// Pay off all debt
pub fn pay_all_debt(state: &mut GameState) -> Result<i64, String> {
    if state.debt > state.cash {
        return Err(format!(
            "You need {} to pay off your debt.",
            super::render::format_money(state.debt)
        ));
    }

    state.cash -= state.debt;
    state.debt = 0;

    Ok(0)
}

// ============================================================================
// BANK
// ============================================================================

/// Check if bank is available (requires $50,000 to unlock)
#[allow(dead_code)]
pub fn check_bank_unlock(state: &mut GameState) -> bool {
    if !state.bank_unlocked && state.cash >= 5000000 {
        state.bank_unlocked = true;
    }
    state.bank_unlocked
}

/// Deposit cash into bank
/// Returns Ok(new_balance) or Err(message)
#[allow(dead_code)]
pub fn deposit(state: &mut GameState, amount: i64) -> Result<i64, String> {
    if !state.bank_unlocked {
        return Err("Bank requires $50,000 minimum to open account.".to_string());
    }

    if amount <= 0 {
        return Err("Amount must be positive.".to_string());
    }

    if amount > state.cash {
        return Err("You don't have that much cash!".to_string());
    }

    state.cash -= amount;
    state.bank_balance += amount;

    Ok(state.bank_balance)
}

/// Withdraw cash from bank
/// Returns Ok(new_balance) or Err(message)
#[allow(dead_code)]
pub fn withdraw(state: &mut GameState, amount: i64) -> Result<i64, String> {
    if !state.bank_unlocked {
        return Err("You don't have a bank account.".to_string());
    }

    if amount <= 0 {
        return Err("Amount must be positive.".to_string());
    }

    if amount > state.bank_balance {
        return Err("Insufficient funds in bank.".to_string());
    }

    state.bank_balance -= amount;
    state.cash += amount;

    Ok(state.bank_balance)
}

/// Deposit all cash
pub fn deposit_all(state: &mut GameState) -> Result<i64, String> {
    if !state.bank_unlocked {
        return Err("Bank requires $50,000 minimum to open account.".to_string());
    }

    let amount = state.cash;
    if amount == 0 {
        return Err("You have no cash to deposit.".to_string());
    }

    state.cash = 0;
    state.bank_balance += amount;

    Ok(state.bank_balance)
}

/// Withdraw all from bank
pub fn withdraw_all(state: &mut GameState) -> Result<i64, String> {
    if !state.bank_unlocked {
        return Err("You don't have a bank account.".to_string());
    }

    let amount = state.bank_balance;
    if amount == 0 {
        return Err("Your bank account is empty.".to_string());
    }

    state.bank_balance = 0;
    state.cash += amount;

    Ok(0)
}

// ============================================================================
// CASINO
// ============================================================================

/// Blackjack result
#[derive(Debug, Clone)]
pub enum BlackjackResult {
    Win { winnings: i64 },
    Lose { lost: i64 },
    Push, // Tie
    Blackjack { winnings: i64 }, // 3:2 payout
}

/// Play blackjack
/// Returns result and message
pub fn play_blackjack(state: &mut GameState, bet: i64) -> Result<(BlackjackResult, String), String> {
    if bet <= 0 {
        return Err("Bet must be positive.".to_string());
    }
    if bet > state.cash {
        return Err("You don't have enough cash!".to_string());
    }

    let mut rng = thread_rng();

    // Simplified blackjack: draw 2 cards, stand or hit once
    // Card value: 1-10, with 10 (J,Q,K) being common
    let mut draw_card = || -> u8 {
        let card = rng.gen_range(1..=13);
        if card > 10 { 10 } else { card }
    };

    let player_card1 = draw_card();
    let player_card2 = draw_card();
    let dealer_card1 = draw_card();
    let dealer_card2 = draw_card();

    let player_total = player_card1 as u32 + player_card2 as u32;
    let dealer_total = dealer_card1 as u32 + dealer_card2 as u32;

    // Check for natural blackjack (21 with 2 cards)
    let player_blackjack = player_total == 21;
    let dealer_blackjack = dealer_total == 21;

    let result = if player_blackjack && dealer_blackjack {
        // Both blackjack = push
        BlackjackResult::Push
    } else if player_blackjack {
        // Player blackjack = 3:2 payout
        let winnings = (bet * 3) / 2;
        state.cash += winnings;
        BlackjackResult::Blackjack { winnings }
    } else if dealer_blackjack {
        state.cash -= bet;
        BlackjackResult::Lose { lost: bet }
    } else if player_total > dealer_total && player_total <= 21 {
        state.cash += bet;
        BlackjackResult::Win { winnings: bet }
    } else if dealer_total > 21 {
        state.cash += bet;
        BlackjackResult::Win { winnings: bet }
    } else if player_total == dealer_total {
        BlackjackResult::Push
    } else {
        state.cash -= bet;
        BlackjackResult::Lose { lost: bet }
    };

    let message = match &result {
        BlackjackResult::Win { winnings } => {
            format!("You win {}! (You: {}, Dealer: {})", super::render::format_money(*winnings), player_total, dealer_total)
        }
        BlackjackResult::Lose { lost } => {
            format!("You lose {}. (You: {}, Dealer: {})", super::render::format_money(*lost), player_total, dealer_total)
        }
        BlackjackResult::Push => {
            format!("Push! (Both: {})", player_total)
        }
        BlackjackResult::Blackjack { winnings } => {
            format!("BLACKJACK! You win {}!", super::render::format_money(*winnings))
        }
    };

    Ok((result, message))
}

/// Roulette bet type
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum RouletteBet {
    Red,
    Black,
    Odd,
    Even,
    Number(u8), // 0-36
}

/// Play roulette
pub fn play_roulette(state: &mut GameState, bet: i64, bet_type: RouletteBet) -> Result<(bool, i64, String), String> {
    if bet <= 0 {
        return Err("Bet must be positive.".to_string());
    }
    if bet > state.cash {
        return Err("You don't have enough cash!".to_string());
    }

    let mut rng = thread_rng();
    let result = rng.gen_range(0..=36);

    // Red numbers: 1,3,5,7,9,12,14,16,18,19,21,23,25,27,30,32,34,36
    let red_numbers = [1,3,5,7,9,12,14,16,18,19,21,23,25,27,30,32,34,36];
    let is_red = red_numbers.contains(&result);
    let is_black = result != 0 && !is_red;

    let (won, payout) = match bet_type {
        RouletteBet::Red => (is_red, bet),
        RouletteBet::Black => (is_black, bet),
        RouletteBet::Odd => (result != 0 && result % 2 == 1, bet),
        RouletteBet::Even => (result != 0 && result % 2 == 0, bet),
        RouletteBet::Number(n) => (result == n, bet * 35), // 35:1 payout
    };

    let color = if result == 0 { "green" } else if is_red { "red" } else { "black" };

    if won {
        state.cash += payout;
        Ok((true, payout, format!("The ball lands on {} {}! You win {}!", result, color, super::render::format_money(payout))))
    } else {
        state.cash -= bet;
        Ok((false, bet, format!("The ball lands on {} {}. You lose {}.", result, color, super::render::format_money(bet))))
    }
}

/// Horse race betting
pub fn bet_on_horse(state: &mut GameState, bet: i64, horse: u8) -> Result<(bool, i64, String), String> {
    if bet <= 0 {
        return Err("Bet must be positive.".to_string());
    }
    if bet > state.cash {
        return Err("You don't have enough cash!".to_string());
    }
    if horse < 1 || horse > 6 {
        return Err("Pick a horse 1-6.".to_string());
    }

    let mut rng = thread_rng();

    // Horse odds (1:2 = 2x, 3:1 = 4x, 5:1 = 6x)
    let odds = [2, 3, 4, 5, 6, 8]; // Horse 1-6 odds
    let win_chances = [40, 30, 25, 20, 15, 10]; // Percent chance to win

    let horse_idx = (horse - 1) as usize;
    let win_chance = win_chances[horse_idx];
    let won = rng.gen_range(0..100) < win_chance;

    if won {
        let payout = bet * odds[horse_idx] as i64;
        state.cash += payout;
        Ok((true, payout, format!("Horse #{} wins! You collect {}!", horse, super::render::format_money(payout))))
    } else {
        let winner = rng.gen_range(1..=6);
        state.cash -= bet;
        Ok((false, bet, format!("Horse #{} wins. You lose {}.", winner, super::render::format_money(bet))))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_borrow_money() {
        let mut state = GameState::new();
        let initial_cash = state.cash;
        let initial_debt = state.debt;

        let amount = 100000; // $1,000
        let result = borrow_money(&mut state, amount);

        assert!(result.is_ok());
        assert_eq!(state.cash, initial_cash + amount);
        assert_eq!(state.debt, initial_debt + amount);
    }

    #[test]
    fn test_borrow_exceeds_max() {
        let mut state = GameState::new();
        state.debt = 100000; // $1,000

        // Max borrow is 2x debt = $2,000
        let result = borrow_money(&mut state, 300000); // Try to borrow $3,000

        assert!(result.is_err());
    }

    #[test]
    fn test_pay_debt() {
        let mut state = GameState::new();
        state.debt = 100000; // $1,000
        state.cash = 200000; // $2,000

        let result = pay_debt(&mut state, 50000); // Pay $500

        assert!(result.is_ok());
        assert_eq!(state.debt, 50000);
        assert_eq!(state.cash, 150000);
    }

    #[test]
    fn test_bank_unlock() {
        let mut state = GameState::new();
        state.cash = 5000000; // $50,000

        assert!(!state.bank_unlocked);
        let unlocked = check_bank_unlock(&mut state);
        assert!(unlocked);
        assert!(state.bank_unlocked);
    }

    #[test]
    fn test_deposit() {
        let mut state = GameState::new();
        state.bank_unlocked = true;
        state.cash = 100000; // $1,000

        let result = deposit(&mut state, 50000); // Deposit $500

        assert!(result.is_ok());
        assert_eq!(state.cash, 50000);
        assert_eq!(state.bank_balance, 50000);
    }

    #[test]
    fn test_withdraw() {
        let mut state = GameState::new();
        state.bank_unlocked = true;
        state.bank_balance = 100000; // $1,000
        state.cash = 0;

        let result = withdraw(&mut state, 50000); // Withdraw $500

        assert!(result.is_ok());
        assert_eq!(state.cash, 50000);
        assert_eq!(state.bank_balance, 50000);
    }
}
