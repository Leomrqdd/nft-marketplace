use anchor_lang::prelude::*;
use anchor_spl::associated_token::AssociatedToken;

use mpl_core::ID as MPL_CORE_ID;
use anchor_spl::token_interface::TokenInterface;
use crate::state::Marketplace;
use crate::state::Offer;
use crate::state::Listing;
use crate::error::MarketplaceError;
use anchor_lang::system_program::{transfer, Transfer};


#[derive(Accounts)]
#[instruction(name: String)]

pub struct MakeOffer<'info> {
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
        init,
        space = Offer::DISCRIMINATOR.len() + Offer::INIT_SPACE,
        payer = taker,
        seeds = [b"offer", listing.key().as_ref(), taker.key().as_ref()],
        bump,

    )]
    pub offer: Account<'info, Offer>,

    #[account(
        seeds = [b"offer_vault", listing.key().as_ref(), taker.key().as_ref()],
        bump,
    )]
    pub offer_vault: SystemAccount<'info>,

    /// CHECK: address is constrained to MPL_CORE_ID
    #[account(address = MPL_CORE_ID)]
    pub mpl_core_program:UncheckedAccount<'info>,
    pub system_program: Program<'info, System>,
    pub token_program: Interface<'info, TokenInterface>,
    pub associated_token_program: Program<'info, AssociatedToken>,
}


impl<'info> MakeOffer<'info> {
    pub fn make_offer(&mut self, offer: u64, bumps: &MakeOfferBumps) -> Result<()> {

        self.offer.set_inner(Offer {
            taker: self.taker.key(),
            asset: self.asset.key(),
            offer_amount: offer,
            bump: bumps.offer,
            bump_vault: bumps.offer_vault,
        });

        transfer(
            CpiContext::new(
                self.system_program.to_account_info(),
                Transfer {
                    from: self.taker.to_account_info(),
                    to: self.offer_vault.to_account_info(),
                },
            ),
            offer,
        )?;

        Ok(())
    }
}

