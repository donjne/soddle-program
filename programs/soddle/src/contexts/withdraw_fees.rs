use anchor_lang::prelude::*;
use crate::states::game_metrics::GameMetrics;

#[derive(Accounts)]
pub struct WithdrawFees<'info> {
    #[account(
        mut,
        seeds = [b"game_metrics"],
        bump = metrics.bump,
        constraint = metrics.authority == authority.key()
    )]
    pub metrics: Account<'info, GameMetrics>,

    #[account(
        mut,
        seeds = [b"fee_vault"],
        bump = fee_vault.bump,
        constraint = fee_vault.authority == authority.key()
    )]
    pub fee_vault: Account<'info, FeeVault>,

    /// CHECK: This is the destination wallet that will receive the fees
    #[account(
        mut,
        constraint = receiver.key() == metrics.authority
    )]
    pub receiver: UncheckedAccount<'info>,

    #[account(
        constraint = authority.key() == metrics.authority
    )]
    pub authority: Signer<'info>,
    pub system_program: Program<'info, System>,
}

pub fn withdraw_fees(
    ctx: Context<WithdrawFees>, 
    amount: u64
) -> Result<()> {
    require!(
        amount > 0,
        GameError::InvalidWithdrawAmount
    );

    let vault_balance = ctx.accounts.fee_vault.to_account_info().lamports();
    require!(
        amount <= vault_balance,
        GameError::InsufficientFunds
    );

    // Calculate minimum required balance for rent-exemption
    let rent = Rent::get()?;
    let minimum_balance = rent.minimum_balance(FeeVault::INIT_SPACE);
    
    require!(
        vault_balance.checked_sub(amount).unwrap() >= minimum_balance,
        GameError::InsufficientFundsForRent
    );

    // Transfer from fee vault to receiver using PDA signing
    let seeds = [
        b"fee_vault".as_ref(),
        &[ctx.accounts.fee_vault.bump],
    ];
    let signer_seeds = &[&seeds[..]];

    let transfer_ix = system_program::Transfer {
        from: ctx.accounts.fee_vault.to_account_info(),
        to: ctx.accounts.receiver.to_account_info(),
    };

    let cpi_ctx = CpiContext::new_with_signer(
        ctx.accounts.system_program.to_account_info(),
        transfer_ix,
        signer_seeds,
    );

    system_program::transfer(cpi_ctx, amount)?;

    // Update fee vault total collected
    ctx.accounts.fee_vault.total_collected = ctx.accounts.fee_vault
        .total_collected
        .checked_add(amount)
        .ok_or(SoddleError::NumericOverflow)?;

    Ok(())
}