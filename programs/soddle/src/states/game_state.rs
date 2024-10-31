use anchor_lang::prelude::*;
use crate::states::competition::Competition;

#[account]
#[derive(InitSpace)]
pub struct GameState {
    pub last_update_time: i64,
    pub bump: u8
}