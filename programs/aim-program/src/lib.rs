use anchor_lang::prelude::*;

declare_id!("AhHHJTu5vodDYE2yLNet2bE6jad9F3xSfbLQdUmykKqB");

// Hardcoded admin wallet — same address used in frontend ADMIN_WALLETS.
// This wallet is permanently blocked from registering as a Farmer or Lender.
pub const ADMIN_PUBKEY: Pubkey = pubkey!("Cz3GvsRaBsuAHoRiJd5sV6ZTAkE8TsFJAyuYWEtV7Qu2");

#[program]
pub mod aim_program {
    use super::*;

    pub fn create_farmer_id(
        ctx: Context<CreateFarmerID>,
        full_name: String,
        crop_type: String,
        district: String,
        country: String,
        phone_number: String,
        farm_size: f64,
    ) -> Result<()> {
        require!(
            ctx.accounts.owner.key() != ADMIN_PUBKEY,
            AimError::AdminCannotRegister
        );

        // Cross-role check: this wallet must not already be a registered Lender.
        // The lender_check account is passed in but never initialized here —
        // if it already exists on-chain (data_is_empty() == false), block registration.
        require!(
            ctx.accounts.lender_check.data_is_empty(),
            AimError::WalletAlreadyHasRole
        );

        let farmer = &mut ctx.accounts.farmer;
        farmer.owner = ctx.accounts.owner.key();
        farmer.full_name = full_name;
        farmer.crop_type = crop_type;
        farmer.district = district;
        farmer.country = country;
        farmer.phone_number = phone_number;
        farmer.farm_size = farm_size;
        farmer.has_active_loan = false;
        farmer.loan_counter = 0;
        farmer.credit_score = 0;
        farmer.created_at = Clock::get()?.unix_timestamp;
        farmer.bump = ctx.bumps.farmer;
        msg!("Farmer ID created for: {}", farmer.full_name);
        Ok(())
    }

    pub fn register_lender(
        ctx: Context<RegisterLender>,
        name: String,
        org_type: String,
        country: String,
        city: String,
        email: String,
        max_loan_lamports: u64,
        interest_rate_bps: u16,
        max_duration_weeks: u8,
        min_credit_score: u64,
        capital_budget_lamports: u64,
    ) -> Result<()> {
        require!(
            ctx.accounts.owner.key() != ADMIN_PUBKEY,
            AimError::AdminCannotRegister
        );

        // Cross-role check: this wallet must not already be a registered Farmer.
        require!(
            ctx.accounts.farmer_check.data_is_empty(),
            AimError::WalletAlreadyHasRole
        );

        let lender = &mut ctx.accounts.lender;
        lender.owner = ctx.accounts.owner.key();
        lender.name = name;
        lender.org_type = org_type;
        lender.country = country;
        lender.city = city;
        lender.email = email;
        lender.max_loan_lamports = max_loan_lamports;
        lender.interest_rate_bps = interest_rate_bps;
        lender.max_duration_weeks = max_duration_weeks;
        lender.min_credit_score = min_credit_score;
        lender.capital_budget_lamports = capital_budget_lamports;
        lender.is_active = false; // Requires admin approval before going live
        lender.created_at = Clock::get()?.unix_timestamp;
        lender.bump = ctx.bumps.lender;
        msg!("Lender registered (pending approval): {}", lender.name);
        Ok(())
    }

    pub fn approve_lender(ctx: Context<ApproveLender>) -> Result<()> {
        require!(
            ctx.accounts.admin.key() == ADMIN_PUBKEY,
            AimError::Unauthorized
        );

        ctx.accounts.lender.is_active = true;
        msg!("Lender approved: {}", ctx.accounts.lender.name);
        Ok(())
    }

    pub fn request_loan(
        ctx: Context<RequestLoan>,
        amount: u64,
        purpose: String,
        repayment_weeks: u8,
    ) -> Result<()> {
        let farmer = &mut ctx.accounts.farmer;
        require!(!farmer.has_active_loan, AimError::ActiveLoanExists);
        require!(ctx.accounts.lender.is_active, AimError::LenderNotActive);

        let loan = &mut ctx.accounts.loan;
        loan.farmer = farmer.key();
        loan.owner = ctx.accounts.owner.key();
        loan.lender = ctx.accounts.lender.key();
        loan.amount = amount;
        loan.purpose = purpose;
        loan.repayment_weeks = repayment_weeks;
        loan.is_repaid = false;
        loan.created_at = Clock::get()?.unix_timestamp;
        loan.bump = ctx.bumps.loan;

        farmer.has_active_loan = true;
        farmer.loan_counter = farmer.loan_counter.saturating_add(1);

        msg!(
            "Loan requested for {} lamports from lender {}",
            amount,
            loan.lender
        );
        Ok(())
    }

    pub fn repay_loan(ctx: Context<RepayLoan>) -> Result<()> {
        require!(!ctx.accounts.loan.is_repaid, AimError::LoanAlreadyRepaid);

        // Mark the loan repaid instead of silently leaving is_repaid false.
        // The loan account is intentionally NOT closed here anymore (see
        // RepayLoan account context below) — it stays on-chain, flagged
        // repaid, so admin dashboards and loan history can see it. The
        // separate close_loan instruction (farmer-initiated, requires
        // is_repaid == true) is what actually reclaims the rent, matching
        // the "Clear repaid loan account to re-borrow" flow in the frontend.
        ctx.accounts.loan.is_repaid = true;
        ctx.accounts.farmer.has_active_loan = false;
        ctx.accounts.farmer.credit_score = ctx.accounts.farmer.credit_score.saturating_add(10);

        msg!(
            "Loan repaid successfully. New credit score: {}",
            ctx.accounts.farmer.credit_score
        );
        Ok(())
    }

    pub fn close_loan(ctx: Context<CloseLoan>) -> Result<()> {
        require!(ctx.accounts.loan.is_repaid, AimError::LoanNotRepaid);
        msg!("Loan account closed");
        Ok(())
    }
}

#[account]
pub struct FarmerAccount {
    pub owner: Pubkey,         // 32
    pub full_name: String,     // 4 + 64
    pub crop_type: String,     // 4 + 32
    pub district: String,      // 4 + 32
    pub country: String,       // 4 + 40
    pub phone_number: String,  // 4 + 20
    pub farm_size: f64,        // 8
    pub has_active_loan: bool, // 1
    pub loan_counter: u64,     // 8
    pub credit_score: u64,     // 8
    pub created_at: i64,       // 8
    pub bump: u8,              // 1
}

#[account]
pub struct LoanAccount {
    pub farmer: Pubkey,      // 32
    pub owner: Pubkey,       // 32
    pub lender: Pubkey,      // 32
    pub amount: u64,         // 8
    pub purpose: String,     // 4 + 64
    pub repayment_weeks: u8, // 1
    pub is_repaid: bool,     // 1
    pub created_at: i64,     // 8
    pub bump: u8,            // 1
}

#[account]
pub struct LenderAccount {
    pub owner: Pubkey,                // 32
    pub name: String,                 // 4 + 64
    pub org_type: String,             // 4 + 48
    pub country: String,              // 4 + 40
    pub city: String,                 // 4 + 32
    pub email: String,                // 4 + 64
    pub max_loan_lamports: u64,       // 8 — stored in lamports for exact precision
    pub interest_rate_bps: u16,       // 2
    pub max_duration_weeks: u8,       // 1
    pub min_credit_score: u64,        // 8
    pub capital_budget_lamports: u64, // 8 — stored in lamports for exact precision
    pub is_active: bool,              // 1
    pub created_at: i64,              // 8
    pub bump: u8,                     // 1
}

#[derive(Accounts)]
pub struct CreateFarmerID<'info> {
    #[account(
        init,
        payer = owner,
        space = 8 + 32 + 68 + 36 + 36 + 44 + 24 + 8 + 1 + 8 + 8 + 8 + 1,
        seeds = [b"farmer", owner.key().as_ref()],
        bump
    )]
    pub farmer: Account<'info, FarmerAccount>,

    /// CHECK: only read via data_is_empty() to confirm no LenderAccount exists for this wallet.
    /// Never deserialized, never written to.
    #[account(
        seeds = [b"lender", owner.key().as_ref()],
        bump
    )]
    pub lender_check: UncheckedAccount<'info>,

    #[account(mut)]
    pub owner: Signer<'info>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct RegisterLender<'info> {
    #[account(
        init,
        payer = owner,
        space = 8 + 32 + 68 + 52 + 44 + 36 + 68 + 8 + 2 + 1 + 8 + 8 + 1 + 8 + 1,
        seeds = [b"lender", owner.key().as_ref()],
        bump
    )]
    pub lender: Account<'info, LenderAccount>,

    /// CHECK: only read via data_is_empty() to confirm no FarmerAccount exists for this wallet.
    /// Never deserialized, never written to.
    #[account(
        seeds = [b"farmer", owner.key().as_ref()],
        bump
    )]
    pub farmer_check: UncheckedAccount<'info>,

    #[account(mut)]
    pub owner: Signer<'info>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct ApproveLender<'info> {
    #[account(mut)]
    pub lender: Account<'info, LenderAccount>,

    pub admin: Signer<'info>,
}

#[derive(Accounts)]
pub struct RequestLoan<'info> {
    #[account(
        init,
        payer = owner,
        space = 8 + 32 + 32 + 32 + 8 + 68 + 1 + 1 + 8 + 1,
        seeds = [b"loan", owner.key().as_ref()],
        bump
    )]
    pub loan: Account<'info, LoanAccount>,

    #[account(
        mut,
        seeds = [b"farmer", owner.key().as_ref()],
        bump = farmer.bump,
        has_one = owner
    )]
    pub farmer: Account<'info, FarmerAccount>,

    pub lender: Account<'info, LenderAccount>,

    #[account(mut)]
    pub owner: Signer<'info>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct RepayLoan<'info> {
    // NOTE: `close = owner` intentionally removed. Repaying a loan should
    // flag it repaid and leave it on-chain — closing it here (as before)
    // destroyed the record before admin dashboards or loan history could
    // ever see it as "Repaid," and made the separate close_loan instruction
    // unreachable dead code. The farmer now clears the account explicitly
    // via close_loan (see CloseLoan below) once they're ready to re-borrow.
    #[account(
        mut,
        seeds = [b"loan", owner.key().as_ref()],
        bump = loan.bump,
        constraint = loan.owner == owner.key() @ AimError::LoanAlreadyRepaid,
    )]
    pub loan: Account<'info, LoanAccount>,

    #[account(
        mut,
        seeds = [b"farmer", owner.key().as_ref()],
        bump = farmer.bump,
        has_one = owner
    )]
    pub farmer: Account<'info, FarmerAccount>,

    #[account(mut)]
    pub owner: Signer<'info>,
}

#[derive(Accounts)]
pub struct CloseLoan<'info> {
    #[account(
        mut,
        seeds = [b"loan", owner.key().as_ref()],
        bump = loan.bump,
        constraint = loan.owner == owner.key() @ AimError::LoanAlreadyRepaid,
        close = owner
    )]
    pub loan: Account<'info, LoanAccount>,

    #[account(mut)]
    pub owner: Signer<'info>,
}

#[error_code]
pub enum AimError {
    #[msg("Farmer already has an active loan")]
    ActiveLoanExists,
    #[msg("Loan has already been repaid")]
    LoanAlreadyRepaid,
    #[msg("Loan has not been repaid yet")]
    LoanNotRepaid,
    #[msg("Admin wallet cannot register as a Farmer or Lender")]
    AdminCannotRegister,
    #[msg("Only the admin wallet can perform this action")]
    Unauthorized,
    #[msg("This lender is not yet approved by admin")]
    LenderNotActive,
    #[msg("This wallet already holds a different role on the protocol")]
    WalletAlreadyHasRole,
}
