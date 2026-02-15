use anchor_lang::prelude::*;

pub mod state;
pub mod instructions;

use instructions::*;

declare_id!("59xbwXbL59nrRgvTDtPLSJpMeZCwkrMvqcD3bhaoeK5k");

#[program]
pub mod lesson_11_quadratic_voting {
    use super::*;

    pub fn initialize_dao(ctx: Context<InitDao>, name: String) -> Result<()> {
        init_dao(ctx, name)
    }
}

#[derive(Accounts)]
pub struct Initialize {}
