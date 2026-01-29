import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { VaultLesson4 } from "../target/types/vault_lesson_4";
import { expect } from "chai";

const publicKeyToUrl = (key: anchor.web3.PublicKey) => {
  return "https://explorer.solana.com/address/" + key.toBase58() + "?cluster=custom&customUrl=http%3A%2F%2Flocalhost%3A8899";
}

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

  console.log(`See vaultStatePda: ${publicKeyToUrl(vaultStatePda)}`);

  const [vaultPda, vaultBump] = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from("vault"), vaultStatePda.toBuffer()],
    program.programId
  );

  console.log(`See vaultPda: ${publicKeyToUrl(vaultPda)}`);

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

  describe("after withdrawing SOL from the vault", async () => {
    it("the SOL amount is transfered from vault to user", async () => {
      const withdrawAmount = 0.5 * anchor.web3.LAMPORTS_PER_SOL; // 0.5 SOL

      const initialVaultBalance = await provider.connection.getBalance(vaultPda);
      const initialUserBalance = await provider.connection.getBalance(user);

      await program.methods
        .withdraw(new anchor.BN(withdrawAmount))
        .accountsStrict({
          user: user,
          vault: vaultPda,
          vaultState: vaultStatePda,
          systemProgram: anchor.web3.SystemProgram.programId,
        })
        .rpc();

      const finalVaultBalance = await provider.connection.getBalance(vaultPda);
      const finalUserBalance = await provider.connection.getBalance(user);

      expect(initialVaultBalance - finalVaultBalance).to.equal(withdrawAmount);
      expect(finalUserBalance - initialUserBalance - withdrawAmount).to.equal(-5000);
    });
  });

  describe("after closing the vault", async () => {
    it("The rent-exempt amounts are transfered back to the user", async () => {
      const initialVaultBalance = await provider.connection.getBalance(vaultPda);
      const initialVaultStateBalance = await provider.connection.getBalance(vaultStatePda);
      const initialUserBalance = await provider.connection.getBalance(user);

      await program.methods
        .close()
        .accountsStrict({
          user: user,
          vault: vaultPda,
          vaultState: vaultStatePda,
          systemProgram: anchor.web3.SystemProgram.programId,
        })
        .rpc();

      const finalUserBalance = await provider.connection.getBalance(user);

      // Vault should be 0
      expect(await provider.connection.getBalance(vaultPda)).to.equal(0);

      // VaultState should be closed (null)
      const vaultStateInfo = await provider.connection.getAccountInfo(vaultStatePda);
      expect(vaultStateInfo).to.be.null;

      // User gets back the remaining balance - fees
      expect(finalUserBalance).to.equal(initialUserBalance + initialVaultBalance + initialVaultStateBalance - 5000);
    });
  });

});
