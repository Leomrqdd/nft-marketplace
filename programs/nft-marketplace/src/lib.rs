#![allow(unexpected_cfgs, deprecated, ambiguous_glob_reexports)]

pub mod error;
pub mod instructions;
pub mod state;

use anchor_lang::prelude::*;

pub use instructions::*;
pub use state::*;

declare_id!("G44fXcPH23NvyBJwENuHpsc75DX9wJEWNb16At3zFzMD");

#[program]
pub mod anchor_core_staking {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>, name: String,fee_bps: u16) -> Result<()> {
        ctx.accounts.initialize(fee_bps, name, &ctx.bumps)
    }

    pub fn list(ctx: Context<List>, name: String, price: u64, payment_mint: Option<Pubkey>) -> Result<()> {
        ctx.accounts.list(price, payment_mint, &ctx.bumps)
    }

    pub fn buy(ctx: Context<Buy>, name: String) -> Result<()> {
        ctx.accounts.buy()
    }

    pub fn delist(ctx: Context<Delist>, name: String) -> Result<()> {
        ctx.accounts.delist()
    }

    pub fn buy_with_tokens(ctx: Context<BuyWithTokens>, name: String) -> Result<()> {
        ctx.accounts.buywithtokens()
    }

    pub fn make_offer(ctx: Context<MakeOffer>, name: String, offer: u64) -> Result<()> {
        ctx.accounts.make_offer(offer, &ctx.bumps)
    }

     pub fn take_offer(ctx: Context<TakeOffer>, name: String) -> Result<()> {
        ctx.accounts.take_offer()
    }

     pub fn cancel_offer(ctx: Context<CancelOffer>, name: String) -> Result<()> {
        ctx.accounts.cancel_offer()
    }


    pub fn withdraw_fees(ctx: Context<WithdrawFees>, name: String) -> Result<()> {
        ctx.accounts.withdraw_fees()
    }

}

