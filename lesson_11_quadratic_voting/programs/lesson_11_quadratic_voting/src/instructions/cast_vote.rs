use anchor_lang::prelude::*;
use anchor_spl::{associated_token::AssociatedToken, token_interface::TokenInterface};

use crate::state::{Dao, Proposal, Vote};

#[derive(Accounts)]
pub struct CastVote<'info> {
    #[account(mut)]
    pub voter: Signer<'info>,

    pub dao: Account<'info, Dao>,

    pub proposal: Account<'info, Proposal>,

    #[account(
        init,
        payer = voter,
        space = 8 + Vote::INIT_SPACE,
        seeds = [b"vote", voter.key().as_ref(), proposal.key().as_ref()],
        bump
    )]
    pub vote_account: Account<'info, Vote>,

    #[account(
        associated_token::authority = voter,
        associated_token::mint = dao.mint,
        associated_token::token_program = token_program,
    )]
    pub creator_token_account: Account<'info, anchor_spl::token::TokenAccount>,

    pub token_program: Interface<'info, TokenInterface>,

    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>
}

pub fn cast_vote(ctx: Context<CastVote>, vote: bool) -> Result<()> {
    let vote_account = &mut ctx.accounts.vote_account;
    let proposal_account = &mut ctx.accounts.proposal;

    let voting_credits: f64 =  (ctx.accounts.creator_token_account.amount as f64)
        .sqrt()
        .max(0.0)
        .min(u64::MAX as f64);

    vote_account.set_inner(Vote {
        authority: ctx.accounts.voter.key(),
        vote_credits: voting_credits as u64,
        bump: vote_account.bump,
    });

    if vote {
        proposal_account.yes_vote_count = proposal_account.yes_vote_count.saturating_add(1);
    } else {
        proposal_account.no_vote_count = proposal_account.no_vote_count.saturating_add(1);
    }

    Ok(())
}
