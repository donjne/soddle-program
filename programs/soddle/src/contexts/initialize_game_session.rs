use anchor_lang::prelude::*;
use crate::states::{kol::Kol, game_session::GameSession, game_attempt::GameAttempt};


#[derive(Accounts)]
pub struct InitializeGameSession<'info> {
    #[account(
        init,
        payer = player,
        space = 8 + GameSession::INIT_SPACE,
        seeds = [
            b"game_session",
            player.key().as_ref(),
            competition_id.to_le_bytes().as_ref(),
            start_time.to_le_bytes().as_ref()
        ],
        bump
    )]
    pub game_session: Account<'info, GameSession>,
    
    // Verify the KOL exists
    #[account(
        seeds = [
            b"kol",
            kol.id.to_le_bytes().as_ref()
        ],
        bump = kol.bump,
    )]
    pub kol: Account<'info, Kol>,
    
    #[account(mut)]
    pub player: Signer<'info>,
    pub system_program: Program<'info, System>,
}

pub fn initialize_game_session(
    ctx: Context<InitializeGameSession>,
    competition_id: u64,
    start_time: i64,
) -> Result<()> {
    let current_time = Clock::get()?.unix_timestamp;
    require!(
        start_time >= current_time,
        GameError::InvalidStartTime
    );
    let game_session = &mut ctx.accounts.game_session;
    game_session.player = ctx.accounts.player.key();
    game_session.start_time = current_timestamp;
    game_session.total_score = 0;
    game_session.deposit = 0;
    game_session.kol = ctx.accounts.kol.key();  // Store KOL's PDA address
    game_session.competition_id = competition_id;
    game_session.current_attempt = 0;
    game_session.bump = ctx.bumps.game_session;
    
    game_session.attempts = [GameAttempt {
        score: 0,
        guesses: 0,
        completed: false,
        timestamp: 0,
    }; 3];
    
    Ok(())
}