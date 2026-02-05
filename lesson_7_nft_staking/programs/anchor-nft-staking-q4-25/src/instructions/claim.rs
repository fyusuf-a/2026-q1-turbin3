use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{Mint, MintTo, Token, TokenAccount, mint_to},
};

use crate::state::{StakeConfig, UserAccount};

#[derive(Accounts)]
pub struct Claim<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(
        seeds = [b"config".as_ref()],
        bump = config.bump,
    )]
    pub config: Account<'info, StakeConfig>,

    #[account(
        seeds = [b"rewards".as_ref(), config.key().as_ref()],
        bump = config.rewards_bump,
    )]
    pub reward_mint: Account<'info, Mint>,

    #[account(
        init_if_needed,
        payer = user,
        associated_token::mint = reward_mint,
        associated_token::authority = user,
        associated_token::token_program = token_program,
    )]
    pub user_ata: Account<'info, TokenAccount>,

    #[account(
        seeds = [b"user".as_ref(), user.key().as_ref()],
        bump = user_account.bump,
    )]
    pub user_account: Account<'info, UserAccount>,
    
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

impl<'info> Claim<'info> {
    pub fn claim(&mut self) -> Result<()> {
        let cpi_program = self.token_program.to_account_info();
        let cpi_accounts = MintTo {
            mint: self.reward_mint.to_account_info(),
            to: self.user_ata.to_account_info(),
            authority: self.config.to_account_info(),
        };

        let config_key = self.config.key();
        let signer_seeds: &[&[&[u8]]] = &[&[
            b"rewards",
            config_key.as_ref(),
            &[self.config.rewards_bump],
        ]];
        let cpi_context = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer_seeds);

        let minted_quantity = (self.user_account.points as u64) * (self.reward_mint.decimals as u64);

        mint_to(cpi_context, minted_quantity)?;

        self.user_account.points = 0;

        Ok(())
    }
}

