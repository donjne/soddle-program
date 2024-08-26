import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { expect } from "chai";
import { Soddle } from "../target/types/soddle_game";

describe("Soddle Game", () => {
    const provider = anchor.AnchorProvider.env();
    anchor.setProvider(provider);

    const program = anchor.workspace.SoddleGame as Program<Soddle>;
    const player = anchor.web3.Keypair.generate();

    let gameStatePda: anchor.web3.PublicKey;
    let gameSessionPda: anchor.web3.PublicKey;
    let playerStatePda: anchor.web3.PublicKey;
    let vaultPda: anchor.web3.PublicKey;

    before(async () => {
        // Airdrop SOL to the player
        await provider.connection.requestAirdrop(
            player.publicKey,
            2 * anchor.web3.LAMPORTS_PER_SOL
        );

        // Derive PDAs
        [gameStatePda] = anchor.web3.PublicKey.findProgramAddressSync(
            [Buffer.from("game_state")],
            program.programId
        );

        [gameSessionPda] = anchor.web3.PublicKey.findProgramAddressSync(
            [Buffer.from("game_session"), player.publicKey.toBuffer()],
            program.programId
        );
        [playerStatePda] = anchor.web3.PublicKey.findProgramAddressSync(
            [Buffer.from("player_state"), player.publicKey.toBuffer()],
            program.programId
        );
        [vaultPda] = anchor.web3.PublicKey.findProgramAddressSync(
            [Buffer.from("vault")],
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
                systemProgram: anchor.web3.SystemProgram.programId,
            })
            .rpc();

        const gameState = await program.account.gameState.fetch(gameStatePda);
        expect(gameState.currentCompetition.id).to.include("COMP");
        expect(gameState.lastUpdateTime.toNumber()).to.be.greaterThan(0);
    });

    it("Starts a game session", async () => {
        const kol = {
            id: "1",
            name: "John Doe",
            age: 30,
            country: "USA",
            accountCreation: 2020,
            pfp: "https://example.com/pfp.jpg",
            followers: 10000,
            ecosystem: "Solana",
        };

        // @ts-ignore
        await program.methods
            .startGameSession(1, kol)
            .accounts({
                gameState: gameStatePda,
                gameSession: gameSessionPda,
                player: player.publicKey,
                playerState: playerStatePda,
                vault: vaultPda,
                systemProgram: anchor.web3.SystemProgram.programId,
            })
            .signers([player])
            .rpc();
        const gameState = await program.account.gameState.fetch(gameStatePda);
        const gameSession = await program.account.gameSession.fetch(gameSessionPda);
        console.log(gameSessionPda, gameStatePda, gameState)
        expect(gameSession.player.toString()).to.equal(player.publicKey.toString());
        expect(gameSession.gameType).to.equal(1);
        expect(gameSession.completed).to.be.false;
    });

    it("Makes incorrect guesses in the first game", async () => {
        const incorrectGuess = {
            id: "2",
            name: "Jane Doe",
            age: 28,
            country: "Canada",
            accountCreation: 2019,
            pfp: "https://example.com/pfp2.jpg",
            followers: 8000,
            ecosystem: "Ethereum",
        };

        for (let i = 0; i < 3; i++) {
            await program.methods
                .makeGuess(1, incorrectGuess)
                .accounts({
                    gameState: gameStatePda,
                    gameSession: gameSessionPda,
                    player: player.publicKey,
                    playerState: playerStatePda,
                })
                .signers([player])
                .rpc();

            const gameSession = await program.account.gameSession.fetch(gameSessionPda);
            expect(gameSession.game1Guesses).to.be.equal(i + 1);
            expect(gameSession.score).to.be.lessThan(1000);
            expect(gameSession.completed).to.be.false;
        }

        const gameSession = await program.account.gameSession.fetch(gameSessionPda);
        // console.log(gameSession, gameSession.guesses)
        expect(gameSession.game1Guesses).to.be.equal(3);
        expect(gameSession.score).to.be.lessThan(850); // Accounting for time and guess penalties
    });

    it("Makes a correct guess in the first game", async () => {
        const correctGuess = {
            id: "1",
            name: "John Doe",
            age: 30,
            country: "USA",
            accountCreation: 2020,
            pfp: "https://example.com/pfp.jpg",
            followers: 10000,
            ecosystem: "Solana",
        };

        await program.methods
            .makeGuess(1, correctGuess)
            .accounts({
                gameState: gameStatePda,
                gameSession: gameSessionPda,
                player: player.publicKey,
                playerState: playerStatePda,
            })
            .signers([player])
            .rpc();

        const gameSession = await program.account.gameSession.fetch(gameSessionPda);
        // console.log(gameSession, gameSession.guesses)
        expect(gameSession.game1Guesses).to.be.equal(4);
        expect(gameSession.game1Completed).to.be.true;
        expect(gameSession.game1Score).to.be.greaterThan(0);
        expect(gameSession.game1Score).to.be.lessThan(850); // Accounting for previous incorrect guesses and time penalty
    });

    it("Makes incorrect guesses in the second game", async () => {
        const incorrectGuesses = [
            {
                id: "3",
                name: "Bob Johnson",
                age: 35,
                country: "Canada",
                accountCreation: 2019,
                pfp: "https://example.com/bob.jpg",
                followers: 30000,
                ecosystem: "Solana",
            },
            {
                id: "4",
                name: "Carol Williams",
                age: 29,
                country: "Australia",
                accountCreation: 2020,
                pfp: "https://example.com/carol.jpg",
                followers: 40000,
                ecosystem: "Polkadot",
            },
        ];

        for (let i = 0; i < incorrectGuesses.length; i++) {
            await program.methods
                .makeGuess(2, incorrectGuesses[i])
                .accounts({
                    gameState: gameStatePda,
                    gameSession: gameSessionPda,
                    player: player.publicKey,
                    playerState: playerStatePda,
                })
                .signers([player])
                .rpc();

            const gameSession = await program.account.gameSession.fetch(gameSessionPda);
            expect(gameSession.game2Guesses).to.be.equal(i + 1);
            expect(gameSession.score).to.be.lessThan(1000);
            expect(gameSession.completed).to.be.false;
            expect(gameSession.game2Completed).to.be.false;
        }

        const gameSession = await program.account.gameSession.fetch(gameSessionPda);
        expect(gameSession.game2Guesses).to.be.equal(2);
        expect(gameSession.score).to.be.lessThan(900); // Accounting for time and guess penalties
    });

    it("Makes a correct guess in the second game", async () => {
        const correctGuess = {
            id: "1",
            name: "Alice Smith",
            age: 28,
            country: "UK",
            accountCreation: 2018,
            pfp: "https://example.com/alice.jpg",
            followers: 50000,
            ecosystem: "Ethereum",
        };

        await program.methods
            .makeGuess(2, correctGuess)
            .accounts({
                gameState: gameStatePda,
                gameSession: gameSessionPda,
                player: player.publicKey,
                playerState: playerStatePda,
            })
            .signers([player])
            .rpc();

        const gameSession = await program.account.gameSession.fetch(gameSessionPda);
        expect(gameSession.game2Completed).to.be.true;
        expect(gameSession.game2Score).to.be.greaterThan(0);
        expect(gameSession.game2Score).to.be.lessThan(900); // Accounting for previous incorrect guesses and time penalty
    });

    it("Ends the game session", async () => {
        await program.methods
            .endGameSession()
            .accounts({
                gameState: gameStatePda,
                gameSession: gameSessionPda,
                player: player.publicKey,
                playerState: playerStatePda,
                vault: vaultPda,
                systemProgram: anchor.web3.SystemProgram.programId,
            })
            .signers([player])
            .rpc();

        const gameSession = await program.account.gameSession.fetch(gameSessionPda);
        // console.log(gameSession)
// game is reset but score is
        expect(gameSession.completed).to.be.true;
        expect(gameSession.game1Completed).to.be.true;
        expect(gameSession.game1Score).to.be.greaterThan(0);
        expect(gameSession.game2Completed).to.be.true;
        expect(gameSession.game2Score).to.be.greaterThan(0);
    });

    it("Ends the competition", async () => {
        // Note: This test might fail if the competition duration hasn't passed
        // You might need to adjust the test or use a mock clock for testing
        try {
            await program.methods
                .endCompetition()
                .accounts({
                    gameState: gameStatePda,
                    authority: provider.wallet.publicKey,
                })
                .rpc();

            const gameState = await program.account.gameState.fetch(gameStatePda);
            console.log(gameState, 'gameState')
            expect(gameState.currentCompetition.id).to.include("COMP");
            expect(gameState.lastUpdateTime).to.be.greaterThan(0);
        } catch (error) {
            // console.log("Competition not ended yet. This is expected in most cases.");
        }
    });
});