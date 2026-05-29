use {
    litesvm::LiteSVM,
    litesvm_token::{CreateMint, MintTo, CreateAssociatedTokenAccount},
    solana_instruction::Instruction,
    solana_message::{Message, VersionedMessage},
    solana_signer::Signer,
    solana_keypair::Keypair,
    solana_transaction::versioned::VersionedTransaction,
    solana_pubkey::Pubkey,
    anchor_lang::{AccountDeserialize, Id, InstructionData, ToAccountMetas},
    mpl_core::accounts::BaseAssetV1,
};

use solana_clock::Clock;

mod ix_handlers;
use ix_handlers::*;

use nft_marketplace::state::Marketplace;
use nft_marketplace::state::Offer;


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

#[test]
fn test_delist() {
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

    let ix = delist_ix(&payer, marketplace, asset.pubkey(), listing);
    let res = send(&mut svm, &[ix], &payer, &[&payer]);
    assert!(res.is_ok(), "{:#?}", res.unwrap_err());

    let account = svm.get_account(&asset.pubkey()).unwrap();
    let asset_data = BaseAssetV1::from_bytes(&account.data).unwrap();
    assert_eq!(asset_data.owner, payer.pubkey());
}

#[test]
fn test_withdraw_fees() {
    let (
        mut svm,
        payer,
        marketplace,
        treasury,
        rewards_mint,
        _asset,
        _collection,
        _listing,
    ) = setup_listing();

    let fees: u64 = 1_000_000_000;
    svm.airdrop(&treasury, fees).unwrap();

    let admin_balance_before = svm.get_balance(&payer.pubkey()).unwrap();

    let ix = withdraw_fees_ix(&payer, marketplace, treasury);
    let res = send(&mut svm, &[ix], &payer, &[&payer]);
    assert!(res.is_ok(), "{:#?}", res.unwrap_err());

    let treasury_balance = svm.get_balance(&treasury).unwrap();
    assert_eq!(treasury_balance, 0);

    let admin_balance_after = svm.get_balance(&payer.pubkey()).unwrap();
    assert!(admin_balance_after > admin_balance_before);
}

#[test]
fn test_buy_with_token() {
    let (
        mut svm,
        payer,
        marketplace,
        treasury,
        rewards_mint,
        asset,
        _collection,
        listing,
    ) = setup_listing();

    // créer le payment mint
    let payment_mint_authority = Keypair::new();
    let payment_mint = CreateMint::new(&mut svm, &payer)
        .authority(&payment_mint_authority.pubkey())
        .decimals(6)
        .send()
        .unwrap();

    // créer les ATAs pour taker, maker (payer) et treasury
    let taker = Keypair::new();
    svm.airdrop(&taker.pubkey(), 10_000_000_000).unwrap();

    let taker_ata = CreateAssociatedTokenAccount::new(&mut svm, &taker, &payment_mint)
        .owner(&taker.pubkey())
        .send()
        .unwrap();

    let maker_ata = CreateAssociatedTokenAccount::new(&mut svm, &payer, &payment_mint)
        .owner(&payer.pubkey())
        .send()
        .unwrap();

    let treasury_ata = CreateAssociatedTokenAccount::new(&mut svm, &payer, &payment_mint)
        .owner(&treasury)
        .send()
        .unwrap();

    let taker_rewards_ata = CreateAssociatedTokenAccount::new(&mut svm, &taker, &rewards_mint)
        .owner(&taker.pubkey())
        .send()
        .unwrap();

    // mint des tokens au taker (price=100, on en mint largement plus)
    MintTo::new(&mut svm, &payer, &payment_mint, &taker_ata, 1_000_000)
        .owner(&payment_mint_authority)
        .send()
        .unwrap();

    // créer et lister l'asset avec payment_mint
    send(&mut svm, &[create_asset_ix(&payer, &asset, None)], &payer, &[&payer, &asset]).unwrap();

    let list_ix_with_token = Instruction::new_with_bytes(
        nft_marketplace::id(),
        &nft_marketplace::instruction::List {
            name: "My Marketplace".to_string(),
            price: 100,
            payment_mint: Some(payment_mint),
        }.data(),
        nft_marketplace::accounts::List {
            maker: payer.pubkey(),
            marketplace,
            asset: asset.pubkey(),
            collection: None,
            listing,
            mpl_core_program: mpl_core::ID,
            system_program: anchor_lang::prelude::System::id(),
        }.to_account_metas(None),
    );
    send(&mut svm, &[list_ix_with_token], &payer, &[&payer]).unwrap();

    // buy with token
    let ix = buy_with_token_ix(
        &taker,
        payer.pubkey(),
        marketplace,
        asset.pubkey(),
        listing,
        treasury,
        rewards_mint,
        taker_rewards_ata,
        payment_mint,
        treasury_ata,
        taker_ata,
        maker_ata,
    );

    let res = send(&mut svm, &[ix], &taker, &[&taker]);
    assert!(res.is_ok(), "{:#?}", res.unwrap_err());

    let account = svm.get_account(&asset.pubkey()).unwrap();
    let asset_data = BaseAssetV1::from_bytes(&account.data).unwrap();
    assert_eq!(asset_data.owner, taker.pubkey());
}




#[test]
fn test_make_offer() {
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


    let list_ix = list_ix(
        &payer,
        marketplace,
        asset.pubkey(),
        None,
        listing,
    );

    send(&mut svm, &[list_ix], &payer, &[&payer]).unwrap();


    let taker = Keypair::new();
     svm.airdrop(&taker.pubkey(), 10000000).unwrap();
    let program_id = nft_marketplace::id();

    let offer = Pubkey::find_program_address(
        &[b"offer", listing.as_ref(), taker.pubkey().as_ref()],
        &program_id,
    ).0;

    let offer_vault = Pubkey::find_program_address(
        &[b"offer_vault", listing.as_ref(), taker.pubkey().as_ref()], 
        &program_id,
    ).0;





    let make_offer_ix = make_offer_ix(
        &taker,
        payer.pubkey(),
        marketplace,
        asset.pubkey(),
        None,
        listing,
        treasury,
        offer,
        offer_vault,
    );
    


    let res = send(&mut svm, &[make_offer_ix], &taker, &[&taker]);
    assert!(res.is_ok(), "{:#?}", res.unwrap_err());

    let account = svm.get_account(&offer).unwrap();
    let offer_data = Offer::try_deserialize(&mut account.data.as_ref()).unwrap();
    assert_eq!(offer_data.taker, taker.pubkey());
    assert_eq!(offer_data.asset, asset.pubkey());
    assert_eq!(offer_data.offer_amount, 2);


    let offer_vault_balance = svm.get_balance(&offer_vault).unwrap();
    assert!(offer_vault_balance >= 2);
}



fn setup_offer() -> (LiteSVM, Keypair, Pubkey, Pubkey, Pubkey, Keypair, Keypair, Pubkey, Keypair, Pubkey, Pubkey) {
    let (mut svm, payer, marketplace, treasury, rewards_mint, asset, collection, listing) = setup_listing();
    let program_id = nft_marketplace::id();

    send(&mut svm, &[create_asset_ix(&payer, &asset, None)], &payer, &[&payer, &asset]).unwrap();
    send(&mut svm, &[list_ix(&payer, marketplace, asset.pubkey(), None, listing)], &payer, &[&payer]).unwrap();

    let taker = Keypair::new();
    svm.airdrop(&taker.pubkey(), 10_000_000_000).unwrap();

    let offer = Pubkey::find_program_address(
        &[b"offer", listing.as_ref(), taker.pubkey().as_ref()],
        &program_id,
    ).0;
    let offer_vault = Pubkey::find_program_address(
        &[b"offer_vault", listing.as_ref(), taker.pubkey().as_ref()],
        &program_id,
    ).0;

    send(&mut svm, &[make_offer_ix(&taker, payer.pubkey(), marketplace, asset.pubkey(), None, listing, treasury, offer, offer_vault)], &taker, &[&taker]).unwrap();

    (svm, payer, marketplace, treasury, rewards_mint, asset, collection, listing, taker, offer, offer_vault)
}

#[test]
fn test_take_offer() {
    let (mut svm, payer, marketplace, treasury, rewards_mint, asset, _collection, listing, taker, offer, offer_vault) = setup_offer();

    let taker_rewards_ata = CreateAssociatedTokenAccount::new(&mut svm, &payer, &rewards_mint)
        .owner(&taker.pubkey())
        .send()
        .unwrap();

    let ix = take_offer_ix(
        &payer,
        taker.pubkey(),
        marketplace,
        asset.pubkey(),
        listing,
        treasury,
        rewards_mint,
        taker_rewards_ata,
        offer,
        offer_vault,
    );

    let res = send(&mut svm, &[ix], &payer, &[&payer]);
    assert!(res.is_ok(), "{:#?}", res.unwrap_err());

    let account = svm.get_account(&asset.pubkey()).unwrap();
    let asset_data = BaseAssetV1::from_bytes(&account.data).unwrap();
    assert_eq!(asset_data.owner, taker.pubkey());
}

#[test]
fn test_cancel_offer() {
    let (mut svm, payer, marketplace, treasury, _rewards_mint, asset, _collection, listing, taker, offer, offer_vault) = setup_offer();

    let taker_balance_before = svm.get_balance(&taker.pubkey()).unwrap();

    let ix = cancel_offer_ix(
        &taker,
        payer.pubkey(),
        marketplace,
        asset.pubkey(),
        listing,
        treasury,
        offer,
        offer_vault,
    );

    let res = send(&mut svm, &[ix], &taker, &[&taker]);
    assert!(res.is_ok(), "{:#?}", res.unwrap_err());

    let vault_balance = svm.get_balance(&offer_vault).unwrap();
    assert_eq!(vault_balance, 0);

    let taker_balance_after = svm.get_balance(&taker.pubkey()).unwrap();
    assert!(taker_balance_after > taker_balance_before);
}
