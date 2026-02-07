use anchor_lang::prelude::*;
use anchor_spl::token::{Token, TokenAccount, Transfer, transfer};

use crate::state::Config;

pub const PRECISION: u8 = 6;

pub fn withdraw_tokens<'info>(
    from: &Account<'info, TokenAccount>,
    to: &Account<'info, TokenAccount>,
    config: &Account<'info, Config>,
    token_program: &Program<'info, Token>,
    amount: u64
) -> Result<()> {
    let cpi_program = token_program.to_account_info();

    let cpi_accounts = Transfer {
        from: from.to_account_info(),
        to: to.to_account_info(),
        authority: from.to_account_info(),
    };

    let signer_seeds: &[&[&[u8]]] = &[&[
        b"config",
        &config.seed.to_le_bytes(),
        &[config.config_bump],
    ]];

    let ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer_seeds);

    transfer(ctx, amount)
}

pub fn deposit_tokens<'info>(
    from: &Account<'info, TokenAccount>,
    to: &Account<'info, TokenAccount>,
    token_program: &Program<'info, Token>,
    amount: u64
) -> Result<()> {
    let cpi_program = token_program.to_account_info();

    let cpi_accounts = Transfer {
        from: from.to_account_info(),
        to: to.to_account_info(),
        authority: from.to_account_info(),
    };

    let ctx = CpiContext::new(cpi_program, cpi_accounts);

    transfer(ctx, amount)
}
