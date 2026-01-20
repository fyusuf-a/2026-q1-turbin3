import { Keypair, PublicKey, Connection, Commitment, SendTransactionError } from "@solana/web3.js";
import { getOrCreateAssociatedTokenAccount, mintTo } from '@solana/spl-token';
import wallet from "../turbin3-wallet.json"

// Import our keypair from the wallet file
const keypair = Keypair.fromSecretKey(new Uint8Array(wallet));

//Create a Solana devnet connection
const commitment: Commitment = "confirmed";
const connection = new Connection("https://api.devnet.solana.com", commitment);

const token_decimals = 1_000_000n;

// Mint address
const mint = new PublicKey("7xK6fsKDGzgzBTctfpVCJdVdAo4ZJg33w583Jk1DxDc1");

(async () => {
    try {
        // Create an ATA
        const ata = await getOrCreateAssociatedTokenAccount(
            connection,
            keypair,
            mint,
            keypair.publicKey
        );
        console.log(`Your ata is: ${ata.address.toBase58()}`);

        // Mint to ATA
         const mintTx = await mintTo(
            connection,
            keypair,
            mint,
            ata.address,
            keypair,
            10n * token_decimals
        );
        console.log(`Your mint txid: ${mintTx}`);
    } catch(error) {
        if (error instanceof SendTransactionError) {
            const logs = await error.getLogs(connection);
            console.error("Transaction logs:", logs);
        }
    }
})()
