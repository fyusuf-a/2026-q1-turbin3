use anchor_lang::prelude::*;
use anchor_spl::{token_interface::{Mint, TokenAccount, TransferChecked, transfer_checked, TokenInterface, close_account, CloseAccount}};

use crate::Escrow;

#[derive(Accounts)]
pub struct Refund<'info> {
    #[account(mut)]
    pub maker: Signer<'info>,

    #[account(mint::token_program = token_program_a)]
    pub mint_a: InterfaceAccount<'info, Mint>,

    #[account(
        mut,
        associated_token::mint = mint_a,
        associated_token::authority = maker,
        associated_token::token_program = token_program_a,
    )]
    pub maker_ata_a: InterfaceAccount<'info, TokenAccount>,

    #[account(
        mut,
        close = maker,
        has_one = mint_a,
        has_one = maker,
        seeds = [b"escrow", maker.key.as_ref(), escrow.seed.to_le_bytes().as_ref()],
        bump = escrow.bump,
    )]
    pub escrow: Account<'info, Escrow>,

    #[account(
        mut,
        associated_token::mint = mint_a,
        associated_token::authority = escrow,
        associated_token::token_program = token_program_a,
    )]
    pub vault: InterfaceAccount<'info, TokenAccount>,

    pub token_program_a: Interface<'info, TokenInterface>,
    pub system_program: Program<'info, System>,
}

impl<'info> Refund<'info> {
    pub fn refund(&mut self) -> Result<()> {
        let cpi_program = self.token_program_a.to_account_info();
        let cpi_accounts = TransferChecked {
            from: self.vault.to_account_info(),
            to: self.maker_ata_a.to_account_info(),
            mint: self.mint_a.to_account_info(),
            authority: self.escrow.to_account_info(),
        };

        let le_bytes = self.escrow.seed.to_le_bytes();
        let signer_seeds: &[&[&[u8]]] = &[&[
            b"escrow",
            self.maker.key.as_ref(),
            le_bytes.as_ref(),
            &[self.escrow.bump],
        ]];

        let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer_seeds);
        transfer_checked(cpi_ctx, self.vault.amount, self.mint_a.decimals)?;


        let cpi_program = self.token_program_a.to_account_info();
        let cpi_accounts = CloseAccount {
            account: self.vault.to_account_info(),
            destination: self.maker.to_account_info(),
            authority: self.escrow.to_account_info(),
        };

        let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer_seeds);
        close_account(cpi_ctx)

        // to replace close = maker
        //let mut data = self.escrow.to_account_info().data.borrow_mut();
        //data.fill(0);

        //let amount: u64 = self.escrow.get_lamports();
        //self.escrow.sub_lamports(amount);
        //self.maker.add_lamports(amount);
        // or --->
        //**self.escrow.to_account_info().try_borrow_mut_lamports()? =
            //self.escrow.to_account_info().get_lamports()
                //.checked_sub(amount)
                //.ok_or(ProgramError::ArithmeticOverflow)?;
        //**self.maker.try_borrow_mut_lamports()? = self.maker.to_account_info().get_lamports()
                //.checked_add(amount)
                //.ok_or(ProgramError::ArithmeticOverflow)?;
        //Ok(())
    }
}
