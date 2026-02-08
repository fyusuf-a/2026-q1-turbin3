use anchor_lang::prelude::*;
use mpl_core::{
    ID as CORE_PROGRAM_ID, instructions::{UpdatePluginV1CpiBuilder, UpdateV1Builder, UpdateV1CpiBuilder, UpdateV2Builder, UpdateV2CpiBuilder}, types::{FreezeDelegate, Plugin, PluginAuthority}
};

use crate::{error::MPLXCoreError, state::CollectionAuthority};

#[derive(Accounts)]
pub struct UpdateNft<'info> {
   #[account(
       mut,
       constraint = collection_authority.creator == authority.key() @ MPLXCoreError::NotAuthorized,
   )]
   pub authority: Signer<'info>,

   #[account(
       mut,
       constraint = asset.owner == &CORE_PROGRAM_ID @ MPLXCoreError::InvalidAsset,
       constraint = !asset.data_is_empty() @ MPLXCoreError::AssetNotInitialized
   )]
   /// CHECK checked by core program
   pub asset: UncheckedAccount<'info>,

   #[account(
       mut,
       constraint = collection.owner == &CORE_PROGRAM_ID @ MPLXCoreError::InvalidCollection,
       constraint = !collection.data_is_empty() @ MPLXCoreError::CollectionNotInitialized
   )]
   /// CHECK: This will also be checked by core
   pub collection: UncheckedAccount<'info>,

   #[account(
       seeds = [b"collection_authority", collection.key().as_ref()],
       bump = collection_authority.bump,
   )]
   pub collection_authority: Account<'info, CollectionAuthority>,

   #[account(address = CORE_PROGRAM_ID)]
   /// CHECK: Verified by address constraint
   pub core_program: UncheckedAccount<'info>,
   pub system_program: Program<'info, System>,
}

impl<'info> UpdateNft<'info> {
    pub fn update_nft(&mut self, new_name: String) -> Result<()> {
       let signer_seeds: &[&[&[u8]]] = &[&[
           b"collection_authority",
           &self.collection.key().to_bytes(),
           &[self.collection_authority.bump],
       ]];

       UpdateV2CpiBuilder::new(&self.core_program.to_account_info())
          .asset(&self.asset.to_account_info())
          .collection(Some(&self.collection.to_account_info()))
          .payer(&self.authority.to_account_info())
          .authority(Some(&self.collection_authority.to_account_info()))
          .system_program(&self.system_program.to_account_info())
          .new_name(new_name)
          .invoke_signed(signer_seeds)?;
        Ok(())
    }
}