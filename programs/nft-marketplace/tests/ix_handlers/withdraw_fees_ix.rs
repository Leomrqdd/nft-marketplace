use {
    anchor_lang::{Id, InstructionData, ToAccountMetas, prelude::System},
    anchor_spl::{associated_token::AssociatedToken, token::Token},
    mpl_core::ID as MPL_CORE_ID,
    solana_instruction::Instruction,
    solana_keypair::Keypair,
    solana_pubkey::Pubkey,
    solana_signer::Signer,
};

pub fn withdraw_fees_ix(
    admin: &Keypair,
    marketplace: Pubkey,
    treasury: Pubkey,
) -> Instruction {
    Instruction::new_with_bytes(
        nft_marketplace::id(),
        &nft_marketplace::instruction::WithdrawFees { name: "My Marketplace".to_string() }.data(),
        nft_marketplace::accounts::WithdrawFees {
            admin: admin.pubkey(),
            marketplace,
            treasury,
            mpl_core_program: MPL_CORE_ID,
            system_program: System::id(),
            token_program: Token::id(),
            associated_token_program: AssociatedToken::id(),
        }
        .to_account_metas(None),
    )
}
