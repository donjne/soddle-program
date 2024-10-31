use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct GameMetrics {
    pub total_fees_collected: u64,
    pub total_games_started: u64,
    pub authority: Pubkey,      // Program authority who can withdraw fees
    pub fee_vault: Pubkey,      // PDA that holds the fees
    pub bump: u8,
}