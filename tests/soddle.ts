import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { SoddleGame } from "../target/types/soddle_game";
import { expect } from "chai";
import { Keypair, PublicKey, SystemProgram, LAMPORTS_PER_SOL } from "@solana/web3.js";
import { bs58 } from '@coral-xyz/anchor/dist/cjs/utils/bytes';
import * as buffer from "node:buffer";
const REQUIRED_DEPOSIT = (0.001 * LAMPORTS_PER_SOL)
describe("soddle-game",  () => {
    const provider = anchor.AnchorProvider.env();
    anchor.setProvider(provider);

    const program = anchor.workspace.SoddleGame as Program<SoddleGame>;
    const player = Keypair.fromSecretKey(bs58.decode("3vxNdn1ujj2HAs2vd8hEwrnMbnPUTtvyHhKkm9AYXsm3c97oevWmBWqvZzp48zWN5XAhFocFr3FQBszKCPedqC3N"))
    const PROGRAM_AUTHORITY = new PublicKey("6kexz7VwA5J895tdWaDP6b4S9okQez1Att6E2jzWLXMk");
    const SODDLE_WALLET = new PublicKey("Bq8t4M2n7eE1AU3AJvjWP6dawJbsALwPTx631Ld59JUF");
    const REWARD_DISTRIBUTION_VAULT = new PublicKey("Bq8t4M2n7eE1AU3AJvjWP6dawJbsALwPTx631Ld59JUF");

    let gameStatePda: PublicKey;
    let vaultPda: PublicKey;
    let authorityPda: PublicKey;

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

    it("Initializes the game", async () => {
        // @ts-ignore
        try{
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
        } catch (error){
            console.log(error)
            expect(error).to.have.property("code", "UnauthorizedAuthority");
        }
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
            pfpType:"human",
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
        expect(gameSession.score.toString()).to.equal('0');
        expect(gameSession.deposit.toNumber()).to.equal(REQUIRED_DEPOSIT *20);
    });
    //
    // it("Submits a score", async () => {
    //     const gameType = 1;
    //     const score = 800;
    //     const guesses = 3;
    //
    //     await program.methods.submitScore(gameType, score, guesses)
    //         .accounts({
    //             gameSession: gameSessionPda,
    //             player: player.publicKey,
    //             authority: provider.wallet.publicKey,
    //             systemProgram: SystemProgram.programId,
    //         })
    //         .rpc();
    //
    //     const gameSessionAccount = await program.account.gameSession.fetch(gameSessionPda);
    //     expect(gameSessionAccount.game1Score).to.equal(score);
    //     expect(gameSessionAccount.game1GuessesCount).to.equal(guesses);
    // });
    it("Distribute Funds", async () => {
        const initialVaultBalance = await provider.connection.getBalance(vaultPda);

        await program.methods
            .distributeFunds()
            .accounts({
                authority: provider.wallet.publicKey,
                vault: vaultPda,
                soddleVault: SODDLE_WALLET,
                rewardDistributionVault: REWARD_DISTRIBUTION_VAULT,
                systemProgram: SystemProgram.programId,
            })
            .signers([provider.wallet])
            .rpc();

        const finalVaultBalance = await provider.connection.getBalance(vaultPda);
        expect(finalVaultBalance).to.equal(0);

        const soddleVaultBalance = await provider.connection.getBalance(SODDLE_WALLET);
        expect(soddleVaultBalance).to.equal(initialVaultBalance * 0.025); // 2.5%
    });

    it("Should prevent unauthorized fund distribution", async () => {
        const unauthorized = Keypair.generate();

        try {
            await program.methods
                .distributeFunds()
                .accounts({
                    authority: unauthorized.publicKey,
                    vault: vaultPda,
                    soddleVault: SODDLE_WALLET,
                    rewardDistributionVault: REWARD_DISTRIBUTION_VAULT,
                    systemProgram: SystemProgram.programId,
                })
                .signers([unauthorized])
                .rpc();

            expect.fail("Should have thrown unauthorized error");
        } catch (error) {
            console.log(error)
            expect(error).to.have.property("code", "UnauthorizedAuthority");
        }
    });

    it("Should prevent playing same game type twice in one day", async () => {
        const player = Keypair.generate();
        const gameType = 1;

        const kol = {
            id: "KOL123",
            name: "Test KOL",
            age: 25,
            country: "Test Country",
            pfp: "https://example.com/pfp.jpg",
            pfpType:"human",
            accountCreation: 2020,
            followers: 10000,
            ecosystem: "Test Ecosystem",
        };

        try {
            // Try to start same game type again
            // @ts-ignore
            await program.methods
                .startGameSession(gameType, kol)
                .accounts({
                    gameState: gameStatePda,
                    gameSession: gameSessionPda,
                    player: player.publicKey,
                    vault: vaultPda,
                    systemProgram: SystemProgram.programId,
                })
                .signers([player])
                .rpc();

            expect.fail("Should have thrown already played error");
        } catch (error) {
            console.log(error)
            expect(error).to.have.property("code", "GameAlreadyPlayed");
        }
    });
});