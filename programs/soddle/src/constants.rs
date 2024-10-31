use anchor_lang::prelude::*;

pub const GAME_FEE: u64 = 20_000_000;  // 0.02 SOL in lamports
pub const MAX_DAILY_ATTEMPTS: u8 = 3;
pub const MAX_GUESSES: u32 = 20;
pub const ATTEMPT_TIME_LIMIT: i64 = 5 * 60; // 5 minutes in seconds
pub const INITIAL_SCORE: u32 = 1000;
pub const TIME_PENALTY_INTERVAL: i64 = 5; // 5 seconds
pub const WRONG_GUESS_PENALTY: u32 = 50;
pub const TIME_PENALTY: u32 = 5; // Points deducted every 5 seconds