use anchor_lang::prelude::*;

use mpl_core::{
    ID as MPL_CORE_ID,
    instructions::TransferV1CpiBuilder
};
use crate::state::Marketplace;
use crate::state::Listing;
use crate::error::MarketplaceError;


#[derive(Accounts)]
#[instruction(name: String)]

pub struct Delist<'info> {
    #[account(mut)]
    pub maker: Signer<'info>,
    #[account(
        seeds = [b"marketplace", name.as_str().as_bytes()],
        bump,
    )]

    pub marketplace: Account<'info, Marketplace>,

     /// CHECK: this is the asset the maker wants to sell
    #[account(mut)]
    pub asset: UncheckedAccount<'info>,

    /// CHECK: this is collection of the asset
    #[account(mut)]
    pub collection: Option<UncheckedAccount<'info>>,

    #[account(
        mut,
        close = maker,
        seeds = [b"listing",asset.key().as_ref()],
        bump,
        has_one = maker @ MarketplaceError::InvalidListing,
        has_one = asset @ MarketplaceError::InvalidListing
    )]
    pub listing: Account<'info, Listing>,

    /// CHECK: address is constrained to MPL_CORE_ID
    #[account(address = MPL_CORE_ID)]
    pub mpl_core_program:UncheckedAccount<'info>,
    pub system_program: Program<'info, System>,
}


impl<'info> Delist<'info> {
    pub fn delist(&mut self) -> Result<()> {

        let asset = self.asset.key();

        let signer_seeds: &[&[&[u8]]] = &[&[
            b"listing",
            asset.as_ref(),
            &[self.listing.bump],
        ]];


        TransferV1CpiBuilder::new(&self.mpl_core_program.to_account_info())
        .asset(&self.asset.to_account_info())
        .collection(self.collection.as_ref().map(|c| c.as_ref()))
        .payer(&self.maker.to_account_info())
        .authority(Some(&self.listing.to_account_info()))
        .new_owner(&self.maker.to_account_info())
        .system_program(Some(&self.system_program.to_account_info()))
        .invoke_signed(signer_seeds)?;

        Ok(())
    }

}