use anchor_lang::prelude::*;
use crate::states::{kol::Kol, game_session::GameSession, game_metrics::GameMetrics, kol::Kol, fee_vault::FeeVault};
use crate::events::*;
use crate::errors::*;

#[derive(Accounts)]
pub struct StartGameSession<'info> {
    #[account(
        mut,
        seeds = [
            b"game_session",
            player.key().as_ref(),
            competition_id.to_le_bytes().as_ref(),
            start_time.to_le_bytes().as_ref()
        ],
        bump = game_session.bump,
        constraint = !game_session.is_expired() @ GameError::SessionExpired,
        constraint = game_session.player == player.key() @ GameError::InvalidPlayer,
        constraint = game_session.can_start_new_attempt() @ GameError::MaxAttemptsReached
    )]
    pub game_session: Account<'info, GameSession>,
    
    #[account(
        seeds = [
            b"kol",
            kol.id.to_le_bytes().as_ref()
        ],
        bump = kol.bump,
        constraint = game_session.kol == kol.key() @ GameError::InvalidKol
    )]
    pub kol: Account<'info, Kol>,

    #[account(
        mut,
        seeds = [b"game_metrics"],
        bump = metrics.bump
    )]
    pub metrics: Account<'info, GameMetrics>,

    #[account(
        mut,
        seeds = [b"fee_vault"],
        bump,
        constraint = fee_vault.key() == metrics.fee_vault @ SoddleError::InvalidFeeVault
    )]
    pub fee_vault: Account<'info, FeeVault>,

    #[account(
        mut,
        constraint = player.lamports() >= GAME_FEE @ SoddleError::InsufficientFunds
    )]
    pub player: Signer<'info>,
    pub system_program: Program<'info, System>,
}

pub fn start_game_session(
    ctx: Context<StartGameSession>,
    competition_id: u64,
    start_time: i64,
) -> Result<()> {
    let game_session = &mut ctx.accounts.game_session;
    let metrics = &mut ctx.accounts.metrics;
    let current_time = Clock::get()?.unix_timestamp;
    
    // Validate session state
    require!(
        game_session.can_start_new_attempt(),
        SoddleError::MaxAttemptsReached
    );

    // Validate timestamp
    require!(
        start_time >= current_time,
        SoddleError::InvalidStartTime
    );

    // Transfer fee from player to fee vault
    let cpi_context = CpiContext::new(
        ctx.accounts.system_program.to_account_info(),
        system_program::Transfer {
            from: ctx.accounts.player.to_account_info(),
            to: ctx.accounts.fee_vault.to_account_info(),
        },
    );

    system_program::transfer(cpi_context, GAME_FEE)?;

    // Update metrics with checked arithmetic
    metrics.total_fees_collected = metrics.total_fees_collected
        .checked_add(GAME_FEE)
        .ok_or(SoddleError::NumericOverflow)?;
        
    metrics.total_games_started = metrics.total_games_started
        .checked_add(1)
        .ok_or(SoddleError::NumericOverflow)?;

    // Update game session state
    let attempt_index = game_session.current_attempt as usize;
    game_session.attempts[attempt_index] = GameAttempt {
        score: INITIAL_SCORE,
        guesses: 0,
        completed: false,
        timestamp: current_time,
    };

    game_session.current_attempt = game_session.current_attempt
        .checked_add(1)
        .ok_or(SoddleError::NumericOverflow)?;
        
    game_session.deposit = game_session.deposit
        .checked_add(GAME_FEE)
        .ok_or(SoddleError::NumericOverflow)?;


    // Emit event
    emit!(GameSessionStarted {
        player: ctx.accounts.player.key(),
        kol: ctx.accounts.kol.key(),
        attempt: game_session.current_attempt,
        timestamp: current_time,
    });

    Ok(())
}