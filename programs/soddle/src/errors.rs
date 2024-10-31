use anchor_lang::prelude::*;

#[error_code]
pub enum SoddleError {
    #[msg("Game session cannot be ended yet")]
    GameSessionNotEnded,
    #[msg("Invalid competition")]
    InvalidCompetition,
    #[msg("Invalid time range")]
    InvalidTimeRange,
    #[msg("Competition has started")]
    CompetitionStarted,
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
    #[msg("Unauthorized authority")]
    UnauthorizedAuthority,
    #[msg("Already played today")]
    AlreadyPlayedToday,
    #[msg("Math Overflow Distribution Error")]
    MathOverflow,
    #[msg("Cannot distribute funds from an empty vault")]
    EmptyVault,
    #[msg("Game session has already started")]
    GameSessionAlreadyStarted,
    #[msg("Numeric overflow occurred")]
    NumericOverflow,
    #[msg("Invalid withdraw amount")]
    InvalidWithdrawAmount,
    #[msg("Insufficient funds in fee vault")]
    InsufficientFunds,
    #[msg("Game session has expired")]
    SessionExpired,
    #[msg("Invalid player")]
    InvalidPlayer,
    #[msg("Invalid KOL account")]
    InvalidKol,
    #[msg("Game session not started")]
    GameNotStarted,
    #[msg("Attempt already completed")]
    AttemptAlreadyCompleted,
    #[msg("Invalid guess count")]
    InvalidGuessCount,
    #[msg("Numeric overflow occurred")]
    NumericOverflow,
}