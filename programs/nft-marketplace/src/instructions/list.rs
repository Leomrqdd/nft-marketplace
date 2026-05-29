use anchor_lang::prelude::*;

use mpl_core::{
    ID as MPL_CORE_ID,
    instructions::TransferV1CpiBuilder
};
use crate::state::Marketplace;
use crate::state::Listing;

#[derive(Accounts)]
#[instruction(name: String)]

pub struct List<'info> {
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
        init,
        payer = maker,
        space = Listing::DISCRIMINATOR.len() + Listing::INIT_SPACE,
        seeds = [b"listing",asset.key().as_ref()],
        bump
    )]
    pub listing: Account<'info, Listing>,

    /// CHECK: address is constrained to MPL_CORE_ID
    #[account(address = MPL_CORE_ID)]
    pub mpl_core_program:UncheckedAccount<'info>,
    pub system_program: Program<'info, System>,
}


impl<'info> List<'info> {
    pub fn list(&mut self, price: u64, bumps: &ListBumps) -> Result<()> {
        self.listing.set_inner(Listing {
            maker: self.maker.key(),
            asset: self.asset.key(),
            price,
            payment_mint: None,
            bump: bumps.listing,
        });

        TransferV1CpiBuilder::new(&self.mpl_core_program.to_account_info())
        .asset(&self.asset.to_account_info())
        .collection(self.collection.as_ref().map(|c| c.as_ref()))
        .payer(&self.maker.to_account_info())
        .authority(Some(&self.maker.to_account_info()))
        .new_owner(&self.listing.to_account_info())
        .system_program(Some(&self.system_program.to_account_info()))
        .invoke()?;
        Ok(())
    }
}

