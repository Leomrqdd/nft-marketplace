#![allow(unexpected_cfgs, deprecated, ambiguous_glob_reexports)]

pub mod error;
pub mod instructions;
pub mod state;

use anchor_lang::prelude::*;

pub use instructions::*;
pub use state::*;

declare_id!("FdQ9QdcVkYVqqGwCX8ghKiZAiDzYdbuC9c63rhuqVCmf");

#[program]
pub mod anchor_core_staking {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>, name: String,fee_bps: u16) -> Result<()> {
        ctx.accounts.initialize(fee_bps, name, &ctx.bumps)
    }

    pub fn list(ctx: Context<List>, price: u64) -> Result<()> {
        ctx.accounts.list(price, &ctx.bumps)
    }

    pub fn buy (ctx:Context<Buy>) -> Result<()> {
        ctx.accounts.buy()
    }

}

