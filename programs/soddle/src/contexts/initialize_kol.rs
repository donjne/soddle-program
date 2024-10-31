use anchor_lang::prelude::*;
use crate::states::kol::Kol;


#[derive(Accounts)]
pub struct InitializeKol<'info> {
    #[account(
        init,
        payer = payer,
        space = 8 + Kol::INIT_SPACE,
        seeds = [
            b"kol",
            kol.id.to_le_bytes().as_ref()
        ],
        bump
    )]
    pub kol: Account<'info, Kol>,
    
    #[account(mut)]
    pub payer: Signer<'info>,
    pub system_program: Program<'info, System>,
}

pub fn initialize_kol(
    ctx: Context<InitializeKol>,
    id: u64,
    name: String,
    age: u8,
    country: String,
    pfp_type: String,
    pfp: String,
    account_creation: u16,
    followers: u32,
    ecosystem: String,
) -> Result<()> {
    let kol = &mut ctx.accounts.kol;
    kol.id = id;
    kol.name = name;
    kol.age = age;
    kol.country = country;
    kol.pfp_type = pfp_type;
    kol.pfp = pfp;
    kol.account_creation = account_creation;
    kol.followers = followers;
    kol.ecosystem = ecosystem;
    kol.bump = ctx.bumps.kol;
    Ok(())
}