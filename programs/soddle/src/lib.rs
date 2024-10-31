use anchor_lang::prelude::*;

declare_id!("2y7L42gHKEBAFprVLJ9XFQuzxLdT9dmptdgQsNdcJ4SP");

#[program]
pub mod soddle_game {
    use super::*;

    pub fn end_game_session(ctx: Context<EndGameSession>) -> Result<()> {

    }

    pub fn initialize_competition(
        ctx: Context<InitializeCompetition>,
        id: u64,
        start_time: i64,
        end_time: i64,
    ) -> Result<()> {
    }

    pub fn initialize_metrics(
        ctx: Context<InitializeGameMetrics>
    ) -> Result<()> {
    }

    pub fn initialize_game_session(
        ctx: Context<InitializeGameSession>,
        competition_id: u64,
        start_time: i64,
    ) -> Result<()> {
    }


    pub fn initialize_game_state(ctx: Context<InitializeGameState>) -> Result<()> {

    }

    pub fn initialize_verifier(
        ctx: Context<InitializeVerifier>, 
        answer_hash: [u8; 32]
    ) -> Result<()> {
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
    }

    pub fn make_guess(
        ctx: Context<MakeGuess>,
        guess: String,
        oracle_signature: [u8; 64],
    ) -> Result<()> {
    }

    pub fn start_game_session(
        ctx: Context<StartGameSession>,
        competition_id: u64,
        start_time: i64,
    ) -> Result<()> {
    }

    pub fn withdraw_fees(
        ctx: Context<WithdrawFees>, 
        amount: u64
    ) -> Result<()> {
    }
}



