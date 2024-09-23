use anchor_lang::prelude::*;
use anchor_lang::solana_program::native_token::LAMPORTS_PER_SOL;

declare_id!("9oHdf9UFE9zbQPUDiBn27qX9Rqc7fVEpHSdxVpkwx6z4");

const REQUIRED_DEPOSIT: u64 = (0.001 * LAMPORTS_PER_SOL as f64) as u64;
const COMPETITION_DURATION: i64 = 24 * 60 * 60; // 24 hours in seconds

#[program]
pub mod soddle_game {
    use super::*;

    pub fn initialize_game(ctx: Context<InitializeGame>) -> Result<()> {
        let game_state = &mut ctx.accounts.game_state;

        let clock = Clock::get()?;
        let current_time: i64 = clock.unix_timestamp;

        // Generate a more unique ID
        let id = format!("COMP{:05}", current_time % 100000);

        let end_time = current_time + COMPETITION_DURATION;

        game_state.current_competition = Competition {
            id,
            start_time: current_time,
            end_time,
        };  // Added semicolon here

        game_state.last_update_time = current_time;

        Ok(())
    }

    pub fn start_game_session(
        ctx: Context<StartGameSession>,
        game_type: u8,
        kol: KOL,
    ) -> Result<()> {
        require!(
        game_type >= 1 && game_type <= 3,
        SoddleError::InvalidGameType
    );
        let game_session = &mut ctx.accounts.game_session;

        // Check if this is a new session or if the existing session is completed
        if game_session.player == Pubkey::default() || game_session.completed || game_session.game_1_completed || game_session.game_2_completed {
            // This is a new session or the existing one is completed, reinitialize it
            game_session.player = ctx.accounts.player.key();
            game_session.competition_id = ctx.accounts.game_state.current_competition.id.clone();
            game_session.deposit = 0; // Will be updated after transfer
            game_session.completed = false;
            game_session.game_1_completed = false;
            game_session.game_2_completed = false;
        }

        // Check if the player has already completed this game type
        match game_type {
            1 => require!(!game_session.game_1_completed, SoddleError::GameAlreadyPlayed),
            2 => require!(!game_session.game_2_completed, SoddleError::GameAlreadyPlayed),
            _ => return Err(SoddleError::InvalidGameType.into()),
        }

        // Transfer deposit
        anchor_lang::solana_program::program::invoke(
            &anchor_lang::solana_program::system_instruction::transfer(
                &ctx.accounts.player.key(),
                &ctx.accounts.vault.key(),
                REQUIRED_DEPOSIT,
            ),
            &[
                ctx.accounts.player.to_account_info(),
                ctx.accounts.vault.to_account_info(),
                ctx.accounts.system_program.to_account_info(),
            ],
        )?;

        // Update game session variables
        game_session.game_type = game_type;
        game_session.start_time = Clock::get()?.unix_timestamp;
        game_session.score = 0;
        game_session.deposit += REQUIRED_DEPOSIT;
        game_session.kol = kol;

        // Initialize game-type specific variables
        // match game_type {
        //     1 => {
        //         game_session.game_1_completed = false;
        //         game_session.game_1_score = 1000;
        //         game_session.game_1_guesses_count = 0;
        //     },
        //     2 => {
        //         game_session.target_index = (Clock::get()?.unix_timestamp % 10) as u8;
        //         game_session.game_2_completed = false;
        //         game_session.game_2_score = 1000;
        //         game_session.game_2_guesses_count = 0;
        //     },
        //     _ => unreachable!(),
        // }

                game_session.game_1_completed = false;
                game_session.game_1_score = 1000;
                game_session.game_1_guesses_count = 0;
                game_session.target_index = (Clock::get()?.unix_timestamp % 10) as u8;
                game_session.game_2_completed = false;
                game_session.game_2_score = 1000;
                game_session.game_2_guesses_count = 0;

        msg!("Game session started for game type: {}", game_type);
        msg!("Competition ID: {}", game_session.competition_id);

        Ok(())
    }

    pub fn submit_score(ctx: Context<SubmitGameScore>, game_type: u8, score: u32, guesses: u32) -> Result<()> {
        let game_session = &mut ctx.accounts.game_session;

        require!(!game_session.completed, SoddleError::GameAlreadyCompleted);
        require!(ctx.accounts.player.key() == game_session.player, SoddleError::InvalidPlayer);

        game_session.score = score;

        // Update the appropriate score based on game_type
        match game_type {
            1 => {
                game_session.game_1_guesses_count = guesses;
                game_session.game_1_score = score;
            },
            2 => {
                game_session.game_2_guesses_count = guesses;
                game_session.game_2_score = score;
            },
            _ => return Err(SoddleError::InvalidGameType.into()),
        }

        if game_session.game_1_completed || game_session.game_2_completed {
            // Reset to initial state
            game_session.player = Pubkey::default();

            msg!("Game session reset to initial state");
        }

        Ok(())
    }

}

#[derive(Accounts)]
pub struct InitializeGame<'info> {
    #[account(
        init_if_needed,
        payer = authority,
        space = 8 + GameState::INIT_SPACE,
        seeds = [b"game_state"],
        bump
    )]
    pub game_state: Account<'info, GameState>,
    #[account(mut)]
    pub authority: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct StartGameSession<'info> {
    #[account(mut)]
    pub game_state: Account<'info, GameState>,
    #[account(
        init_if_needed,
        payer = player,
        space = 8 + GameSession::INIT_SPACE,
        seeds = [b"game_session", player.key().as_ref(),game_state.current_competition.id.as_bytes()],
        bump
    )]
    pub game_session: Account<'info, GameSession>,
    #[account(mut)]
    pub player: Signer<'info>,
    #[account(
        mut,
        seeds = [b"vault"],
        bump
    )]
    /// CHECK: This is the vault account that will hold the deposits
    pub vault: AccountInfo<'info>,
    pub system_program: Program<'info, System>,
}


#[derive(Accounts)]
pub struct SubmitGameScore<'info> {
    #[account(mut)]
    pub game_session: Account<'info, GameSession>,
    /// CHECK: This is not dangerous because we don't read or write from this account
    pub player: AccountInfo<'info>,
    #[account(mut)]
    pub authority: Signer<'info>,
    pub system_program: Program<'info, System>,
}


#[account]
#[derive(InitSpace)]
pub struct GameState {
    pub current_competition: Competition,
    pub last_update_time: i64,
}
#[account]
#[derive(InitSpace)]
pub struct GameSession {
    pub player: Pubkey,
    pub game_type: u8,
    pub start_time: i64,
    pub game_1_completed: bool,
    pub game_2_completed: bool,
    pub game_1_score: u32,
    pub game_2_score: u32,
    pub game_1_guesses_count: u32,
    pub game_2_guesses_count: u32,
    pub total_score: u32,
    pub target_index: u8,
    pub completed: bool,
    pub score: u32,
    pub deposit: u64,
    pub kol: KOL,
    #[max_len(15)]
    pub competition_id: String,

}

#[derive(Accounts)]
pub struct EndGameSession<'info> {
    #[account(mut)]
    pub game_state: Account<'info, GameState>,
    #[account(mut, has_one = player)]
    pub game_session: Account<'info, GameSession>,
    #[account(mut)]
    pub player: Signer<'info>,
    #[account(mut)]
    /// CHECK: This is the vault account that holds the deposits
    pub vault: AccountInfo<'info>,
    pub system_program: Program<'info, System>,
}


#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, InitSpace)]
pub struct Competition {
    #[max_len(15)]
    pub id: String,
    pub start_time: i64,
    pub end_time: i64,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
#[derive(InitSpace)]
pub struct Game1GuessResult {
    #[max_len(15)]
    pub kol: String,
    #[max_len(7)]
    pub result: [AttributeResult; 7],
}

#[derive(InitSpace,AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct Game2GuessResult {
    #[max_len(15)]
    pub kol: String,
    pub result: bool,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Debug)]
pub enum AttributeResult {
    Correct,
    Incorrect,
    Higher,
    Lower,
}

impl Space for AttributeResult {
    const INIT_SPACE: usize = 1; // Enum variants are represented as u8
}

#[event]
pub struct TweetGuessEvent {
    pub kol_id: u32,
    pub tweet: String,
}
#[derive(AnchorSerialize, AnchorDeserialize, PartialEq, Clone, Debug, InitSpace)]
pub struct KOL {
    #[max_len(15)]
    pub id: String,  // Store the ObjectId as 12 bytes
    #[max_len(30)]
    pub name: String,
    pub age: u8,
    #[max_len(30)]
    pub country: String,
    #[max_len(100)]
    pub pfp: String,
    pub account_creation: u16,
    pub followers: u32,
    #[max_len(20)]
    pub ecosystem: String,
}

#[error_code]
pub enum SoddleError {
    #[msg("Game session cannot be ended yet")]
    GameSessionNotEnded,
    #[msg("Invalid competition")]
    InvalidCompetition,
    #[msg("Maximum number of guesses reached")]
    MaxGuessesReachedForGame1,
    #[msg("Maximum number of guesses reached")]
    MaxGuessesReachedForGame2,
    #[msg("Maximum number of guesses reached")]
    MaxGuessesReachedForGame3,
    #[msg("Invalid number of KOLs. Expected 20.")]
    InvalidKOLCount,
    #[msg("Invalid game type. Must be 1, 2, or 3.")]
    InvalidGameType,
    #[msg("Game has already been played today.")]
    GameAlreadyPlayed,
    #[msg("Game session is already completed.")]
    GameAlreadyCompleted,
    #[msg("Invalid guess index.")]
    InvalidGuessIndex,
    #[msg("Competition has not ended yet.")]
    CompetitionNotEnded,
    #[msg("Game is not completed yet.")]
    GameNotCompleted,
    #[msg("This player is invalid")]
    InvalidPlayer,
}
