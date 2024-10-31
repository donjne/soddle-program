use anchor_lang::prelude::*;

#[event]
pub struct GameSessionStarted {
    pub player: Pubkey,
    pub kol: Pubkey,
    pub attempt: u8,
    pub timestamp: i64,
}

#[event]
pub struct GuessAttempted {
    pub player: Pubkey,
    pub attempt_index: u8,
    pub guess_number: u32,
    pub current_score: u32,
    pub was_correct: bool,
    pub timestamp: i64,
    pub time_deduction: u32,
    pub wrong_guess_penalty: u32,
}

#[event]
pub struct GameSessionCompleted {
    pub player: Pubkey,
    pub kol: Pubkey,
    pub final_score: u32,
    pub total_attempts: u8,
    pub timestamp: i64,
}

#[event]
pub struct AttemptCompleted {
    pub player: Pubkey,
    pub attempt_index: u8,
    pub final_score: u32,
    pub total_guesses: u32,
    pub was_correct: bool,
    pub timestamp: i64,
    pub reason: CompletionReason,
}

#[event]
pub struct GameSessionEnded {
    pub player: Pubkey,
    pub competition_id: u64,
    pub total_attempts: u8,
    pub best_score: u32,
    pub total_score: u32,
    pub deposit_refunded: bool,
    pub refund_amount: u64,
    pub timestamp: i64,
}

#[event]
pub struct NewHighScore {
    pub player: Pubkey,
    pub score: u32,
    pub competition_id: u64,
    pub timestamp: i64,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub enum CompletionReason {
    CorrectGuess,
    MaxGuesses,
    TimeExpired,
}