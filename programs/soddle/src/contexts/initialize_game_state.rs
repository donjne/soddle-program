use anchor_lang::prelude::*;
use crate::states::game_state::GameState;

#[derive(Accounts)]
pub struct InitializeGameState<'info> {
    #[account(
        init,
        payer = payer,
        space = 8 + GameState::INIT_SPACE,
        seeds = [b"game_state", payer.key().as_ref()],
        bump
    )]
    pub game_state: Account<'info, GameState>,

    #[account(mut)]
    pub payer: Signer<'info>,
    pub system_program: Program<'info, System>,
}

pub fn initialize_game_state(ctx: Context<InitializeGameState>) -> Result<()> {
    let game_state = &mut ctx.accounts.game_state;
    game_state.last_update_time = Clock::get()?.unix_timestamp;
    game_state.bump = ctx.bumps.game_state;
    Ok(())
}