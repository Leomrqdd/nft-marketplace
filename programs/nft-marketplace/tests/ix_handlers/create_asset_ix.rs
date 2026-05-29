use {
    anchor_lang::{Id, prelude::System},
    mpl_core::instructions::CreateV2Builder,
    solana_instruction::Instruction,
    solana_keypair::Keypair,
    solana_pubkey::Pubkey,
    solana_signer::Signer,
};

pub fn create_asset_ix(
    payer: &Keypair,
    asset: &Keypair,
    collection: Option<Pubkey>,
) -> Instruction {
    CreateV2Builder::new()
        .asset(asset.pubkey())
        .collection(collection)
        .payer(payer.pubkey())
        .owner(Some(payer.pubkey()))
        .authority(Some(payer.pubkey()))
        .system_program(System::id())
        .name("Test Asset".to_string())
        .uri("https://example.com".to_string())
        .instruction()
}
