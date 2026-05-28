use anchor_lang::prelude::*;

#[error_code]
pub enum MarketplaceError {
    #[msg("Invalid listing")]
    InvalidListing,
    #[msg("Invalid payment mint")]
    InvalidPaymentMint,
    #[msg("Invalid offer")]
    InvalidOffer,
    #[msg("Unauthorized")]
    Unauthorized,
}
