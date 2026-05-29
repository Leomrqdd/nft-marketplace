use anchor_lang::prelude::*;
use anchor_spl::associated_token::AssociatedToken;

use mpl_core::{
    ID as MPL_CORE_ID,
    instructions::TransferV1CpiBuilder
};
use anchor_spl::token_interface::{Mint,TokenInterface,TokenAccount};
use crate::state::Marketplace;
use crate::state::Listing;
use crate::error::MarketplaceError;
use anchor_lang::system_program::{transfer, Transfer};
use anchor_spl::token::mint_to;


#[derive(Accounts)]
#[instruction(name: String)]

pub struct Buy<'info> {
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
        mut,
        close = maker,
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
        mint::decimals = 6,
        mint::authority = marketplace,
        seeds = [b"reward_mint", marketplace.key().as_ref()],
        bump = marketplace.rewards_bump,
    )]
    pub rewards_mint: InterfaceAccount<'info,Mint>,

    #[account(
        init_if_needed,
        payer = taker,
        associated_token::mint = rewards_mint,
        associated_token::authority = taker,
        associated_token::token_program = token_program,

    )]
    pub taker_rewards_ata: InterfaceAccount<'info,TokenAccount>,

    /// CHECK: address is constrained to MPL_CORE_ID
    #[account(address = MPL_CORE_ID)]
    pub mpl_core_program:UncheckedAccount<'info>,
    pub system_program: Program<'info, System>,
    pub token_program: Interface<'info, TokenInterface>,
    pub associated_token_program: Program<'info, AssociatedToken>,
}


impl<'info> Buy<'info> {
    pub fn buy(&mut self) -> Result<()> {


        let price = self.listing.price;
        let fees = price.checked_mul(self.marketplace.fee_bps as u64).unwrap().checked_div(10_000).unwrap();
        let seller_amount = price.checked_sub(fees).unwrap();

        transfer(
            CpiContext::new(
                self.system_program.to_account_info(),
                Transfer {
                    from: self.taker.to_account_info(),
                    to: self.maker.to_account_info(),
                },
            ),
            seller_amount,
        )?;

         transfer(
            CpiContext::new(
                self.system_program.to_account_info(),
                Transfer {
                    from: self.taker.to_account_info(),
                    to: self.treasury.to_account_info(),
                },
            ),
            fees,
        )?;

        let asset = self.asset.key();

        let signer_seeds: &[&[&[u8]]] = &[&[
            b"listing",
            asset.as_ref(),
            &[self.listing.bump],
        ]];



        TransferV1CpiBuilder::new(&self.mpl_core_program.to_account_info())
        .asset(&self.asset.to_account_info())
        .collection(self.collection.as_ref().map(|c| c.as_ref()))
        .payer(&self.taker.to_account_info())
        .authority(Some(&self.listing.to_account_info()))
        .new_owner(&self.taker.to_account_info())
        .system_program(Some(&self.system_program.to_account_info()))
        .invoke_signed(signer_seeds)?;


        let name_bytes = self.marketplace.name.as_bytes().to_vec();
        let signer_seeds_2 : &[&[&[u8]]] = &[&[
            b"marketplace",
            name_bytes.as_ref(),
            &[self.marketplace.bump],
        ]];

        mint_to(
            CpiContext::new_with_signer(
                self.token_program.to_account_info(),
                anchor_spl::token::MintTo {
                    mint: self.rewards_mint.to_account_info(),
                    to: self.taker_rewards_ata.to_account_info(),
                    authority: self.marketplace.to_account_info(),
                },
                signer_seeds_2,
            ),
            1, // mint 1 reward token per purchase
        )?;


        
        Ok(())
    }
}

