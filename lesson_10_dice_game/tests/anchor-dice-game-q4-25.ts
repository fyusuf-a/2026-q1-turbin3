import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { AnchorDiceGameQ425 } from "../target/types/anchor_dice_game_q4_25";
import { Keypair, PublicKey } from "@solana/web3.js";
import { randomBytes } from "crypto";

describe("anchor-dice-game-q4-25", () => {
  // Configure the client to use the local cluster.
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.anchorDiceGameQ425 as Program<AnchorDiceGameQ425>;

  const house = new Keypair();
  const player = new Keypair();
  const seed = new anchor.BN(randomBytes(8));
  console.log("seed:", seed.toString());
  const vault = PublicKey.findProgramAddressSync(
    [Buffer.from("vault"), house.publicKey.toBuffer()],
    program.programId
  )[0];
  const bet = PublicKey.findProgramAddressSync(
    [Buffer.from("bet"), player.publicKey.toBuffer(), seed.toBuffer("le", 16)],
    program.programId
  )[0];
  let signature: Uint8Array;

  before(async () => {
    const users = [house, player];
    const promises = [];

    for (const user of users) {
      // Airdrop for fees
      const signature = await provider.connection.requestAirdrop(user.publicKey, 10 * anchor.web3.LAMPORTS_PER_SOL);
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
    const tx = await program.methods.initialize(new anchor.BN(anchor.web3.LAMPORTS_PER_SOL))
      .accountsStrict({
        house: house.publicKey,
        vault,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .signers([house])
      .rpc();
    console.log("Your transaction signature", tx);
  });
});
