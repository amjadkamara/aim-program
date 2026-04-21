use anchor_lang::prelude::*;

declare_id!("AhHHJTu5vodDYE2yLNet2bE6jad9F3xSfbLQdUmykKqB");

#[program]
pub mod aim_program {
    use super::*;

    // Create a new farmer identity on-chain
    pub fn create_farmer_id(
        ctx: Context<CreateFarmerID>,
        full_name: String,
        crop_type: String,
        district: String,
        farm_size: f64,
    ) -> Result<()> {
        let farmer = &mut ctx.accounts.farmer;
        farmer.owner = ctx.accounts.owner.key();
        farmer.full_name = full_name;
        farmer.crop_type = crop_type;
        farmer.district = district;
        farmer.farm_size = farm_size;
        farmer.has_active_loan = false;
        farmer.created_at = Clock::get()?.unix_timestamp;
        msg!("Farmer ID created for: {}", farmer.full_name);
        Ok(())
    }

    // Request a microloan — only if farmer has no active loan
    pub fn request_loan(
        ctx: Context<RequestLoan>,
        amount: u64,
        purpose: String,
        repayment_weeks: u8,
    ) -> Result<()> {
        let farmer = &mut ctx.accounts.farmer;

        // Check farmer has no active loan
        require!(!farmer.has_active_loan, AimError::ActiveLoanExists);

        let loan = &mut ctx.accounts.loan;
        loan.farmer = farmer.key();
        loan.owner = ctx.accounts.owner.key();
        loan.amount = amount;
        loan.purpose = purpose;
        loan.repayment_weeks = repayment_weeks;
        loan.is_repaid = false;
        loan.created_at = Clock::get()?.unix_timestamp;

        // Mark farmer as having active loan
        farmer.has_active_loan = true;

        msg!("Loan approved for {} lamports", amount);
        Ok(())
    }

    // Repay a loan
    pub fn repay_loan(ctx: Context<RepayLoan>) -> Result<()> {
        let loan = &mut ctx.accounts.loan;
        let farmer = &mut ctx.accounts.farmer;

        require!(!loan.is_repaid, AimError::LoanAlreadyRepaid);

        loan.is_repaid = true;
        farmer.has_active_loan = false;

        msg!("Loan repaid successfully");
        Ok(())
    }
}

// Farmer ID account structure
#[account]
pub struct FarmerAccount {
    pub owner: Pubkey,
    pub full_name: String,
    pub crop_type: String,
    pub district: String,
    pub farm_size: f64,
    pub has_active_loan: bool,
    pub created_at: i64,
}

// Loan account structure
#[account]
pub struct LoanAccount {
    pub farmer: Pubkey,
    pub owner: Pubkey,
    pub amount: u64,
    pub purpose: String,
    pub repayment_weeks: u8,
    pub is_repaid: bool,
    pub created_at: i64,
}

// Account contexts
#[derive(Accounts)]
pub struct CreateFarmerID<'info> {
    #[account(
        init,
        payer = owner,
        space = 8 + 32 + 64 + 32 + 32 + 8 + 1 + 8
    )]
    pub farmer: Account<'info, FarmerAccount>,
    #[account(mut)]
    pub owner: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct RequestLoan<'info> {
    #[account(
        init,
        payer = owner,
        space = 8 + 32 + 32 + 8 + 64 + 1 + 1 + 8
    )]
    pub loan: Account<'info, LoanAccount>,
    #[account(mut, has_one = owner)]
    pub farmer: Account<'info, FarmerAccount>,
    #[account(mut)]
    pub owner: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct RepayLoan<'info> {
    #[account(mut)]
    pub loan: Account<'info, LoanAccount>,
    #[account(mut, has_one = owner)]
    pub farmer: Account<'info, FarmerAccount>,
    pub owner: Signer<'info>,
}

// Custom errors
#[error_code]
pub enum AimError {
    #[msg("Farmer already has an active loan")]
    ActiveLoanExists,
    #[msg("Loan has already been repaid")]
    LoanAlreadyRepaid,
}
