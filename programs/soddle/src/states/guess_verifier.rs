use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct GuessVerifier {
    pub oracle_authority: Pubkey,  // Oracle's public key
    pub answer_hash: [u8; 32],    // Hash of the correct answer
    pub bump: u8,
}