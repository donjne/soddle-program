use anchor_lang::prelude::*;
use crate::states::game_metrics::GameMetrics;

#[derive(Accounts)]
pub struct InitializeGameMetrics<'info> {
    #[account(
        init_if_needed,
        payer = authority,
        space = 8 + GameMetrics::INIT_SPACE,
        seeds = [b"game_metrics"],
        bump
    )]
    pub metrics: Account<'info, GameMetrics>,

    #[account(
        init_if_needed,
        payer = authority,
        space = 8 + FeeVault::INIT_SPACE,
        seeds = [b"fee_vault"],
        bump
    )]
    pub fee_vault: Account<'info, FeeVault>,

    #[account(mut)]
    pub authority: Signer<'info>,
    pub system_program: Program<'info, System>,
}

pub fn initialize_metrics(
    ctx: Context<InitializeGameMetrics>
) -> Result<()> {
    let metrics = &mut ctx.accounts.metrics;
    let fee_vault = &mut ctx.accounts.fee_vault;

    // Initialize metrics account
    metrics.total_fees_collected = 0;
    metrics.total_games_started = 0;
    metrics.authority = ctx.accounts.authority.key();
    metrics.fee_vault = ctx.accounts.fee_vault.key();
    metrics.bump = ctx.bumps.metrics;

    // Initialize fee vault account
    fee_vault.authority = ctx.accounts.authority.key();
    fee_vault.total_collected = 0;
    fee_vault.bump = ctx.bumps.fee_vault;

    // Log initialization
    msg!("Initialized GameMetrics with authority: {}", metrics.authority);
    msg!("Initialized FeeVault with authority: {}", fee_vault.authority);

    Ok(())
}