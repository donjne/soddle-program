use anchor_lang::prelude::*;
use crate::states::game_attempt::GameAttempt;
use crate::constants::*;

#[account]
#[derive(InitSpace)]
pub struct GameSession {
    pub player: Pubkey,
    pub start_time: i64,
    pub total_score: u32,
    pub deposit: u64,
    pub kol: Pubkey,  // Changed to store KOL's PDA address
    pub competition_id: u64,  // Changed to u64 to match Competition PDA
    pub current_attempt: u8,  // Tracks which attempt (1-3) the player is on
    pub attempts: [GameAttempt; 3],  // Fixed array for the 3 possible daily attempts
    pub bump: u8,
}

impl GameSession {
    pub fn is_expired(&self) -> bool {
        Clock::get().unwrap().unix_timestamp >= self.start_time + 24 * 60 * 60
    }

    pub fn can_start_new_attempt(&self) -> bool {
        self.current_attempt < MAX_DAILY_ATTEMPTS
    }

    pub fn can_end_session(&self) -> bool {
        let current_attempt = self.current_attempt.saturating_sub(1) as usize;
        
        // Can end if:
        // 1. No attempts made yet (forfeit)
        if self.current_attempt == 0 {
            return true;
        }

        // 2. Current attempt is completed
        if current_attempt < self.attempts.len() {
            return self.attempts[current_attempt].completed;
        }

        false
    }

    pub fn calculate_current_score(&self, current_time: i64, is_wrong_guess: bool) -> u32 {
        let current_attempt_index = self.current_attempt.saturating_sub(1) as usize;
        let attempt = &self.attempts[current_attempt_index];
        
        // Calculate time-based penalty
        let elapsed_time = current_time - attempt.timestamp;
        let time_periods = elapsed_time / TIME_PENALTY_INTERVAL;
        let time_deduction = (time_periods as u32).saturating_mul(TIME_PENALTY);
        
        // Calculate wrong guess penalty if applicable
        let wrong_guess_deduction = if is_wrong_guess {
            WRONG_GUESS_PENALTY
        } else {
            0
        };

        // Calculate final score
        INITIAL_SCORE
            .saturating_sub(time_deduction)
            .saturating_sub(wrong_guess_deduction)
    }

    pub fn should_end_game(&self) -> bool {
        let current_attempt_index = self.current_attempt.saturating_sub(1) as usize;
        let attempt = &self.attempts[current_attempt_index];
        
        let current_time = Clock::get().unwrap().unix_timestamp;
        let time_expired = current_time - attempt.timestamp >= ATTEMPT_TIME_LIMIT;
        let max_guesses_reached = attempt.guesses >= MAX_GUESSES;
        
        time_expired || max_guesses_reached
    }

    pub fn should_refund_deposit(&self) -> bool {
        // Example refund conditions:
        // 1. Player got a correct guess within first 5 guesses
        // 2. Or scored above certain threshold
        let has_winning_attempt = self.attempts.iter().any(|attempt| {
            attempt.completed && 
            attempt.guesses <= 5 && 
            attempt.score >= 800  // High score threshold
        });

        has_winning_attempt
    }
}
