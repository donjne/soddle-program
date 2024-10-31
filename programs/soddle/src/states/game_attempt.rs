use anchor_lang::prelude::*;

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct GameAttempt {
    pub score: u32,
    pub guesses: u32,
    pub completed: bool,
    pub timestamp: i64,
}