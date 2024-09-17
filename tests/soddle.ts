import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { SoddleGame } from "../target/types/soddle_game";
import { expect } from "chai";
import { Keypair, PublicKey, SystemProgram } from "@solana/web3.js";

describe("soddle-game", () => {
    const provider = anchor.AnchorProvider.env();
    anchor.setProvider(provider);

    const program = anchor.workspace.SoddleGame as Program<SoddleGame>;
    const player = Keypair.generate();

    let gameStatePda: PublicKey;
    let gameSessionPda: PublicKey;
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
        [gameSessionPda] = PublicKey.findProgramAddressSync(
            [Buffer.from("game_session"), player.publicKey.toBuffer()],
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

    it("Starts a game session", async () => {
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
        expect(gameSession.gameType).to.equal(1);
        expect(gameSession.player.toString()).to.equal(player.publicKey.toString());
    });

    it("Submits a score", async () => {
        console.log(player.publicKey.toBase58(), provider.wallet.publicKey.toBase58())
        await program.methods
            .submitScore(500, 5)
            .accounts({
                gameSession: gameSessionPda,
                player: player.publicKey,
                authority: provider.wallet.publicKey,
                systemProgram: SystemProgram.programId,
            })
            .signers([player])
            .rpc();

        const gameSession = await program.account.gameSession.fetch(gameSessionPda);
        expect(gameSession.score).to.equal(500);
    });
});