use anchor_lang::prelude::*;
use anchor_spl::associated_token::AssociatedToken;

use mpl_core::ID as MPL_CORE_ID;
use anchor_spl::token_interface::TokenInterface;
use crate::state::Marketplace;
use crate::error::MarketplaceError;
use anchor_lang::system_program::{transfer, Transfer};


#[derive(Accounts)]
#[instruction(name: String)]

pub struct WithdrawFees<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,

    /// CHECK: this is the maker of the listing
    #[account(mut)]
    pub maker: UncheckedAccount<'info>,

    #[account(
        seeds = [b"marketplace", name.as_str().as_bytes()],
        bump = marketplace.bump,
        has_one = admin @ MarketplaceError::Unauthorized,
    )]
    pub marketplace: Account<'info, Marketplace>,

    #[account(
        mut,
        seeds = [b"treasury", marketplace.key().as_ref()],
        bump = marketplace.treasury_bump,
    )]
    pub treasury: SystemAccount<'info>,


    /// CHECK: address is constrained to MPL_CORE_ID
    #[account(address = MPL_CORE_ID)]
    pub mpl_core_program:UncheckedAccount<'info>,
    pub system_program: Program<'info, System>,
    pub token_program: Interface<'info, TokenInterface>,
    pub associated_token_program: Program<'info, AssociatedToken>,
}


impl<'info> WithdrawFees<'info> {
    pub fn withdraw_fees(&mut self) -> Result<()> {

        let amount = self.treasury.to_account_info().lamports();

        let marketplace = self.marketplace.key();


        let signer_seeds: &[&[&[u8]]] = &[&[
            b"treasury",
            marketplace.as_ref(),
            &[self.marketplace.treasury_bump],
        ]];

        transfer(
            CpiContext::new_with_signer(
                self.system_program.to_account_info(),
                Transfer {
                    from: self.treasury.to_account_info(),
                    to: self.admin.to_account_info(),
                },
                signer_seeds
            ),
            amount,
        )?;

        Ok(())
    }
}

