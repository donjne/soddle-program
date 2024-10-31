use anchor_lang::prelude::*;
use crate::states::game_state::GameState;
use crate::errors::SoddleError;

#[derive(Accounts)]
pub struct InitializeCompetition<'info> {
    #[account(mut)]
    pub game_state: Account<'info, GameState>,

    #[account(
        init,
        payer = payer,
        space = 8 + Competition::INIT_SPACE,
        seeds = [
            b"competition",
            game_state.key().as_ref(),
            competition.id.to_le_bytes().as_ref()
        ],
        bump
    )]
    pub competition: Account<'info, Competition>,

    #[account(mut)]
    pub payer: Signer<'info>,
    pub system_program: Program<'info, System>,
}

pub fn initialize_competition(
    ctx: Context<InitializeCompetition>,
    id: u64,
    start_time: i64,
    end_time: i64,
) -> Result<()> {
    require!(start_time < end_time, SoddleError::InvalidTimeRange);
    require!(
        Clock::get()?.unix_timestamp <= start_time,
        SoddleError::CompetitionStarted
    );

    let competition = &mut ctx.accounts.competition;
    competition.id = id;
    competition.start_time = start_time;
    competition.end_time = end_time;
    competition.game_state = ctx.accounts.game_state.key();
    competition.bump = ctx.bumps.competition;

    Ok(())
}