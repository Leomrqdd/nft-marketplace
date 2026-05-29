# NFT Marketplace

Anchor program on Solana for listing, buying, and making offers on mpl-core NFTs.

## Features

- List / delist mpl-core assets
- Buy with SOL or SPL tokens
- Make / take / cancel offers
- Reward token minted to buyer on each purchase
- Fee collection and admin withdrawal

## Program Instructions

| Instruction | Description |
|---|---|
| `initialize` | Create a marketplace with fee (bps) and reward mint |
| `list` | Transfer asset to listing PDA and set price |
| `delist` | Return asset to maker and close listing |
| `buy` | Buy with SOL — fees go to treasury, reward token minted to buyer |
| `buy_with_tokens` | Buy with SPL token — same fee/reward logic |
| `make_offer` | Lock SOL in offer vault |
| `take_offer` | Maker accepts offer — SOL released, asset transferred |
| `cancel_offer` | Taker cancels — SOL returned from vault |
| `withdraw_fees` | Admin withdraws accumulated SOL from treasury |

## Accounts (PDAs)

| Account | Seeds |
|---|---|
| Marketplace | `["marketplace", name]` |
| Treasury | `["treasury", marketplace]` |
| Reward mint | `["reward_mint", marketplace]` |
| Listing | `["listing", asset]` |
| Offer | `["offer", listing, taker]` |
| Offer vault | `["offer_vault", listing, taker]` |

## Getting Started

```bash
# Install dependencies
yarn

# Build
anchor build

# Run tests (litesvm, no validator required)
cargo test -p nft-marketplace
```

## Tests

Integration tests use [litesvm](https://github.com/LiteSVM/litesvm) — fast, no local validator needed.

| Test | What it covers |
|---|---|
| `test_initialize` | Marketplace creation |
| `test_list` | Asset transfer to listing PDA |
| `test_delist` | Asset returned to maker |
| `test_buy` | SOL purchase + reward mint |
| `test_buy_with_token` | SPL token purchase |
| `test_make_offer` | Offer creation and vault funding |
| `test_take_offer` | Offer acceptance and asset transfer |
| `test_cancel_offer` | Offer cancellation and vault refund |
| `test_withdraw_fees` | Admin treasury withdrawal |

## Stack

- [Anchor](https://www.anchor-lang.com/) 0.31
- [mpl-core](https://developers.metaplex.com/core) 0.11
- [litesvm](https://github.com/LiteSVM/litesvm) 0.6 (tests)
- Rust 1.89
