import * as anchor from "@coral-xyz/anchor";
import {AnchorProvider, Program, Wallet} from "@coral-xyz/anchor";
import { SoddleGame } from "../target/types/soddle_game";
import { expect } from "chai";
import { Keypair, PublicKey, SystemProgram, LAMPORTS_PER_SOL, Connection } from "@solana/web3.js";
import { bs58 } from '@coral-xyz/anchor/dist/cjs/utils/bytes';

// Logger utility function
const log = (section: string, message: string, data?: any) => {
    console.log(`\n[${section}] ${message}`);
    if (data) {
        console.log(JSON.stringify(data, null, 2));
    }
};

describe("soddle-game", () => {
    log("Test Suite", "Starting Soddle Game test suite");

    const connection = new Connection(
        'https://devnet.helius-rpc.com/?api-key=1d33d108-520d-4e5c-998e-548383eb6665',
    );
    log("Connection", "Initialized Helius RPC connection");

    const keypair = Keypair.fromSecretKey(
        bs58.decode("3vxNdn1ujj2HAs2vd8hEwrnMbnPUTtvyHhKkm9AYXsm3c97oevWmBWqvZzp48zWN5XAhFocFr3FQBszKCPedqC3N"),
    );
    log("Setup", "Created keypair", { publicKey: keypair.publicKey.toBase58() });

    const wallet = new Wallet(keypair);
    const provider = new AnchorProvider(connection, wallet, {
        preflightCommitment: 'confirmed',
    });
    anchor.setProvider(provider);
    log("Setup", "Initialized Anchor provider");

    const program = anchor.workspace.SoddleGame as Program<SoddleGame>;
    log("Setup", "Loaded Soddle Game program", { programId: program.programId.toBase58() });

    const REQUIRED_DEPOSIT = 0.02 * LAMPORTS_PER_SOL;
    const player = Keypair.fromSecretKey(bs58.decode("EATP68qnKvrJjWSkZbwwNvNG9YRaRugudkHcED79ZMERsF9Rkk8WxgG4iofisgR9chZybxMeMyyYymVqem3brQA"));
    const SODDLE_WALLET = new PublicKey("Bq8t4M2n7eE1AU3AJvjWP6dawJbsALwPTx631Ld59JUF");
    const REWARD_DISTRIBUTION_VAULT = new PublicKey("Bq8t4M2n7eE1AU3AJvjWP6dawJbsALwPTx631Ld59JUF");

    log("Constants", "Initialized game constants", {
        requiredDeposit: REQUIRED_DEPOSIT / LAMPORTS_PER_SOL + " SOL",
        playerPublicKey: player.publicKey.toBase58(),
        soddleWallet: SODDLE_WALLET.toBase58(),
        rewardDistributionVault: REWARD_DISTRIBUTION_VAULT.toBase58()
    });

    let gameStatePda: PublicKey;
    let gameStateBump: number;
    let vaultPda: PublicKey;
    let vaultBump: number;
    let gameSessionPda: PublicKey;

    before(async () => {
        log("PDA Setup", "Finding program PDAs");

        [gameStatePda, gameStateBump] = PublicKey.findProgramAddressSync(
            [Buffer.from("game_state")],
            program.programId
        );
        log("PDA Setup", "Found game state PDA", {
            address: gameStatePda.toBase58(),
            bump: gameStateBump
        });

        [vaultPda, vaultBump] = PublicKey.findProgramAddressSync(
            [Buffer.from("vault")],
            program.programId
        );
        log("PDA Setup", "Found vault PDA", {
            address: vaultPda.toBase58(),
            bump: vaultBump
        });

        log("Airdrop", "Requesting airdrop for player");
        try {
            const airdropSig = await provider.connection.requestAirdrop(
                player.publicKey,
                2 * LAMPORTS_PER_SOL
            );
            await provider.connection.confirmTransaction(airdropSig);
            const balance = await provider.connection.getBalance(player.publicKey);
            log("Airdrop", "Airdrop successful", {
                signature: airdropSig,
                newBalance: balance / LAMPORTS_PER_SOL + " SOL"
            });
        } catch (e) {
            log("Airdrop", "Airdrop failed, continuing anyway", { error: e });
        }
    });
    it("Initializes the game", async () => {
        console.log(gameStatePda.toBase58())
        try {
            await program.methods
                .initializeGame()
                .accounts({
                    gameState: gameStatePda,
                    authority: provider.wallet.publicKey,
                    systemProgram: SystemProgram.programId,
                })
                .rpc();

            // Verify the game state was initialized
            const gameState = await program.account.gameState.fetch(gameStatePda);
            console.log(gameState)
            expect(gameState.currentCompetition.id).to.include("COMP");
        } catch (error: any) {
            if (error.error?.errorCode?.code === "UnauthorizedAuthority") {
                // Expected error since we're not the program authority
                console.log(error)
                return;
            }
            throw error;
        }
    });

    it("Starts a game session", async () => {
        log("Game Session", "Starting new game session test");

            const gameState = await program.account.gameState.fetch(gameStatePda);
        try {

            log("Game Session", "Fetched game state", {
                competitionId: gameState.currentCompetition.id,
                playerPubkey: player.publicKey.toBase58()
            });

            [gameSessionPda] = PublicKey.findProgramAddressSync(
                [
                    Buffer.from('game_session'),
                    player.publicKey.toBuffer(),
                    Buffer.from(gameState.currentCompetition.id),
                ],
                program.programId,
            );

            log("Game Session", "PDA Derivation Details", {
                seeds: {
                    prefix: Buffer.from("game_session").toString('hex'),
                    player: player.publicKey.toBuffer().toString('hex'),
                    competitionId: gameState.currentCompetition.id
                },
                derivedPda: gameSessionPda.toBase58(),
                programId: program.programId.toBase58()
            });

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
            log("Game Session", "Created KOL data", kol);

            const playerBalance = await provider.connection.getBalance(player.publicKey);
            log("Game Session", "Checked player balance", {
                balance: playerBalance / LAMPORTS_PER_SOL + " SOL"
            });

            log("Game Session", "Sending start game session transaction");
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

            log("Game Session", "Transaction sent, awaiting confirmation", { signature: tx });
            await provider.connection.confirmTransaction(tx);
            log("Game Session", "Transaction confirmed");

            const gameSession = await program.account.gameSession.fetch(gameSessionPda);
            log("Game Session", "Fetched game session data", {
                player: gameSession.player.toBase58(),
                gameType: gameSession.gameType,
                competitionId: gameSession.competitionId,
                deposit: gameSession.deposit.toString(),
                completed: gameSession.completed
            });

            // Assertions with logging
            log("Game Session", "Running assertions");
            expect(gameSession.player.toString()).to.equal(player.publicKey.toString());
            expect(gameSession.gameType).to.equal(1);
            expect(gameSession.competitionId).to.equal(gameState.currentCompetition.id);
            expect(gameSession.completed).to.be.false;
            expect(gameSession.deposit.toString()).to.equal(REQUIRED_DEPOSIT.toString());
            log("Game Session", "All assertions passed");

        } catch (error: any) {
            log("Game Session Error", "Failed to start game session", {
                errorCode: error.error?.errorCode,
                errorMessage: error.error?.errorMessage,
                logs: error.logs,
                comparedValues: error.error?.comparedValues,
                seeds: {
                    prefix: Buffer.from("game_session").toString('hex'),
                    player: player.publicKey.toBuffer().toString('hex'),
                    competitionId: Buffer.from(gameState.currentCompetition.id, 'utf8').toString('hex')
                }
            });
            throw error;
        }
    });

    // it("Distribute Funds", async () => {
    //     log("Distribution", "Starting funds distribution test");
    //
    //     try {
    //         const initialVaultBalance = await provider.connection.getBalance(vaultPda);
    //         log("Distribution", "Initial vault state", {
    //             balance: initialVaultBalance / LAMPORTS_PER_SOL + " SOL",
    //             address: vaultPda.toBase58()
    //         });
    //
    //         log("Distribution", "Executing distribute funds transaction");
    //         await program.methods
    //             .distributeFunds()
    //             .accounts({
    //                 authority: provider.wallet.publicKey,
    //                 vault: vaultPda,
    //                 systemProgram: SystemProgram.programId,
    //             })
    //             .rpc();
    //
    //         const finalVaultBalance = await provider.connection.getBalance(vaultPda);
    //         const soddleWalletBalance = await provider.connection.getBalance(SODDLE_WALLET);
    //         const rewardVaultBalance = await provider.connection.getBalance(REWARD_DISTRIBUTION_VAULT);
    //
    //         log("Distribution", "Distribution complete", {
    //             finalVaultBalance: finalVaultBalance / LAMPORTS_PER_SOL + " SOL",
    //             soddleWalletBalance: soddleWalletBalance / LAMPORTS_PER_SOL + " SOL",
    //             rewardVaultBalance: rewardVaultBalance / LAMPORTS_PER_SOL + " SOL"
    //         });
    //
    //         expect(finalVaultBalance).to.equal(0);
    //         log("Distribution", "Assertion passed: vault balance is zero");
    //
    //     } catch (error: any) {
    //         if (error.error?.errorCode?.code === "UnauthorizedAuthority") {
    //             log("Distribution", "Expected unauthorized authority error");
    //             return;
    //         }
    //         log("Distribution Error", "Failed to distribute funds", {
    //             error: error.error,
    //             logs: error.logs,
    //             message: error.message
    //         });
    //         throw error;
    //     }
    // });
});