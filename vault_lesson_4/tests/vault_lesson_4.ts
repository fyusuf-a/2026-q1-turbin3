import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { VaultLesson4 } from "../target/types/vault_lesson_4";
import { expect } from "chai";

describe("vault_lesson_4", () => {
  // Configure the client to use the local cluster.
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.vaultLesson4 as Program<VaultLesson4>;
  const user = provider.wallet.publicKey;

  // Derive PDAs
  const [vaultStatePda, stateBump] = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from("state"), user.toBuffer()],
    program.programId
  );

  const [vaultPda, vaultBump] = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from("vault"), vaultStatePda.toBuffer()],
    program.programId
  );

  before(async () => {
    // Airdrop for fees
    const signature = await provider.connection.requestAirdrop(user, 10 * anchor.web3.LAMPORTS_PER_SOL);
    const latestBlockhash = await provider.connection.getLatestBlockhash();

    await provider.connection.confirmTransaction({
      signature,
      ...latestBlockhash,
    }, "confirmed");

    console.log("Airdropped 10 SOL to user:", user.toBase58());
  });

  describe("after initializing the vault", async () => {
    
    before(async () => {
      await program.methods
        .initialize()
        .accountsStrict({
          user: user,
          vaultState: vaultStatePda,
          vault: vaultPda,
          systemProgram: anchor.web3.SystemProgram.programId,
        })
        .rpc();
    });

    it("it creates the vault state account with correct bumps", async () => {
      const vaultState = await program.account.vaultState.fetch(vaultStatePda);
      expect(vaultState.stateBump).to.equal(stateBump);
      expect(vaultState.vaultBump).to.equal(vaultBump);
    });

    it("it creates the vault account with rent-exempt balance", async () => {
      const vaultBalance = await provider.connection.getBalance(vaultPda);
      const rentExemption = await provider.connection.getMinimumBalanceForRentExemption(0);
      expect(vaultBalance).to.equal(rentExemption);
    });
  });

  describe("after depositing SOL into the vault", async () => {
    it("the SOL amount is transfered from user to vault", async () => {
      const depositAmount = 1 * anchor.web3.LAMPORTS_PER_SOL; // 1 SOL

      const initialVaultBalance = await provider.connection.getBalance(vaultPda);
      const initialUserBalance = await provider.connection.getBalance(user);

      await program.methods
        .deposit(new anchor.BN(depositAmount))
        .accountsStrict({
          user: user,
          vault: vaultPda,
          vaultState: vaultStatePda,
          systemProgram: anchor.web3.SystemProgram.programId,
        })
        .rpc();

      const finalVaultBalance = await provider.connection.getBalance(vaultPda);
      const finalUserBalance = await provider.connection.getBalance(user);

      expect(finalVaultBalance - initialVaultBalance).to.equal(depositAmount);
      expect(initialUserBalance - finalUserBalance - depositAmount).to.equal(5000);
    });
  });

});
