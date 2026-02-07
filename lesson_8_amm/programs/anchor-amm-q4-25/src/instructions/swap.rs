use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{Mint, Token, TokenAccount},
};
use constant_product_curve::{ConstantProduct, LiquidityPair, SwapResult};

use crate::{errors::AmmError, state::Config, utils::{self, PRECISION, deposit_tokens}};

#[derive(Accounts)]
pub struct Swap<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    pub mint_x: Account<'info, Mint>,
    pub mint_y: Account<'info, Mint>,
    #[account(
        has_one = mint_x,
        has_one = mint_y,
        seeds = [b"config", config.seed.to_le_bytes().as_ref()],
        bump = config.config_bump,
    )]
    pub config: Account<'info, Config>,
    #[account(
        mut,
        seeds = [b"lp", config.key().as_ref()],
        bump = config.lp_bump,
    )]
    pub mint_lp: Account<'info, Mint>,
    #[account(
        mut,
        associated_token::mint = mint_x,
        associated_token::authority = config,
    )]
    pub vault_x: Account<'info, TokenAccount>,
    #[account(
        mut,
        associated_token::mint = mint_y,
        associated_token::authority = config,
    )]
    pub vault_y: Account<'info, TokenAccount>,
    #[account(
        mut,
        associated_token::mint = mint_x,
        associated_token::authority = user,
    )]
    pub user_x: Box<Account<'info, TokenAccount>>,
    #[account(
        mut,
        associated_token::mint = mint_y,
        associated_token::authority = user,
    )]
    pub user_y: Box<Account<'info, TokenAccount>>,
    #[account(
        mut,
        associated_token::mint = mint_lp,
        associated_token::authority = user,
    )]
    pub user_lp: Box<Account<'info, TokenAccount>>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub associated_token_program: Program<'info, AssociatedToken>,
}

impl<'info> Swap<'info> {
     pub fn swap(&mut self, is_x: bool, amount: u64, min: u64) -> Result<()> {
        require!(self.config.locked == false, AmmError::PoolLocked);
        require!(amount != 0, AmmError::InvalidAmount);
        require!(self.mint_lp.supply != 0 || self.vault_x.amount != 0 || self.vault_y.amount != 0, AmmError::NoLiquidityInPool);

        let pair = if is_x { LiquidityPair::X } else { LiquidityPair::Y };

        let mut product = ConstantProduct::init(
            self.vault_x.amount,
            self.vault_y.amount,
            self.mint_lp.supply,
            self.config.fee,
            Some(PRECISION),
        ).unwrap();

        let SwapResult { deposit, withdraw, fee } = product.swap_unsafe(pair, amount).unwrap();

        require!(withdraw >= min, AmmError::SlippageExceeded);

        self.deposit_tokens(is_x, deposit).unwrap();

        self.withdraw_tokens(is_x, amount + fee)

        /*match self.config.authority {
            Some(pubkey) => {
                self.withdraw_tokens(!is_x, withdraw).unwrap();
                utils::helpers::withdraw_tokens(
                    if is_x { &self.vault_y } else { &self.vault_x },
                    pubkey,
                    &self.config,
                    &self.token_program,
                    fee,
                );
                return Ok(())
            },
            None => { return self.withdraw_tokens(!is_x, withdraw + fee) }
        }*/
    }

    pub fn deposit_tokens(&self, is_x: bool, amount: u64) -> Result<()> {
        let (from, to) = match is_x {
            true => (&self.user_x, &self.vault_x),
            false => (&self.user_y, &self.vault_y),
        };
        deposit_tokens(from, to, &self.token_program, amount)
    }

    pub fn withdraw_tokens(&self, is_x: bool, amount: u64) -> Result<()> {
        let (from, to) = match is_x {
            true => (&self.vault_x, &self.user_x),
            false => (&self.vault_y, &self.user_y),
        };
        utils::helpers::withdraw_tokens(from, to, &self.config, &self.token_program, amount)
    }
}
