# AIM Protocol — Solana Smart Contract

> On-chain Anchor program powering the AIM Protocol microfinance platform.

[![Devnet](https://img.shields.io/badge/Network-Solana%20Devnet-purple)](https://explorer.solana.com/address/AhHHJTu5vodDYE2yLNet2bE6jad9F3xSfbLQdUmykKqB?cluster=devnet)
[![Anchor](https://img.shields.io/badge/Anchor-0.31.1-blue)](https://www.anchor-lang.com/)
[![Tests](https://img.shields.io/badge/Tests-5%2F5%20passing-green)](./tests/aim-program.ts)

## Overview

This is the Anchor smart contract for AIM Protocol — a decentralized microfinance platform connecting smallholder farmers across Africa to DeFi financial services. One wallet = one Farmer ID, enforced on-chain via Program Derived Addresses.

**Frontend repo:** [github.com/amjadkamara/aim-protocol](https://github.com/amjadkamara/aim-protocol)  
**Live app:** [aim-protocol.vercel.app](https://aim-protocol.vercel.app/)

## Program ID

`AhHHJTu5vodDYE2yLNet2bE6jad9F3xSfbLQdUmykKqB` — deployed on Solana Devnet

## Instructions

### `create_farmer_id`

Creates a verified on-chain farmer identity at a PDA derived from the wallet.  
Seeds: `["farmer", owner_pubkey]` — guarantees one Farmer ID per wallet.

| Argument    | Type   | Description                       |
| ----------- | ------ | --------------------------------- |
| `full_name` | String | Farmer's full name (max 50 chars) |
| `crop_type` | String | Primary crop being farmed         |
| `district`  | String | District location                 |
| `farm_size` | f64    | Farm size in acres                |

### `request_loan`

Requests a crop-backed microloan. Requires an existing Farmer ID. Blocked if farmer already has an active loan.  
Seeds: `["loan", owner_pubkey]`

| Argument          | Type   | Description               |
| ----------------- | ------ | ------------------------- |
| `amount`          | u64    | Loan amount in lamports   |
| `purpose`         | String | Loan purpose              |
| `repayment_weeks` | u8     | Repayment period in weeks |

### `repay_loan`

Repays the active loan. Closes the loan account on repayment (`close = owner`), returning rent SOL to the farmer and freeing the PDA for a new loan.

### `close_loan`

Closes a repaid legacy loan account. Used for migration of accounts created before V2.1.

## Account Structures

### `FarmerAccount`

| Field             | Type   | Description                       |
| ----------------- | ------ | --------------------------------- |
| `owner`           | Pubkey | Wallet that owns this farmer ID   |
| `full_name`       | String | Farmer's name                     |
| `crop_type`       | String | Primary crop                      |
| `district`        | String | District location                 |
| `farm_size`       | f64    | Acres                             |
| `has_active_loan` | bool   | Whether farmer has an active loan |
| `created_at`      | i64    | Unix timestamp of registration    |
| `bump`            | u8     | PDA bump seed                     |

### `LoanAccount`

| Field             | Type   | Description                |
| ----------------- | ------ | -------------------------- |
| `farmer`          | Pubkey | Farmer account PDA         |
| `owner`           | Pubkey | Wallet that owns this loan |
| `amount`          | u64    | Loan amount in lamports    |
| `purpose`         | String | Loan purpose               |
| `repayment_weeks` | u8     | Repayment period           |
| `is_repaid`       | bool   | Repayment status           |
| `created_at`      | i64    | Unix timestamp of loan     |
| `bump`            | u8     | PDA bump seed              |

## PDA Derivation

```rust
// Farmer ID — one per wallet
let farmer_pda = PublicKey.findProgramAddressSync(
  [Buffer.from("farmer"), walletPublicKey.toBuffer()],
  PROGRAM_ID
);

// Loan — one active loan per wallet
let loan_pda = PublicKey.findProgramAddressSync(
  [Buffer.from("loan"), walletPublicKey.toBuffer()],
  PROGRAM_ID
);
```

## Custom Errors

| Error               | Code | Message                           |
| ------------------- | ---- | --------------------------------- |
| `ActiveLoanExists`  | 6000 | Farmer already has an active loan |
| `LoanAlreadyRepaid` | 6001 | Loan has already been repaid      |
| `LoanNotRepaid`     | 6002 | Loan has not been repaid yet      |

## Running Tests

```bash
# Install dependencies
npm install

# Run full test suite against devnet
anchor test
```

Expected output:

```
✅ Farmer ID created at PDA
✔ Creates a farmer ID
✅ Duplicate blocked as expected
✔ Blocks duplicate farmer registration
✅ Loan requested successfully
✔ Requests a loan
✅ Second loan blocked as expected
✔ Blocks second loan while one is active
✅ Loan repaid and account closed successfully
✔ Repays the loan

5 passing
```

## Deploying

```bash
# Build
anchor build

# Deploy to devnet
anchor deploy --provider.cluster devnet

# Copy updated IDL to frontend
cp target/idl/aim_program.json ../aim-protocol/src/idl/aim_program.json
```

## Tech Stack

- Rust
- Anchor Framework 0.31.1
- Solana CLI 3.1.13

## Built By

**Amjad Kamara** — Founder, Aadios Systems (SL) Ltd.  
Sierra Leone 🇸🇱 • Building for Africa on Solana 🌍

## License

MIT
