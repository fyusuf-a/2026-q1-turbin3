use anchor_lang::{prelude::*, system_program::{Transfer, transfer}};

use crate::state::Bet;
use crate::errors::*;

#[derive(Accounts)]
#[instruction(seed:u128)]
pub struct PlaceBet<'info> {
    #[account(mut)]
    pub player: Signer<'info>,
    ///CHECK: This is safe
    pub house: UncheckedAccount<'info>,

    #[account(
        mut,
        seeds = [b"player_vault", house.key().as_ref(), player.key().as_ref(), seed.to_le_bytes().as_ref()],
        bump
    )]
    pub player_vault: SystemAccount<'info>,
    #[account(
        init,
        payer = player,
        space = Bet::DISCRIMINATOR.len() + Bet::INIT_SPACE,
        seeds = [b"bet", house.key().as_ref(), player.key().as_ref(), seed.to_le_bytes().as_ref()],
        bump
    )]
    pub bet: Account<'info, Bet>,
    pub system_program: Program<'info, System>
}

impl<'info> PlaceBet<'info> {
    pub fn create_bet(&mut self, bumps: &PlaceBetBumps, seed: u128, roll: u8, amount: u64) -> Result<()> {
        self.bet.set_inner(Bet{
            slot : Clock::get()?.slot,
            player: self.player.key(),
            seed,
            roll,
            amount,
            bump : bumps.bet,
        });
        Ok(())
    }

    pub fn deposit(&mut self, amount: u64) -> Result<()> {
        let rent_exempt: u64 = Rent::get()?.minimum_balance(self.player_vault.to_account_info().data_len());
        require!(amount > rent_exempt, DiceError::MinimumBet);

        let accounts = Transfer {
            from: self.player.to_account_info(),
            to: self.player_vault.to_account_info()
        };

        let ctx = CpiContext::new(
            self.system_program.to_account_info(),
            accounts
        );
        transfer(ctx, amount)
    }
}