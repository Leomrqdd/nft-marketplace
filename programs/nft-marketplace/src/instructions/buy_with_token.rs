use anchor_lang::prelude::*;
use anchor_spl::associated_token::AssociatedToken;

use mpl_core::{
    ID as MPL_CORE_ID,
    instructions::TransferV1CpiBuilder
};
use anchor_spl::token_interface::{Mint,TokenInterface,TokenAccount,transfer_checked, TransferChecked};
use crate::state::Marketplace;
use crate::state::Listing;
use crate::error::MarketplaceError;
use anchor_spl::token::mint_to;


#[derive(Accounts)]
#[instruction(name: String)]

pub struct BuyWithTokens<'info> {
    #[account(mut)]
    pub taker: Signer<'info>,

    /// CHECK: this is the maker of the listing
    #[account(mut)]
    pub maker: UncheckedAccount<'info>,

    #[account(
        seeds = [b"marketplace", name.as_str().as_bytes()],
        bump = marketplace.bump,
    )]
    pub marketplace: Box<Account<'info, Marketplace>>,

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
        has_one = asset @ MarketplaceError::InvalidListing,
    )]
    pub listing: Box<Account<'info, Listing>>,

    #[account(
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
    pub rewards_mint: Box<InterfaceAccount<'info,Mint>>,

    #[account(
        init_if_needed,
        payer = taker,
        associated_token::mint = rewards_mint,
        associated_token::authority = taker,
        associated_token::token_program = token_program,

    )]
    pub taker_rewards_ata: Box<InterfaceAccount<'info,TokenAccount>>,


    pub payment_mint: Box<InterfaceAccount<'info, Mint>>,

    #[account(
        mut,
        associated_token::mint = payment_mint,
        associated_token::authority = treasury,
    )]
    pub treasury_ata: Box<InterfaceAccount<'info,TokenAccount>>,


    #[account(
        mut,
        associated_token::mint = payment_mint,
        associated_token::authority = taker,
    )]
    pub taker_ata: Box<InterfaceAccount<'info,TokenAccount>>,


    #[account(
        mut,
        associated_token::mint = payment_mint,
        associated_token::authority = maker,
    )]
    pub maker_ata: Box<InterfaceAccount<'info,TokenAccount>>,

    /// CHECK: address is constrained to MPL_CORE_ID
    #[account(address = MPL_CORE_ID)]
    pub mpl_core_program:UncheckedAccount<'info>,
    pub system_program: Program<'info, System>,
    pub token_program: Interface<'info, TokenInterface>,
    pub associated_token_program: Program<'info, AssociatedToken>,
}


impl<'info> BuyWithTokens<'info> {
    pub fn buywithtokens(&mut self) -> Result<()> {
        require!(self.listing.payment_mint.is_some(), MarketplaceError::InvalidPaymentMint);


        let price = self.listing.price;
        let fees = price.checked_mul(self.marketplace.fee_bps as u64).unwrap().checked_div(10_000).unwrap();
        let seller_amount = price.checked_sub(fees).unwrap();

        transfer_checked(
            CpiContext::new(
                self.token_program.to_account_info(),
                TransferChecked {
                    from: self.taker_ata.to_account_info(),
                    to: self.maker_ata.to_account_info(),
                    mint: self.payment_mint.to_account_info(),
                    authority: self.taker.to_account_info(),
                },
            ),
            seller_amount,
            self.payment_mint.decimals
        )?;

        transfer_checked(
            CpiContext::new(
                self.token_program.to_account_info(),
                TransferChecked {
                    from: self.taker_ata.to_account_info(),
                    to: self.treasury_ata.to_account_info(),
                    mint: self.payment_mint.to_account_info(),
                    authority: self.taker.to_account_info(),
                },
            ),
            fees,
            self.payment_mint.decimals
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

