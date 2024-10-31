use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct Competition {
    pub id: u64,
    pub start_time: i64,
    pub end_time: i64,
    pub game_state: Pubkey,  // Reference back to parent GameState
    pub bump: u8,
}