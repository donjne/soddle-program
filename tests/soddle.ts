import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { SoddleGame } from "../target/types/soddle_game";
import { expect } from "chai";
import { Keypair, PublicKey, SystemProgram, LAMPORTS_PER_SOL } from "@solana/web3.js";
import * as buffer from "node:buffer";
const REQUIRED_DEPOSIT = (0.001 * LAMPORTS_PER_SOL)
describe("soddle-game", () => {
    const provider = anchor.AnchorProvider.env();
    anchor.setProvider(provider);

    const program = anchor.workspace.SoddleGame as Program<SoddleGame>;
    const player = Keypair.generate();

    let gameStatePda: PublicKey;
    let vaultPda: PublicKey;
    let authorityPda: PublicKey;

    before(async () => {
        // Airdrop SOL to the player
        await provider.connection.requestAirdrop(player.publicKey, 1000000000);

        // Derive PDAs
        [gameStatePda] = PublicKey.findProgramAddressSync(
            [Buffer.from("game_state")],
            program.programId
        );

        [vaultPda] = PublicKey.findProgramAddressSync(
            [Buffer.from("vault")],
            program.programId
        );
         [authorityPda] = PublicKey.findProgramAddressSync(
            [Buffer.from("authority")],
            program.programId
        );
    });

    it("Initializes the game", async () => {
        // @ts-ignore
        await program.methods
            .initializeGame()
            .accounts({
                gameState: gameStatePda,
                authority: provider.wallet.publicKey,
                systemProgram: SystemProgram.programId,
            })
            .rpc();

        const gameState = await program.account.gameState.fetch(gameStatePda);
        expect(gameState.currentCompetition.id).to.include("COMP");
    });
let gameSessionPda: PublicKey
    it("Starts a game session", async () => {
        const gameState = await program.account.gameState.fetch(gameStatePda);
         [gameSessionPda] = PublicKey.findProgramAddressSync(
            [Buffer.from("game_session"), player.publicKey.toBuffer(), Buffer.from(gameState.currentCompetition.id)],
            program.programId
        );

        const kol = {
            id: "KOL123",
            name: "Test KOL",
            age: 25,
            country: "Test Country",
            pfp: "https://example.com/pfp.jpg",
            accountCreation: 2020,
            followers: 10000,
            ecosystem: "Test Ecosystem",
        };

        await program.methods
            .startGameSession(1, kol)
            .accounts({
                gameState: gameStatePda,
                gameSession: gameSessionPda,
                player: player.publicKey,
                vault: vaultPda,
                systemProgram: SystemProgram.programId,
            })
            .signers([player])
            .rpc();

        const gameSession = await program.account.gameSession.fetch(gameSessionPda);
        console.log(gameState, gameSession)
        expect(gameSession.gameType).to.equal(1);
        expect(gameSession.player.toString()).to.equal(player.publicKey.toString());
        expect(gameSession.player.toString()).to.equal(player.publicKey.toString());
        expect(gameSession.competitionId.toString()).to.equal(gameState.currentCompetition.id.toString());
        expect(gameSession.gameType).to.equal(1);
        expect(gameSession.completed).to.be.false;
        expect(gameSession.score.toString()).to.equal("1000");
        expect(gameSession.deposit.toNumber()).to.equal(REQUIRED_DEPOSIT);
    });

    it("Submits a score", async () => {
        const gameType = 1;
        const score = 800;
        const guesses = 3;

        await program.methods.submitScore(gameType, score, guesses)
            .accounts({
                gameSession: gameSessionPda,
                player: player.publicKey,
                authority: provider.wallet.publicKey,
                systemProgram: SystemProgram.programId,
            })
            .rpc();

        const gameSessionAccount = await program.account.gameSession.fetch(gameSessionPda);
        expect(gameSessionAccount.game1Score).to.equal(score);
        expect(gameSessionAccount.game1GuessesCount).to.equal(guesses);
    });
});