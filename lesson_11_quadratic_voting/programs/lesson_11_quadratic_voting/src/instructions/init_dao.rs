use anchor_lang::prelude::*;
use anchor_spl::token_interface::{Mint, TokenInterface};

use crate::state::*;

#[derive(Accounts)]
#[instruction(name: String)]
pub struct InitDao<'info> {
    #[account(mut)]
    pub creator: Signer<'info>,

    #[account(
        init,
        payer = creator,
        space = Dao::DISCRIMINATOR.len() + Dao::INIT_SPACE,
        seeds = [b"dao", creator.key().as_ref(), name.as_bytes()],
        bump
    )]
    pub dao_account: Account<'info, Dao>,

    pub mint: InterfaceAccount<'info, Mint>,

    pub token_program: Interface<'info, TokenInterface>,
    pub system_program: Program<'info, System>,
}

pub fn init_dao(ctx: Context<InitDao>, name: String) -> Result<()> {
    let dao_account = &mut ctx.accounts.dao_account;

    dao_account.set_inner(Dao {
        name,
        mint: ctx.accounts.mint.key(),
        authority: ctx.accounts.creator.key(),
        proposal_count: 0,
        bump: ctx.bumps.dao_account,
    });
    Ok(())
}
