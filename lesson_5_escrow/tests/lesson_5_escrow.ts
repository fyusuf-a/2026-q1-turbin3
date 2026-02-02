import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { Lesson5Escrow } from "../target/types/lesson_5_escrow";
import { ASSOCIATED_TOKEN_PROGRAM_ID, createAssociatedTokenAccountInstruction, createMint, getAssociatedTokenAddressSync, mintTo, TOKEN_PROGRAM_ID } from "@solana/spl-token";
import { expect } from "chai";

const airdropAndMint = async ({
  provider,
  authority,
  ata,
  amount,
}: {
  provider: anchor.AnchorProvider,
  authority: anchor.web3.Keypair,
  ata: anchor.web3.PublicKey,
  amount: number,
}) => {
      const signature = await provider.connection.requestAirdrop(authority.publicKey, 10 * anchor.web3.LAMPORTS_PER_SOL);
      const latestBlockhash = await provider.connection.getLatestBlockhash();
      await provider.connection.confirmTransaction({
        signature,
        ...latestBlockhash,
      }, "confirmed")

      const mint = await createMint(provider.connection, provider.wallet.payer, authority.publicKey, null, 0);
      ata = getAssociatedTokenAddressSync(mint, authority.publicKey);
      const ataTx = new anchor.web3.Transaction().add(
        createAssociatedTokenAccountInstruction(authority.publicKey, ata, authority.publicKey, mint)
      );
      await provider.sendAndConfirm(ataTx, [authority]);
      await mintTo(provider.connection, authority, mint, ata, authority.publicKey, amount);
      return {
        mint,
        ata
      };
};

describe("lesson_5_escrow", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.lesson5Escrow as Program<Lesson5Escrow>;

  const maker = provider.wallet.publicKey;
  const taker = anchor.web3.Keypair.generate();

  let mintA: anchor.web3.PublicKey;
  let mintB: anchor.web3.PublicKey;
  let makerAtaA: anchor.web3.PublicKey;
  let takerAtaB: anchor.web3.PublicKey;
  let makerAtaB: anchor.web3.PublicKey;
  let takerAtaA: anchor.web3.PublicKey;

  const seed = new anchor.BN(1234);
  const seed2 = new anchor.BN(2222);
  let escrowPda: anchor.web3.PublicKey;
  let escrowBump: number;
  let vault: anchor.web3.PublicKey;

  const depositAmount = 100;
  const receiveAmount = 200;

  before(async () => {

    const promises = [];
    for (const obj of [
        { authority: provider.wallet.payer, ata: makerAtaA, amount: depositAmount },
        { authority: taker, ata: takerAtaB, amount: receiveAmount }
    ]) {
      promises.push(airdropAndMint({
        provider,
        authority: obj.authority,
        ata: obj.ata,
        amount: 2 * obj.amount,
      }));
    }
    const results = await Promise.all(promises);
    mintA = results[0].mint;
    makerAtaA = results[0].ata;
    mintB = results[1].mint;
    takerAtaB = results[1].ata;
  });

  describe("after initializing the vault", async () => {
    before(async () => {
      [escrowPda, escrowBump] = anchor.web3.PublicKey.findProgramAddressSync(
        [Buffer.from("escrow"), maker.toBuffer(), seed.toArrayLike(Buffer, "le", 8)],
        program.programId
      );
      vault = getAssociatedTokenAddressSync(mintA, escrowPda, true);

      await program.methods.make(seed,  new anchor.BN(depositAmount), new anchor.BN(receiveAmount))
        .accountsStrict({
          maker,
          mintA,
          mintB,
          makerAtaA,
          escrow: escrowPda,
          vault,
          associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
          tokenProgramA: TOKEN_PROGRAM_ID,
          tokenProgramB: TOKEN_PROGRAM_ID,
          systemProgram: anchor.web3.SystemProgram.programId,
        })
        .rpc();
    });

    it("it sets up the escrow account correctly", async () => {
      const escrowAccount = await program.account.escrow.fetch(escrowPda);
      expect(escrowAccount.seed.toString()).to.equal(seed.toString());
      expect(escrowAccount.maker.toBase58()).to.equal(maker.toBase58());
      expect(escrowAccount.mintA.toBase58()).to.equal(mintA.toBase58());
      expect(escrowAccount.receive.toNumber()).to.equal(receiveAmount);
      expect(escrowAccount.bump).to.equal(escrowBump);
    });

    it("it transfers the tokens to the vault", async () => {
      const vaultAccount = await program.provider.connection.getTokenAccountBalance(vault);
      expect(vaultAccount.value.uiAmount).to.equal(depositAmount);
    });
  });

  describe("when refunding the escrow", async () => {
    it("it transfers the tokens to the maker", async () => {
      // Refund
      await program.methods
        .refund()
        .accountsStrict({
          maker: maker,
          mintA: mintA,
          makerAtaA: makerAtaA,
          escrow: escrowPda,
          vault: vault,
          tokenProgramA: TOKEN_PROGRAM_ID,
          systemProgram: anchor.web3.SystemProgram.programId,
        })
        .rpc();

      // Check closed
      const escrowInfo = await provider.connection.getAccountInfo(escrowPda);
      expect(escrowInfo).to.be.null;

      const vaultInfo = await provider.connection.getAccountInfo(vault);
      expect(vaultInfo).to.be.null;
    });
  });

  describe("When taking the escrow", async () => {

    before(async () => {
      [escrowPda, escrowBump] = anchor.web3.PublicKey.findProgramAddressSync(
        [Buffer.from("escrow"), maker.toBuffer(), seed2.toArrayLike(Buffer, "le", 8)],
        program.programId
      );
      vault = getAssociatedTokenAddressSync(mintA, escrowPda, true);

      // Make (again for take path)
      await program.methods
        .make(seed2, new anchor.BN(depositAmount), new anchor.BN(receiveAmount))
        .accountsStrict({
          maker: maker,
          mintA: mintA,
          mintB: mintB,
          makerAtaA: makerAtaA,
          escrow: escrowPda,
          vault: vault,
          associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
          tokenProgramA: TOKEN_PROGRAM_ID,
          tokenProgramB: TOKEN_PROGRAM_ID,
          systemProgram: anchor.web3.SystemProgram.programId,
        })
        .rpc();


      // Setup for take
      takerAtaA = getAssociatedTokenAddressSync(mintA, taker.publicKey);
      makerAtaB = getAssociatedTokenAddressSync(mintB, maker);

      // Take
      await program.methods
        .take()
        .accountsStrict({
          taker: taker.publicKey,
          maker: maker,
          mintA: mintA,
          mintB: mintB,
          makerAtaB: makerAtaB,
          takerAtaA: takerAtaA,
          takerAtaB: takerAtaB,
          escrow: escrowPda,
          vault: vault,
          associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
          tokenProgramA: TOKEN_PROGRAM_ID,
          tokenProgramB: TOKEN_PROGRAM_ID,
          systemProgram: anchor.web3.SystemProgram.programId,
        })
        .signers([taker])
        .rpc();
    });

    // Check closed
    it("it closes the escrow", async () => {
      const escrowInfo = await provider.connection.getAccountInfo(escrowPda);
      expect(escrowInfo).to.be.null;
    });

    it("it closes the vault", async () => {
      const vaultInfo = await provider.connection.getAccountInfo(vault);
      expect(vaultInfo).to.be.null;
    });

    it("it transfers token A to the taker", async () => {
      const takerBalanceA = (await provider.connection.getTokenAccountBalance(takerAtaA)).value.uiAmount;
      expect(takerBalanceA).to.equal(depositAmount);
    });

    it("it transfers token B to the maker", async () => {
      const makerBalanceB = (await provider.connection.getTokenAccountBalance(makerAtaB)).value.uiAmount;
      expect(makerBalanceB).to.equal(receiveAmount);
    });
  });

});
