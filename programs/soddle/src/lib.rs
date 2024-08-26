use anchor_lang::prelude::*;
use anchor_lang::solana_program::native_token::LAMPORTS_PER_SOL;

declare_id!("9b4J9BTbBnKfrfZ7YC1WpArk1z9DgQuTEhWJtoBshz7w");

const REQUIRED_DEPOSIT: u64 = (0.001 * LAMPORTS_PER_SOL as f64) as u64;
const COMPETITION_DURATION: i64 = 24 * 60 * 60; // 24 hours in seconds
const MAX_GUESSES: usize = 5;

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

        game_session.completed = false;
        game_session.competition_id = ctx.accounts.game_state.current_competition.id.clone();
        msg!(&game_session.competition_id);
        msg!(&ctx.accounts.game_state.current_competition.id.clone());


        // Check if the player already has a game session for today's competition
    //     require!(
    //     game_session.competition_id != ctx.accounts.game_state.current_competition.id,
    //     SoddleError::GameAlreadyPlayed
    // );

        let game_session = &mut ctx.accounts.game_session;
        let player_state = &mut ctx.accounts.player_state;
        // let vault = TokenAccount::try_from(&mut ctx.accounts.vault)?;

        match game_type {
            1 => require!(
                !player_state.game_1_completed,
                SoddleError::GameAlreadyPlayed
            ),
            2 => require!(
                !player_state.game_2_completed,
                SoddleError::GameAlreadyPlayed
            ),
            3 => require!(
                !player_state.game_3_completed,
                SoddleError::GameAlreadyPlayed
            ),
            _ => unreachable!(),
        }

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

        game_session.player = ctx.accounts.player.key();
        game_session.competition_id = ctx.accounts.game_state.current_competition.id.clone();
        game_session.game_type = game_type;
        game_session.start_time = Clock::get()?.unix_timestamp;
        game_session.target_index = (Clock::get()?.unix_timestamp % 10) as u8;
        game_session.guesses = Vec::new();
        game_session.game_1_completed = false;
        game_session.game_2_completed = false;
        game_session.game_3_completed = false;
        game_session.game_1_score = 0;
        game_session.game_2_score = 0;
        game_session.game_3_score = 0;
        game_session.completed = false;
        game_session.score = 1000;
        game_session.kol = kol;
        game_session.deposit = REQUIRED_DEPOSIT;

        Ok(())
    }

    pub fn make_guess(ctx: Context<MakeGuess>, game_type: u8, guess: KOL) -> Result<()> {
        let game_session = &mut ctx.accounts.game_session;
        let current_time = Clock::get()?.unix_timestamp;

        // Check if the user has already guessed the max guesses
        require!(
        game_session.game_1_guesses < MAX_GUESSES as u32,
        SoddleError::MaxGuessesReachedForGame1
    );
        require!(
        game_session.game_2_guesses < MAX_GUESSES as u32,
        SoddleError::MaxGuessesReachedForGame2
    );
        require!(
        game_session.game_3_guesses < MAX_GUESSES as u32,
        SoddleError::MaxGuessesReachedForGame3
    );

        // Check if the whole game has been completed
        require!(!game_session.completed, SoddleError::GameAlreadyCompleted);

        match game_type {
            1 => require!(!game_session.game_1_completed, SoddleError::GameAlreadyPlayed),
            2 => require!(!game_session.game_2_completed, SoddleError::GameAlreadyPlayed),
            3 => require!(!game_session.game_3_completed, SoddleError::GameAlreadyPlayed),
            _ => return Err(SoddleError::InvalidGameType.into()),
        }

        match game_type {
            1 => {
                // Logic for game type 1 (KOL attribute guessing)
                let result = evaluate_guess(&game_session.kol, &guess);
                game_session.guesses.push(GuessResult {
                    kol: guess,
                    result: result.clone(),
                });

                // Update guess for game 1
                game_session.game_1_guesses += 1;

                // Update score
                let time_penalty = ((current_time - game_session.start_time) / 60) as u32 * 10;
                let guess_penalty = game_session.guesses.len() as u32 * 50;
                game_session.score = game_session
                    .score
                    .saturating_sub(time_penalty + guess_penalty);

                // Check if guess is correct
                if result.iter().all(|r| *r == AttributeResult::Correct) {
                    game_session.game_1_completed = true;
                    game_session.game_1_score = game_session.score;
                }
            }
            2 => {
                // Logic for game type 2 (tweet guessing)
                // Update guess for game 2
                game_session.game_2_guesses += 1;
                if guess.id == game_session.kol.id {
                    game_session.game_2_completed = true;
                    game_session.game_2_score = game_session.score;
                } else {
                    // Update score
                    let time_penalty = ((current_time - game_session.start_time) / 60) as u32 * 10;
                    let guess_penalty = game_session.guesses.len() as u32 * 50;
                    game_session.score = game_session
                        .score
                        .saturating_sub(time_penalty + guess_penalty);
                    game_session.guesses.push(GuessResult {
                        kol: guess,
                        result: <[AttributeResult; 7]>::try_from(vec![AttributeResult::Correct; 7])
                            .unwrap(),
                    });
                }
            }
            3 => {
                // Logic for game type 3 (existing logic)
                // Update guess for game 3
                game_session.game_3_guesses += 1;
                if guess.id == game_session.kol.id {
                    game_session.game_3_completed = true;
                    game_session.game_3_score = game_session.score;
                } else {
                    game_session.score = game_session.score.saturating_sub(50);
                }
            }
            _ => return Err(SoddleError::InvalidGameType.into()),
        }

        Ok(())
    }

    pub fn end_game_session(ctx: Context<EndGameSession>) -> Result<()> {
        let game_session = &mut ctx.accounts.game_session;
        let player_state = &mut ctx.accounts.player_state;

        // Ensure the game session belongs to the current competition
        // require!(
        // game_session.competition_id == game_state.current_competition.id,
        // SoddleError::InvalidCompetition
    // );

        // Update player state based on the completed game
        match game_session.game_type {
            1 => {
                player_state.game_1_completed = true;
                player_state.game_1_score = game_session.score;
            },
            2 => {
                player_state.game_2_completed = true;
                player_state.game_2_score = game_session.score;
            },
            3 => {
                player_state.game_3_completed = true;
                player_state.game_3_score = game_session.score;
            },
            _ => return Err(SoddleError::InvalidGameType.into()),
        }

        // Calculate total score
        let total_score = player_state.game_1_score + player_state.game_2_score + player_state.game_3_score;

        // TODO: Implement reward distribution logic here
        // For example, you could transfer a portion of the deposit back to the player based on their score

        // Mark the game session as ended
        game_session.total_score = total_score;
        game_session.completed = true;

        player_state.game_1_completed = false;
        player_state.game_2_completed = false;
        player_state.game_3_completed = false;
        player_state.game_1_score = 0;
        player_state.game_2_score = 0;
        player_state.game_3_score = 0;

        Ok(())
    }

    pub fn end_competition(ctx: Context<EndCompetition>) -> Result<()> {
        let game_state = &mut ctx.accounts.game_state;

        let clock = Clock::get()?;
        let current_time: i64 = clock.unix_timestamp;

        require!(
            current_time >= game_state.current_competition.end_time,
            SoddleError::CompetitionNotEnded
        );


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
        init,
        payer = player,
        space = 8 + GameSession::INIT_SPACE,
        seeds = [b"game_session", player.key().as_ref()],
        bump
    )]
    pub game_session: Account<'info, GameSession>,
    #[account(mut)]
    pub player: Signer<'info>,
    #[account(
        init,
        payer = player,
        space = 8 + Player::INIT_SPACE,
        seeds = [b"player_state", player.key().as_ref()],
        bump
    )]
    pub player_state: Account<'info, Player>,
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
pub struct MakeGuess<'info> {
    pub game_state: Account<'info, GameState>,
    #[account(mut, has_one = player)]
    pub game_session: Account<'info, GameSession>,
    pub player: Signer<'info>,
    #[account(mut)]
    pub player_state: Account<'info, Player>,
}

#[derive(Accounts)]
pub struct EndCompetition<'info> {
    #[account(mut)]
    pub game_state: Account<'info, GameState>,
    #[account(
        seeds = [b"authority"],
        bump,
    )]
    pub authority: Signer<'info>,
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
    pub game_3_completed: bool,
    pub game_1_score: u32,
    pub game_2_score: u32,
    pub game_3_score: u32,
    pub game_1_guesses: u32,
    pub game_2_guesses: u32,
    pub game_3_guesses: u32,
    pub total_score: u32,
    pub target_index: u8,
    #[max_len(MAX_GUESSES)]
    pub guesses: Vec<GuessResult>,
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
    pub player_state: Account<'info, Player>,
    #[account(mut)]
    /// CHECK: This is the vault account that holds the deposits
    pub vault: AccountInfo<'info>,
    pub system_program: Program<'info, System>,
}

#[account]
#[derive(InitSpace)]
pub struct Player {
    pub game_1_completed: bool,
    pub game_2_completed: bool,
    pub game_3_completed: bool,
    pub game_1_score: u32,
    pub game_2_score: u32,
    pub game_3_score: u32,
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
pub struct GuessResult {
    pub kol: KOL,
    #[max_len(7)]
    pub result: [AttributeResult; 7],
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Debug)]
pub enum AttributeResult {
    Correct,
    Incorrect,
    Higher,
    Lower,
}

impl anchor_lang::Space for AttributeResult {
    const INIT_SPACE: usize = 1; // Enum variants are represented as u8
}

#[event]
pub struct TweetGuessEvent {
    pub kol_id: u32,
    pub tweet: String,
}
#[derive(AnchorSerialize, AnchorDeserialize, PartialEq, Clone, Debug, InitSpace)]
pub struct KOL {
    #[max_len(50)]
    pub id: String,
    #[max_len(100)]
    pub name: String,
    pub age: u8,
    #[max_len(50)]
    pub country: String,
    pub account_creation: u16,
    #[max_len(200)]
    pub pfp: String,
    pub followers: u32,
    #[max_len(50)]
    pub ecosystem: String,
}



pub fn derive_authority_pda(program_id: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[b"authority"], program_id)
}

fn evaluate_guess(actual: &KOL, guess: &KOL) -> [AttributeResult; 7] {
    [
        if actual.name == guess.name {
            AttributeResult::Correct
        } else {
            AttributeResult::Incorrect
        },
        if actual.age == guess.age {
            AttributeResult::Correct
        } else if actual.age > guess.age {
            AttributeResult::Higher
        } else {
            AttributeResult::Lower
        },
        if actual.country == guess.country {
            AttributeResult::Correct
        } else {
            AttributeResult::Incorrect
        },
        if actual.pfp == guess.pfp {
            AttributeResult::Correct
        } else {
            AttributeResult::Incorrect
        },
        if actual.account_creation == guess.account_creation {
            AttributeResult::Correct
        } else if actual.account_creation > guess.account_creation {
            AttributeResult::Higher
        } else {
            AttributeResult::Lower
        },
        if actual.followers == guess.followers {
            AttributeResult::Correct
        } else if actual.followers > guess.followers {
            AttributeResult::Higher
        } else {
            AttributeResult::Lower
        },
        if actual.ecosystem == guess.ecosystem {
            AttributeResult::Correct
        } else {
            AttributeResult::Incorrect
        },
    ]
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
}
