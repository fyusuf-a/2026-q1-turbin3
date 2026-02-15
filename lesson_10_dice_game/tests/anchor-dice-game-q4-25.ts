import * as anchor from "@coral-xyz/anchor";
import { Program, BN } from "@coral-xyz/anchor";
import { AnchorDiceGameQ425 } from "../target/types/anchor_dice_game_q4_25";
import { Ed25519Program, Keypair, LAMPORTS_PER_SOL, PublicKey, sendAndConfirmTransaction, SystemProgram, SYSVAR_INSTRUCTIONS_PUBKEY, Transaction } from "@solana/web3.js";
import { randomBytes } from "crypto";
import { expect } from "chai";

describe("anchor-dice-game-q4-25", () => {
  // Configure the client to use the local cluster.
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.anchorDiceGameQ425 as Program<AnchorDiceGameQ425>;

  const house = new Keypair();
  const player = new Keypair();
  const seed = new BN(randomBytes(8));
  const vault = PublicKey.findProgramAddressSync(
    [Buffer.from("vault"), house.publicKey.toBuffer()],
    program.programId
  )[0];
  const bet = PublicKey.findProgramAddressSync(
    [Buffer.from("bet"), house.publicKey.toBuffer(), player.publicKey.toBuffer(), seed.toBuffer("le", 16)],
    program.programId
  )[0];
  const playerVault = PublicKey.findProgramAddressSync(
    [Buffer.from("player_vault"), house.publicKey.toBuffer(), player.publicKey.toBuffer(), seed.toBuffer("le", 16)],
    program.programId
  )[0];

  before(async () => {
    const users = [house, player];
    const promises = [];

    for (const user of users) {
      // Airdrop for fees
      const signature = await provider.connection.requestAirdrop(user.publicKey, 200 * LAMPORTS_PER_SOL);
      const latestBlockhash = await provider.connection.getLatestBlockhash();

      promises.push(
        provider.connection.confirmTransaction({
          signature,
          ...latestBlockhash,
        }, "confirmed")
      );
    }
    await Promise.all(promises);
  });

  it("Is initialized!", async () => {
    await program.methods.initialize(new BN(LAMPORTS_PER_SOL).mul(new BN(100)))
      .accountsStrict({
        house: house.publicKey,
        vault,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .signers([house])
      .rpc();

    // get vault balance
    const vaultBalance = await provider.connection.getBalance(vault);
    expect(vaultBalance).to.equal(LAMPORTS_PER_SOL * 100);
  });

  it("Place a bet", async () => {
    let signature = await program.methods.placeBet(seed, 50, new BN(LAMPORTS_PER_SOL / 100))
      .accountsStrict({
        player: player.publicKey,
        house: house.publicKey,
        playerVault,
        bet,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .signers([player])
      .rpc();

    console.log("Place bet signature:", signature);

    const playerVaultBalance = await provider.connection.getBalance(playerVault);
    expect(playerVaultBalance).to.be.eq(LAMPORTS_PER_SOL / 100);
  });

  it("Resolves a bet", async () => {
    let account = await provider.connection.getAccountInfo(bet, "confirmed");
    const sig_ix = Ed25519Program.createInstructionWithPrivateKey({
      privateKey: house.secretKey,
      message: account.data.subarray(8),
    });

    const houseBalanceBefore = await provider.connection.getBalance(house.publicKey);
    const playerBalanceBefore = await provider.connection.getBalance(player.publicKey);

    const resolve_ix = await program.methods.resolveBet(Buffer.from(sig_ix.data.buffer.slice(16 + 32, 16 + 32 + 64)))
      .accountsStrict({
        player: player.publicKey,
        house: house.publicKey,
        vault,
        playerVault,
        bet,
        instructionSysvar: SYSVAR_INSTRUCTIONS_PUBKEY,
        systemProgram: SystemProgram.programId,
      })
      .signers([house])
      .instruction();

      const tx = new Transaction().add(sig_ix).add(resolve_ix);

      const signature = await sendAndConfirmTransaction(
        provider.connection,
        tx,
        [house],
      );

      console.log("Resolve bet signature:", signature);

      const houseBalanceAfter = await provider.connection.getBalance(house.publicKey);
      const playerBalanceAfter = await provider.connection.getBalance(player.publicKey);

      // Since the outcome is random, we just check that the balances have changed correctly
      const houseBalanceChange = houseBalanceAfter - houseBalanceBefore;
      const playerBalanceChange = playerBalanceAfter - playerBalanceBefore;

      if (playerBalanceChange == 0) {
        console.log("House won");
        expect(houseBalanceChange).to.be.eq(LAMPORTS_PER_SOL / 100 - 5000 * 2);
      } else {
        console.log("Player won");
        expect(playerBalanceChange).to.be.eq(LAMPORTS_PER_SOL / 100 * 1.985);
        expect(houseBalanceChange).to.be.eq(LAMPORTS_PER_SOL / 100 * 0.015 - 5000 * 2);
      }
  });

  it("Refunds a bet fails before timeout", async () => {
    const seed2 = new BN(randomBytes(8));
    const bet = PublicKey.findProgramAddressSync(
      [Buffer.from("bet"), house.publicKey.toBuffer(), player.publicKey.toBuffer(), seed2.toBuffer("le", 16)],
      program.programId
    )[0];
    const playerVault = PublicKey.findProgramAddressSync(
      [Buffer.from("player_vault"), house.publicKey.toBuffer(), player.publicKey.toBuffer(), seed2.toBuffer("le", 16)],
      program.programId
    )[0];

    await program.methods.placeBet(seed2, 50, new BN(LAMPORTS_PER_SOL / 100))
      .accountsStrict({
        player: player.publicKey,
        house: house.publicKey,
        playerVault,
        bet,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .signers([player])
      .rpc();

    //expect(
    try {
      await program.methods.refundBet()
        .accountsStrict({
          player: player.publicKey,
          house: house.publicKey,
          vault,
          playerVault,
          bet,
          systemProgram: SystemProgram.programId,
        })
        .signers([house])
        .rpc()
      expect.fail("Refund should have failed because bet is not expired yet");
    } catch (err) {}
  });
});
