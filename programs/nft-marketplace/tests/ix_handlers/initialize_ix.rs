use {
    anchor_lang::{Id, InstructionData, ToAccountMetas, prelude::System}, anchor_spl::{associated_token::AssociatedToken, mint, token::Token}, solana_keypair::Keypair, solana_instruction::Instruction, solana_pubkey::Pubkey, solana_signer::Signer
};

pub fn initialize_ix(
    admin:&Keypair,
    marketplace: Pubkey,
    treasury: Pubkey,
    rewards_mint: Pubkey,
) -> Instruction {
    
    Instruction::new_with_bytes(
        nft_marketplace::id(),
        &nft_marketplace::instruction::Initialize {
            fee_bps: 250,
            name: "My Marketplace".to_string(),
        }
        .data(),
        nft_marketplace::accounts::Initialize {
            admin: admin.pubkey(),
            marketplace,
            treasury,
            rewards_mint,
            system_program: System::id(),
            token_program: Token::id(),

        }
        .to_account_metas(None),

    )
}


