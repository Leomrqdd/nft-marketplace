use anchor_lang::prelude::*;
use anchor_spl::associated_token::AssociatedToken;

use mpl_core::{
    ID as MPL_CORE_ID,
    accounts::BaseCollectionV1,
    instructions::{TransferV1, TransferV1CpiBuilder}
};
use anchor_spl::token_interface::{Mint,TokenInterface,TokenAccount};
use crate::state::Marketplace;
use crate::state::Offer;
use crate::state::Listing;
use crate::error::MarketplaceError;
use anchor_lang::system_program::{transfer, Transfer};
use anchor_spl::token::{MintTo, mint_to};


#[derive(Accounts)]
#[instruction(name: String)]

pub struct CancelOffer<'info> {
    #[account(mut)]
    pub taker: Signer<'info>,

    /// CHECK: this is the maker of the listing
    #[account(mut)]
    pub maker: UncheckedAccount<'info>,

    #[account(
        seeds = [b"marketplace", name.as_str().as_bytes()],
        bump = marketplace.bump,
    )]
    pub marketplace: Account<'info, Marketplace>,

     /// CHECK: this is the asset the maker wants to sell
    #[account(mut)]
    pub asset: UncheckedAccount<'info>,

    /// CHECK: this is collection of the asset
    #[account(mut)]
    pub collection: Option<UncheckedAccount<'info>>,

    #[account(
        seeds = [b"listing",asset.key().as_ref()],
        bump = listing.bump,
        has_one = maker @ MarketplaceError::InvalidListing,
        has_one = asset @ MarketplaceError::InvalidListing
    )]
    pub listing: Account<'info, Listing>,

    #[account(
        mut,
        seeds = [b"treasury", marketplace.key().as_ref()],
        bump = marketplace.treasury_bump,
    )]
    pub treasury: SystemAccount<'info>,

    #[account(
        mut,
        close = taker,
        seeds = [b"offer", listing.key().as_ref(), taker.key().as_ref()],
        bump,
        has_one = taker @ MarketplaceError::InvalidOffer,
        has_one = asset @ MarketplaceError::InvalidOffer,

    )]
    pub offer: Account<'info, Offer>,

    #[account(
        mut,
        seeds = [b"offer_vault", listing.key().as_ref(), taker.key().as_ref()],
        bump,
    )]
    pub offer_vault: SystemAccount<'info>,

    #[account(address = MPL_CORE_ID)]
    pub mpl_core_program:UncheckedAccount<'info>,
    pub system_program: Program<'info, System>,
    pub token_program: Interface<'info, TokenInterface>,
    pub associated_token_program: Program<'info, AssociatedToken>,
}


impl<'info> CancelOffer<'info> {
    pub fn cancel_offer(&mut self) -> Result<()> {


        let listing = self.listing.key();
        let taker = self.taker.key();
        let amount = self.offer_vault.to_account_info().lamports();

        let signer_seeds: &[&[&[u8]]] = &[&[
            b"offer_vault",
            listing.as_ref(),
            taker.as_ref(),
            &[self.offer.bump_vault],
        ]];

        transfer(
            CpiContext::new_with_signer(
                self.system_program.to_account_info(),
                Transfer {
                    from: self.offer_vault.to_account_info(),
                    to: self.taker.to_account_info(),
                },
                signer_seeds
            ),
            amount,
        )?;

        Ok(())
    }
}

