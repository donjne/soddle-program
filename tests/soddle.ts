import * as anchor from "@coral-xyz/anchor";
import {AnchorProvider, Program, Wallet} from "@coral-xyz/anchor";
import { SoddleGame } from "../target/types/soddle_game";
import { expect } from "chai";
import { Keypair, PublicKey, SystemProgram, LAMPORTS_PER_SOL, Connection } from "@solana/web3.js";
import { bs58 } from '@coral-xyz/anchor/dist/cjs/utils/bytes';

describe("soddle-game", () => {
    const connection = new Connection(
        'https://devnet.helius-rpc.com/?api-key=1d33d108-520d-4e5c-998e-548383eb6665',
    );

    const keypair = Keypair.fromSecretKey(
        bs58.decode("3vxNdn1ujj2HAs2vd8hEwrnMbnPUTtvyHhKkm9AYXsm3c97oevWmBWqvZzp48zWN5XAhFocFr3FQBszKCPedqC3N"),
    );
   const wallet = new Wallet(keypair);

    // Initialize Anchor provider
    const provider = new AnchorProvider(connection, wallet, {
        preflightCommitment: 'confirmed',
    });
    anchor.setProvider(provider);

    const program = anchor.workspace.SoddleGame as Program<SoddleGame>;
    const REQUIRED_DEPOSIT = 0.02 * LAMPORTS_PER_SOL;
    const player = Keypair.fromSecretKey(bs58.decode("EATP68qnKvrJjWSkZbwwNvNG9YRaRugudkHcED79ZMERsF9Rkk8WxgG4iofisgR9chZybxMeMyyYymVqem3brQA"));
    const SODDLE_WALLET = new PublicKey("Bq8t4M2n7eE1AU3AJvjWP6dawJbsALwPTx631Ld59JUF");
    const REWARD_DISTRIBUTION_VAULT = new PublicKey("Bq8t4M2n7eE1AU3AJvjWP6dawJbsALwPTx631Ld59JUF");

    let gameStatePda: PublicKey;
    let gameStateBump: number;
    let vaultPda: PublicKey;
    let vaultBump: number;
    let gameSessionPda: PublicKey;

    before(async () => {
        // Find PDAs with bumps
        [gameStatePda, gameStateBump] = PublicKey.findProgramAddressSync(
            [Buffer.from("game_state")],
            program.programId
        );

        [vaultPda, vaultBump] = PublicKey.findProgramAddressSync(
            [Buffer.from("vault")],
            program.programId
        );

        // Airdrop to player
        try {
            const airdropSig = await provider.connection.requestAirdrop(
                player.publicKey,
                2 * LAMPORTS_PER_SOL
            );
            await provider.connection.confirmTransaction(airdropSig);
        } catch (e) {
            console.log("Airdrop failed, but continuing...");
        }
    });

    // it("Initializes the game", async () => {
    //     console.log(gameStatePda.toBase58())
    //     try {
    //         await program.methods
    //             .initializeGame()
    //             .accounts({
    //                 gameState: gameStatePda,
    //                 authority: provider.wallet.publicKey,
    //                 systemProgram: SystemProgram.programId,
    //             })
    //             .rpc();
    //
    //         // Verify the game state was initialized
    //         const gameState = await program.account.gameState.fetch(gameStatePda);
    //         expect(gameState.currentCompetition.id).to.include("COMP");
    //     } catch (error: any) {
    //         if (error.error?.errorCode?.code === "UnauthorizedAuthority") {
    //             // Expected error since we're not the program authority
    //             console.log(error)
    //             return;
    //         }
    //         throw error;
    //     }
    // });

    it("Starts a game session", async () => {
        try {
            // First fetch game state
            const gameState = await program.account.gameState.fetch(gameStatePda);

            // Log the competition ID for debugging
            console.log("Competition ID:", gameState.currentCompetition.id);
            console.log("Player pubkey:", player.publicKey.toBase58());

            // Derive game session PDA with exact same seeds as program
            [gameSessionPda] = PublicKey.findProgramAddressSync(
                [
                    Buffer.from("game_session"),
                    player.publicKey.toBuffer(),
                    Buffer.from(gameState.currentCompetition.id)
                ],
                program.programId
            );

            const kol = {
                id: "KOL123",
                name: "Test KOL",
                age: 25,
                country: "Test Country",
                pfpType: "human",
                pfp: "https://example.com/pfp.jpg",
                accountCreation: 2020,
                followers: 10000,
                ecosystem: "Test Ecosystem",
            };

            // Add remaining lamports check
            const playerBalance = await provider.connection.getBalance(player.publicKey);
            console.log("Player balance:", playerBalance / LAMPORTS_PER_SOL, "SOL");

            // Start game session
            const tx = await program.methods
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

            // Wait for confirmation
            await provider.connection.confirmTransaction(tx);

            // Fetch and verify game session
            const gameSession = await program.account.gameSession.fetch(gameSessionPda);

            // Log full game session for debugging
            console.log("Game Session:", {
                player: gameSession.player.toBase58(),
                gameType: gameSession.gameType,
                competitionId: gameSession.competitionId,
                deposit: gameSession.deposit.toString(),
                completed: gameSession.completed
            });

            // Assertions
            expect(gameSession.player.toString()).to.equal(player.publicKey.toString());
            expect(gameSession.gameType).to.equal(1);
            expect(gameSession.competitionId).to.equal(gameState.currentCompetition.id);
            expect(gameSession.completed).to.be.false;
            expect(gameSession.deposit.toString()).to.equal(REQUIRED_DEPOSIT.toString());

        } catch (error: any) {
            console.error("Detailed error:", {
                errorCode: error.error?.errorCode,
                errorMessage: error.error?.errorMessage,
                logs: error.logs,
                comparedValues: error.error?.comparedValues
            });
            throw error;
        }
    });

//     it("Submit score", async () => {
//         try {
//             await program.methods
//                 .submitScore(1, 850, 3)
//                 .accounts({
//                     gameSession: gameSessionPda,
//                     player: player.publicKey,
//                     authority: provider.wallet.publicKey,
//                     systemProgram: SystemProgram.programId,
//                 })
//                 .signers([player])
//                 .rpc();
//
//             const gameSession = await program.account.gameSession.fetch(gameSessionPda);
//             expect(gameSession.game1Score).to.equal(850);
//             expect(gameSession.game1GuessesCount).to.equal(3);
//         } catch (error: any) {
//         console.error("Error submitting score:", error);
//         throw error;
//     }
// });

    it("Distribute Funds", async () => {
        try {
            // Get initial balance
            const initialVaultBalance = await provider.connection.getBalance(vaultPda);
            console.log("Initial vault balance:", initialVaultBalance);
            console.log(vaultPda.toBase58())

            // Execute distribute funds
            await program.methods
                .distributeFunds()
                .accounts({
                    authority: provider.wallet.publicKey,
                    vault: vaultPda,
                    soddleVault:SODDLE_WALLET,
                    rewardDistributionVault:REWARD_DISTRIBUTION_VAULT,
                    systemProgram: SystemProgram.programId,
                })
                .rpc();

            // Verify the distribution
            const finalVaultBalance = await provider.connection.getBalance(vaultPda);
            console.log("Final vault balance:", finalVaultBalance);

            // Check balances
            const soddleWalletBalance = await provider.connection.getBalance(new PublicKey(SODDLE_WALLET));
            const rewardVaultBalance = await provider.connection.getBalance(new PublicKey(REWARD_DISTRIBUTION_VAULT));

            console.log("Distribution results:", {
                soddleAmount: soddleWalletBalance,
                rewardAmount: rewardVaultBalance
            });

            // Vault should be empty after distribution
            expect(finalVaultBalance).to.equal(0);

        } catch (error: any) {
            if (error.error?.errorCode?.code === "UnauthorizedAuthority") {
                // Expected error since we're not the program authority
                console.log("Expected unauthorized authority error");
                return;
            }
            console.error("Distribution error:", {
                error: error.error,
                logs: error.logs,
                message: error.message
            });
            throw error;
        }
    });
});