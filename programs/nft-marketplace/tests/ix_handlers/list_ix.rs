use {
    anchor_lang::{Id, InstructionData, ToAccountMetas, prelude::System},
    mpl_core::ID as MPL_CORE_ID,
    solana_keypair::Keypair,
    solana_instruction::Instruction,
    solana_pubkey::Pubkey,
    solana_signer::Signer,
};

pub fn list_ix(
    maker: &Keypair,
    marketplace: Pubkey,
    asset: Pubkey,
    collection: Option<Pubkey>,
    listing: Pubkey,
) -> Instruction {
    Instruction::new_with_bytes(
        nft_marketplace::id(),
        &nft_marketplace::instruction::List { name: "My Marketplace".to_string(), price: 100, payment_mint: None }.data(),
        nft_marketplace::accounts::List {
            maker: maker.pubkey(),
            marketplace,
            asset,
            collection,
            listing,
            mpl_core_program: MPL_CORE_ID,
            system_program: System::id(),
        }
        .to_account_metas(None),
    )
}


