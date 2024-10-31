use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct InitializeVerifier<'info> {
    #[account(
        init,
        payer = oracle_authority,
        space = 8 + GuessVerifier::INIT_SPACE,
        seeds = [
            b"guess_verifier",
            game_session.key().as_ref(),
        ],
        bump
    )]
    pub verifier: Account<'info, GuessVerifier>,

    pub game_session: Account<'info, GameSession>,

    #[account(mut)]
    pub oracle_authority: Signer<'info>,
    pub system_program: Program<'info, System>,
}

pub fn initialize_verifier(
    ctx: Context<InitializeVerifier>, 
    answer_hash: [u8; 32]
) -> Result<()> {
    let verifier = &mut ctx.accounts.verifier;
    verifier.oracle_authority = ctx.accounts.oracle_authority.key();
    verifier.answer_hash = answer_hash;
    verifier.bump = ctx.bumps.verifier;
    Ok(())
}