use {
    anchor_spl::associated_token::{self, get_associated_token_address},
    litesvm::LiteSVM,
    litesvm_token::CreateMint,
    litesvm_token::MintTo,
    litesvm_token::Transfer,
    litesvm_token::CreateAssociatedTokenAccount,
    solana_instruction::Instruction,
    solana_message::{Message, VersionedMessage},
    solana_signer::Signer,
    solana_keypair::Keypair,
    solana_transaction::versioned::VersionedTransaction,
    solana_pubkey::Pubkey,
    anchor_lang::AccountDeserialize,
    anchor_spl::token::TokenAccount,
    mpl_core::{
        accounts::{BaseCollectionV1,BaseAssetV1},
        fetch_plugin,
        types::{Attributes, FreezeDelegate, PluginType},
    },
    solana_account_info::AccountInfo,
};

use solana_clock::Clock;

mod ix_handlers;
use ix_handlers::*;

use nft_marketplace::state::Marketplace;


fn send(
    svm: &mut LiteSVM,
    ixs:&[Instruction],
    payer: &Keypair,
    signers: &[&dyn Signer]
) -> litesvm::types::TransactionResult {
    svm.expire_blockhash();
    let blockhash = svm.latest_blockhash();
    let msg = Message::new_with_blockhash(ixs, Some(&payer.pubkey()), &blockhash);
    let tx = VersionedTransaction::try_new(VersionedMessage::Legacy(msg), signers).unwrap();
    svm.send_transaction(tx)
}

fn setup() -> (
    LiteSVM,
    Keypair,
    Pubkey,
    Pubkey,
    Pubkey,
) {
    let program_id = nft_marketplace::id();
    let payer = Keypair::new();
    let mut svm = LiteSVM::new();
    let bytes = include_bytes!("../../../target/deploy/nft_marketplace.so");
    let mpl_core_bytes = include_bytes!("fixtures/mpl_core.so");
    svm.add_program(mpl_core::ID, mpl_core_bytes);
    svm.add_program(program_id, bytes);
    svm.airdrop(&payer.pubkey(), 10_000_000_000).unwrap();


    let name = "My Marketplace".to_string();
    let marketplace = Pubkey::find_program_address(
        &[b"marketplace", name.as_bytes()],
        &program_id,
    ).0;

    let treasury = Pubkey::find_program_address(
        &[b"treasury", marketplace.as_ref()],
        &program_id,
    ).0;

    let rewards_mint = Pubkey::find_program_address(
        &[b"reward_mint", marketplace.as_ref()],
        &program_id,
    ).0;

    (
        svm,
        payer,
        marketplace,
        treasury,
        rewards_mint
    )

}


#[test]

fn test_initialize() {
    let (
        mut svm,
        payer,
        marketplace,
        treasury,
        rewards_mint
    ) = setup();

    let ix = initialize_ix(
        &payer,
        marketplace,
        treasury,
        rewards_mint
    );

    let res = send(&mut svm, &[ix], &payer, &[&payer]);
    assert!(res.is_ok());



    let account = svm.get_account(&marketplace).unwrap();
    let marketplace_data = Marketplace::try_deserialize(&mut account.data.as_ref()).unwrap();
    assert_eq!(marketplace_data.name, "My Marketplace".to_string());
    assert_eq!(marketplace_data.fee_bps, 250);
    assert_eq!(marketplace_data.admin, payer.pubkey());

}




fn setup_listing() -> (LiteSVM, Keypair, Pubkey, Pubkey, Pubkey, Keypair, Keypair, Pubkey) {
    let (mut svm, payer, marketplace, treasury, rewards_mint) = setup();
    let program_id = nft_marketplace::id();

    let ix = initialize_ix(&payer, marketplace, treasury, rewards_mint);
    send(&mut svm, &[ix], &payer, &[&payer]).unwrap();

    let asset = Keypair::new();
    let collection = Keypair::new();
    let listing = Pubkey::find_program_address(
        &[b"listing", asset.pubkey().as_ref()],
        &program_id,
    ).0;

    (svm, payer, marketplace, treasury, rewards_mint, asset, collection, listing)
}

#[test]
fn test_list() {
    let (
        mut svm,
        payer,
        marketplace,
        treasury,
        rewards_mint,
        asset,
        collection,
        listing,
    ) = setup_listing();

    let create_ix = create_asset_ix(&payer, &asset, None);
    send(&mut svm, &[create_ix], &payer, &[&payer, &asset]).unwrap();

    let ix = list_ix(
        &payer,
        marketplace,
        asset.pubkey(),
        None,
        listing,
    );

    let res = send(&mut svm, &[ix], &payer, &[&payer]);
    assert!(res.is_ok(), "{:#?}", res.unwrap_err());


    let account = svm.get_account(&listing).unwrap();
    let listing_data = nft_marketplace::state::Listing::try_deserialize(&mut account.data.as_ref()).unwrap();
    assert_eq!(listing_data.maker, payer.pubkey());
    assert_eq!(listing_data.asset, asset.pubkey());
    assert_eq!(listing_data.price, 100);
    assert_eq!(listing_data.payment_mint, None);

    let account = svm.get_account(&asset.pubkey()).unwrap();
    let asset_data = BaseAssetV1::from_bytes(&account.data).unwrap();
    assert_eq!(asset_data.owner, listing);

}




#[test]
fn test_buy() {
    let (
        mut svm,
        payer,
        marketplace,
        treasury,
        rewards_mint,
        asset,
        collection,
        listing,
    ) = setup_listing();

    send(&mut svm, &[create_asset_ix(&payer, &asset, None)], &payer, &[&payer, &asset]).unwrap();
    send(&mut svm, &[list_ix(&payer, marketplace, asset.pubkey(), None, listing)], &payer, &[&payer]).unwrap();

    let taker = Keypair::new();
    svm.airdrop(&taker.pubkey(), 10_000_000_000).unwrap();

    let taker_rewards_ata = CreateAssociatedTokenAccount::new(&mut svm, &taker, &rewards_mint)
        .owner(&taker.pubkey())
        .send()
        .unwrap();

    let ix = buy_ix(
        &taker,
        payer.pubkey(),
        marketplace,
        asset.pubkey(),
        None,
        listing,
        treasury,
        rewards_mint,
        taker_rewards_ata,
    );

    let res = send(&mut svm, &[ix], &taker, &[&taker]);
    assert!(res.is_ok(), "{:#?}", res.unwrap_err());

    let account = svm.get_account(&asset.pubkey()).unwrap();
    let asset_data = BaseAssetV1::from_bytes(&account.data).unwrap();
    assert_eq!(asset_data.owner, taker.pubkey());
}

