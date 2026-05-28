use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct Marketplace {
    pub admin: Pubkey,
    pub fee_bps: u16,
    pub bump: u8,
    pub treasury_bump: u8,
    pub rewards_bump: u8,
    #[max_len(32)]
    pub name: String,
}


#[account]
#[derive(InitSpace)]
pub struct Listing {
    pub maker: Pubkey,
    pub asset: Pubkey,
    pub price: u64,
    pub payment_mint: Option<Pubkey>,
    pub bump: u8,
}



#[account]
#[derive(InitSpace)]
pub struct Offer {
    pub taker: Pubkey,
    pub asset: Pubkey,
    pub offer_amount: u64,
    pub bump: u8,
    pub bump_vault: u8,
}