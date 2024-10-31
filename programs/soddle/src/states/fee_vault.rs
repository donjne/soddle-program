use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct FeeVault {
    pub authority: Pubkey,
    pub total_collected: u64,
    pub bump: u8,
}