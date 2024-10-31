use anchor_lang::prelude::*;
use anchor_lang::solana_program::hash::hash;

#[derive(Accounts)]
pub struct MakeGuess<'info> {
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
        has_one = player @ GameError::InvalidPlayer,
    )]
    pub game_session: Account<'info, GameSession>,

    #[account(
        seeds = [
            b"guess_verifier",
            game_session.key().as_ref(),
        ],
        bump = verifier.bump,
    )]
    pub verifier: Account<'info, GuessVerifier>,

    #[account(
        constraint = oracle_authority.key() == verifier.oracle_authority @ GameError::InvalidOracle
    )]
    pub oracle_authority: Signer<'info>,

    #[account(mut)]
    pub player: Signer<'info>,
}

pub fn make_guess(
    ctx: Context<MakeGuess>,
    guess: String,
    oracle_signature: [u8; 64],
) -> Result<()> {
    let game_session = &mut ctx.accounts.game_session;
    let current_time = Clock::get()?.unix_timestamp;

    // 1. Validate game session state
    let current_attempt_index = game_session.current_attempt
        .checked_sub(1)
        .ok_or(GameError::GameNotStarted)? as usize;

    let current_attempt = &mut game_session.attempts[current_attempt_index];
    
    require!(!current_attempt.completed, GameError::AttemptAlreadyCompleted);

    // 2. Validate attempt hasn't exceeded limits
    let elapsed_time = current_time
        .checked_sub(current_attempt.timestamp)
        .ok_or(GameError::InvalidTimestamp)?;

    require!(elapsed_time < ATTEMPT_TIME_LIMIT, GameError::AttemptTimedOut);
    require!(current_attempt.guesses < MAX_GUESSES, GameError::TooManyGuesses);

    // 3. Verify guess with oracle
    let message = [
        &hash(guess.as_bytes()).to_bytes(),
        &ctx.accounts.verifier.answer_hash
    ].concat();

    let is_valid_signature = verify_oracle_signature(
        &oracle_signature,
        &message,
        &ctx.accounts.oracle_authority.key()
    );
    require!(is_valid_signature, GameError::InvalidOracleSignature);

    let is_correct = hash(guess.as_bytes()).to_bytes() == ctx.accounts.verifier.answer_hash;

    // 4. Update attempt state
    current_attempt.guesses = current_attempt.guesses
        .checked_add(1)
        .ok_or(GameError::NumericOverflow)?;

    // 5. Calculate and update score using the existing method
    current_attempt.score = game_session.calculate_current_score(current_time, !is_correct);

    // Calculate these values for the event emission
    let elapsed_time = current_time - current_attempt.timestamp;
    let time_periods = elapsed_time / TIME_PENALTY_INTERVAL;
    let time_deduction = (time_periods as u32).saturating_mul(TIME_PENALTY);
    let wrong_guess_penalty = if !is_correct { WRONG_GUESS_PENALTY } else { 0 };

    // 6. Check if attempt should be completed
    let should_end = is_correct 
        || current_attempt.guesses >= MAX_GUESSES 
        || elapsed_time >= ATTEMPT_TIME_LIMIT;

    // 7. Emit guess event
    emit!(GuessAttempted {
        player: ctx.accounts.player.key(),
        attempt_index: current_attempt_index as u8,
        guess_number: current_attempt.guesses,
        current_score: current_attempt.score,
        was_correct: is_correct,
        timestamp: current_time,
        time_deduction,
        wrong_guess_penalty,
    });

    // 8. Handle attempt completion if needed
    if should_end {
        current_attempt.completed = true;
        
        emit!(AttemptCompleted {
            player: ctx.accounts.player.key(),
            attempt_index: current_attempt_index as u8,
            final_score: current_attempt.score,
            total_guesses: current_attempt.guesses,
            was_correct: is_correct,
            timestamp: current_time,
            reason: if is_correct {
                CompletionReason::CorrectGuess
            } else if current_attempt.guesses >= MAX_GUESSES {
                CompletionReason::MaxGuesses
            } else {
                CompletionReason::TimeExpired
            },
        });
    }

    Ok(())
}


