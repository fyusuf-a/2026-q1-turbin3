use anchor_lang::prelude::*;
use mpl_core::{
    ID as CORE_PROGRAM_ID, instructions::{RemovePluginV1CpiBuilder, UpdatePluginV1CpiBuilder}, types::{FreezeDelegate, Plugin, PluginType}
};

use crate::{
    errors::StakeError,
    state::{StakeAccount, StakeConfig, UserAccount},
};

 #[derive(Accounts)]
 pub struct Unstake<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(
        mut,
        constraint = asset.owner == &CORE_PROGRAM_ID @ StakeError::InvalidAsset,
        constraint = !asset.data_is_empty() @ StakeError::AssetNotInitialized
    )]
    /// CHECK checked by core as well
    pub asset: UncheckedAccount<'info>,

    #[account(
        mut,
        constraint = collection.owner == &CORE_PROGRAM_ID @ StakeError::InvalidCollection,
        constraint = !collection.data_is_empty() @ StakeError::CollectionNotInitialized
    )]
    /// CHECK checked by core as well
    pub collection: UncheckedAccount<'info>,

    #[account(
        mut,
        close = user,
        seeds = [b"stake", config.key().as_ref(), asset.key().as_ref()],
        bump = stake_account.bump,
        constraint = stake_account.owner == user.key() @ StakeError::NotOwner,
    )]
    pub stake_account: Account<'info, StakeAccount>,

    #[account(
        seeds = [b"config".as_ref(), config.key().as_ref()],
        bump = config.bump,
    )]
    pub config: Account<'info, StakeConfig>,

    #[account(
        seeds = [b"user".as_ref(), user.key().as_ref()],
        bump = user_account.bump,
    )]
    pub user_account: Account<'info, UserAccount>,


    #[account(address = CORE_PROGRAM_ID)]
    /// CHECK: Verified by address constraint
    pub core_program: UncheckedAccount<'info>,

    pub system_program: Program<'info, System>,
 }

 impl<'info> Unstake<'info> {
     pub fn unstake(&mut self) -> Result<()> {
        let time_elapsed = Clock::get()?.unix_timestamp - self.stake_account.staked_at;
        const SECONDS_IN_DAY: i64 = 24 * 60 * 60;
        let days_elapsed = u32::try_from(time_elapsed / SECONDS_IN_DAY)?;
        require!(days_elapsed > self.config.freeze_period, StakeError::FreezePeriodNotPassed);

        let points_earned: u32 = days_elapsed * (self.config.points_per_stake as u32);
        self.user_account.points = self.user_account.points.saturating_add(points_earned);

        let signer_seeds: &[&[&[u8]]] = &[&[
            b"stake",
            &self.config.key().to_bytes(),
            &self.asset.key().to_bytes(),
            &[self.stake_account.bump],
        ]];

        UpdatePluginV1CpiBuilder::new(&self.core_program.to_account_info())
            .asset(&self.asset.to_account_info())
            .collection(Some(&self.collection.to_account_info()))
            .payer(&self.user.to_account_info())
            .authority(None)
            .system_program(&self.system_program.to_account_info())
            .plugin(Plugin::FreezeDelegate(FreezeDelegate { frozen: false }))
            .invoke_signed(signer_seeds)?;

        RemovePluginV1CpiBuilder::new(&self.core_program.to_account_info())
            .asset(&self.asset.to_account_info())
            .collection(Some(&self.collection.to_account_info()))
            .payer(&self.user.to_account_info())
            .authority(None)
            .system_program(&self.system_program.to_account_info())
            .plugin_type(PluginType::FreezeDelegate)
            .invoke()?;

        self.user_account.amount_staked = self.user_account.amount_staked.saturating_sub(1);

        Ok(())
     }
 }
