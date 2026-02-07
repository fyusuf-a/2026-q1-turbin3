use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{burn, Burn, Mint, Token, TokenAccount },
};
use constant_product_curve::ConstantProduct;

use crate::{errors::AmmError, state::Config, utils::{PRECISION, withdraw_tokens}};

#[derive(Accounts)]
pub struct Withdraw<'info> {
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

impl<'info> Withdraw<'info> {
    pub fn withdraw(
        &mut self,
        amount: u64, // Amount of LP tokens that the user wants to "burn"
        min_x: u64,  // Minimum amount of token X that the user wants to receive
        min_y: u64,  // Minimum amount of token Y that the user wants to receive
    ) -> Result<()> {
        require!(self.config.locked == false, AmmError::PoolLocked);
        require!(amount != 0, AmmError::InvalidAmount);
        require!(self.mint_lp.supply != 0 || self.vault_x.amount != 0 || self.vault_y.amount != 0, AmmError::NoLiquidityInPool);

        let amounts = ConstantProduct::xy_withdraw_amounts_from_l(
                    self.vault_x.amount,
                    self.vault_y.amount,
                    self.mint_lp.supply,
                    amount,
                    PRECISION as u32,
                )
                .unwrap();

        require!(amounts.x <= min_x && amounts.y <= min_y, AmmError::SlippageExceeded);

        // deposit token x
        self.withdraw_tokens(true, amounts.x)?;
        // deposit token y
        self.withdraw_tokens(false, amounts.y)?;
        // mint lp tokens
        self.burn_lp_tokens(amount)
    }

    pub fn withdraw_tokens(&self, is_x: bool, amount: u64) -> Result<()> {
        let (from, to) = match is_x {
            true => (&self.vault_x, &self.user_x),
            false => (&self.vault_y, &self.user_y),
        };
        withdraw_tokens(from, to, &self.config, &self.token_program, amount)
    }

    pub fn burn_lp_tokens(&self, amount: u64) -> Result<()> {
        let cpi_program = self.token_program.to_account_info();

        let cpi_accounts = Burn {
            from: self.mint_lp.to_account_info(),
            mint: self.mint_lp.to_account_info(),
            authority: self.user.to_account_info(),
        };

        let ctx = CpiContext::new(cpi_program, cpi_accounts);

        burn(ctx, amount)
    }
}
