use anchor_lang::prelude::*;

pub mod state;
pub mod instructions;

pub use state::*;
pub use instructions::*;

declare_id!("oHZ31emYbKtjHBHDM4etLd7MYputcyyhJPttWU1SYtV");

#[program]
pub mod lesson_5_escrow {

    use super::*;

    pub fn make(ctx: Context<Make>, seed: u64, deposit: u64, receive: u64) -> Result<()> {
        ctx.accounts.init_escrow(seed, receive, &ctx.bumps)?;
        ctx.accounts.deposit(deposit)
    }

    pub fn refund(ctx: Context<Refund>) -> Result<()> {
        ctx.accounts.refund()
    }
}

#[derive(Accounts)]
pub struct Initialize {}
