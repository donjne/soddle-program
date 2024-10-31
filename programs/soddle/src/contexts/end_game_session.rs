use anchor_lang::prelude::*;
use anchor_lang::system_program;

#[derive(Accounts)]
pub struct EndGameSession<'info> {
    #[account(
        mut,
        seeds = [
            b"game_session",
            player.key().as_ref(),
            game_session.competition_id.to_le_bytes().as_ref(),
            game_session.start_time.to_le_bytes().as_ref()
        ],
        bump = game_session.bump,
        constraint = !game_session.is_expired() @ GameError::SessionExpired,
        constraint = game_session.player == player.key() @ GameError::InvalidPlayer,
        close = player
    )]
    pub game_session: Account<'info, GameSession>,

    #[account(
        mut,
        seeds = [b"game_metrics"],
        bump = metrics.bump
    )]
    pub metrics: Account<'info, GameMetrics>,

    #[account(
        mut,
        seeds = [b"fee_vault"],
        bump = fee_vault.bump
    )]
    pub fee_vault: Account<'info, FeeVault>,

    #[account(mut)]
    pub player: Signer<'info>,
    pub system_program: Program<'info, System>,
}


    pub fn end_game_session(ctx: Context<EndGameSession>) -> Result<()> {
        let game_session = &ctx.accounts.game_session;
        let metrics = &mut ctx.accounts.metrics;
        let current_time = Clock::get()?.unix_timestamp;

        // 1. Validate game can be ended
        require!(
            game_session.can_end_session(),
            GameError::CannotEndGameYet
        );

        // 2. Update metrics
        metrics.total_games_completed = metrics.total_games_completed
            .checked_add(1)
            .ok_or(GameError::NumericOverflow)?;

        // Calculate best score from attempts
        let best_score = game_session.attempts
            .iter()
            .map(|attempt| attempt.score)
            .max()
            .unwrap_or(0);

        // Update high score metrics if applicable
        if best_score > metrics.highest_score {
            metrics.highest_score = best_score;
            metrics.highest_scorer = game_session.player;
        }

        // 3. Handle deposit refund if conditions met
        if game_session.should_refund_deposit() && game_session.deposit > 0 {
            let signer_seeds = &[
                b"fee_vault".as_ref(),
                &[ctx.accounts.fee_vault.bump],
            ];

            // Transfer deposit back to player
            let transfer_ix = system_program::Transfer {
                from: ctx.accounts.fee_vault.to_account_info(),
                to: ctx.accounts.player.to_account_info(),
            };

            let cpi_ctx = CpiContext::new_with_signer(
                ctx.accounts.system_program.to_account_info(),
                transfer_ix,
                &[&signer_seeds[..]],
            );

            system_program::transfer(cpi_ctx, game_session.deposit)?;

            // Update fee vault total
            ctx.accounts.fee_vault.total_collected = ctx.accounts.fee_vault.total_collected
                .checked_sub(game_session.deposit)
                .ok_or(GameError::NumericOverflow)?;
        }

        // 4. Emit completion events
        emit!(GameSessionEnded {
            player: game_session.player,
            competition_id: game_session.competition_id,
            total_attempts: game_session.current_attempt,
            best_score,
            total_score: game_session.total_score,
            deposit_refunded: game_session.should_refund_deposit(),
            refund_amount: if game_session.should_refund_deposit() { 
                game_session.deposit 
            } else { 
                0 
            },
            timestamp: current_time,
        });

        if best_score > metrics.highest_score {
            emit!(NewHighScore {
                player: game_session.player,
                score: best_score,
                competition_id: game_session.competition_id,
                timestamp: current_time,
            });
        }

        // Account will be automatically closed to player due to close = player constraint
        Ok(())
    }