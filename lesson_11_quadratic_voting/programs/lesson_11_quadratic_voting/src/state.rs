use anchor_lang::prelude::*;

#[account]
#[derive(Debug, InitSpace)]
pub struct Dao {
    pub authority: Pubkey,
    #[max_len(500)]
    pub name: String,
    pub proposal_count: u64,
    pub bump: u8,
    pub mint: Pubkey,
}

#[account]
#[derive(Debug, InitSpace)]
pub struct Proposal {
    pub authority: Pubkey,
    #[max_len(500)]
    pub metadata: String,
    pub yes_vote_count: u64,
    pub no_vote_count: u64,
    pub bump: u8,
}

#[account]
#[derive(Debug, InitSpace)]
pub struct Vote {
    pub authority: Pubkey,
    pub vote_credits: u64,
    pub bump: u8,
}
