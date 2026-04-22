# AIM Protocol — Solana Smart Contract

> On-chain Anchor program powering the AIM Protocol microfinance platform.

## Overview

This is the Anchor smart contract (program) for AIM Protocol — a decentralized microfinance platform connecting smallholder farmers in Sierra Leone and across Africa to DeFi financial services.

## Program ID

`AhHHJTu5vodDYE2yLNet2bE6jad9F3xSfbLQdUmykKqB` — deployed on Solana Devnet

## Instructions

### create_farmer_id
Creates a verified on-chain farmer identity account.
- `full_name` — farmer's full name
- `crop_type` — primary crop being farmed
- `district` — district in Sierra Leone
- `farm_size` — farm size in acres

### request_loan
Requests a simulated crop-backed microloan. Requires an existing Farmer ID. Blocked if farmer has an active loan.
- `amount` — loan amount in lamports
- `purpose` — loan purpose
- `repayment_weeks` — repayment period in weeks

### repay_loan
Marks a loan as repaid and clears the farmer's active loan status.

## Tech Stack

- Rust
- Anchor Framework 0.31.1
- Solana CLI 3.1.13

## Frontend

The React frontend that interacts with this program lives at:
https://github.com/amjadkamara/aim-protocol

## Live Demo

https://aim-protocol.vercel.app/

## Built By

Amjad Kamara — Founder, Aadios Systems (SL) Ltd.
Sierra Leone

## License

MIT
