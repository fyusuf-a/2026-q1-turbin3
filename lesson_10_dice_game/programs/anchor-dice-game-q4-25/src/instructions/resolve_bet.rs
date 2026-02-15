use anchor_instruction_sysvar::{Ed25519InstructionSignatures};
use anchor_lang::{
    prelude::*,
    system_program::{Transfer, transfer}
};
use solana_program::{ed25519_program, sysvar::instructions::load_instruction_at_checked, hash::hash};

use crate::{errors::DiceError, state::Bet};

pub const HOUSE_EDGE: u16 = 150; // 1.5% House edge

#[derive(Accounts)]
pub struct ResolveBet<'info> {
    #[account(mut)]
    pub house: Signer<'info>,

    #[account(mut)]
    /// CHECK: this is safe
    pub player: UncheckedAccount<'info>,

    #[account(
        mut,
        seeds = [b"vault", house.key().as_ref()],
        bump
    )]
    pub vault: SystemAccount<'info>,

    #[account(
        mut,
        seeds = [b"player_vault", house.key().as_ref(), player.key().as_ref(), bet.seed.to_le_bytes().as_ref()],
        bump
    )]
    pub player_vault: SystemAccount<'info>,

    #[account(
        mut,
        has_one = player,
        seeds = [b"bet", house.key().as_ref(), player.key().as_ref(), bet.seed.to_le_bytes().as_ref()],
        bump
    )]
    pub bet: Account<'info, Bet>,

    #[account(
        address = solana_program::sysvar::instructions::ID,
    )]
    /// CHECK: address is checked
    pub instruction_sysvar: UncheckedAccount<'info>,

    pub system_program: Program<'info, System>
}

impl<'info> ResolveBet<'info> {
    pub fn verify_ed25519_signature(&mut self, sig: &[u8]) -> Result<()> {
        let ix = load_instruction_at_checked(0, &self.instruction_sysvar.to_account_info())
            .map_err(|_| DiceError::Ed25519Program)?;
        require_eq!(ix.program_id, ed25519_program::ID, DiceError::Ed25519Program);
        require_eq!(ix.accounts.len(), 0, DiceError::Ed25519Accounts);

        let Ed25519InstructionSignatures(signatures) = Ed25519InstructionSignatures::unpack(&ix.data).map_err(|_| DiceError::Ed25519Signature)?;

        require_eq!(signatures.len(), 1, DiceError::Ed25519Signature);

        let signature = &signatures[0];

        require!(signature.is_verifiable, DiceError::Ed25519Header);

        require_keys_eq!(
            signature.public_key.ok_or(DiceError::Ed25519Pubkey)?,
            self.house.key(),
            DiceError::Ed25519Pubkey
        );

        require!(&signature.signature.ok_or(DiceError::Ed25519Signature)?.eq(sig), DiceError::Ed25519Signature);

        require!(&signature.message.as_ref().ok_or(DiceError::Ed25519Signature)?.eq(&self.bet.to_slice()), DiceError::Ed25519Message);

        Ok(())
    }

    pub fn resolve_bet(&mut self, sig: &[u8], bumps: &ResolveBetBumps) -> Result<()> {
        let hash = hash(sig).to_bytes();
        let mut hash_16: [u8;16] = [0;16];
        hash_16.copy_from_slice(&hash[0..16]);
        let lower = u128::from_le_bytes(hash_16);

        hash_16.copy_from_slice(&hash[16..32]);
        let upper = u128::from_le_bytes(hash_16);

        let roll = lower
            .wrapping_add(upper)
            .wrapping_rem(100) as u8
            + 1;

        let payout: u64 = (self.bet.amount as u128)
            .checked_mul(10_000 - HOUSE_EDGE as u128)
            .ok_or(DiceError::Overflow)? 
            .checked_div(10_000)
            .ok_or(DiceError::Overflow)?
            .try_into()
            .map_err(|_| DiceError::Overflow)?;

        msg!("roll: {:?}", roll);
        let bet_won = 50 < roll;

        if bet_won {
            self.transfer_from_player_vault(self.player.to_account_info(), payout, bumps)?;

            let remainder = self.player_vault.try_lamports()?;
            self.transfer_from_player_vault(self.house.to_account_info(), remainder, bumps)?;

            self.transfer_from_vault(self.player.to_account_info(), self.bet.amount, bumps)
        } else {
            let remainder = self.player_vault.try_lamports()?;
            self.transfer_from_player_vault(self.house.to_account_info(), remainder, bumps)
        }
    }

    pub fn transfer_from_vault(&self, to: AccountInfo<'info>, amount: u64, bumps: &ResolveBetBumps) -> Result<()> {
        let signer_seeds: &[&[&[u8]]] = &[&[
            b"vault",
            &self.house.key().to_bytes(),
            &[bumps.vault]
        ]];

        let accounts = Transfer {
            from: self.vault.to_account_info(),
            to,
        };
        let ctx = CpiContext::new_with_signer(
            self.system_program.to_account_info(),
            accounts,
            signer_seeds
        );
        transfer(ctx, amount)
    }

    pub fn transfer_from_player_vault(&self, to: AccountInfo<'info>, amount: u64, bumps: &ResolveBetBumps) -> Result<()> {
        let signer_seeds: &[&[&[u8]]] = &[&[
            b"player_vault",
            &self.house.key().to_bytes(),
            &self.player.key().to_bytes(),
            &self.bet.seed.to_le_bytes(),
            &[bumps.player_vault]
        ]];

        let accounts = Transfer {
            from: self.player_vault.to_account_info(),
            to,
        };
        let ctx = CpiContext::new_with_signer(
            self.system_program.to_account_info(),
            accounts,
            signer_seeds
        );
        transfer(ctx, amount)
    }
}
