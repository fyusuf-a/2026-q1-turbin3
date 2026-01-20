use anchor_lang::prelude::*;

declare_id!("FTTfUpWfzBB55b9pzmoTZ3zU6FhrHX12QLZbkUxeQmcd");

#[program]
pub mod test_hello_world {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        msg!("Greetings from: {:?}", ctx.program_id);
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize {}
