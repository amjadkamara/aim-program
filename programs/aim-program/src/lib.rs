use anchor_lang::prelude::*;

declare_id!("AhHHJTu5vodDYE2yLNet2bE6jad9F3xSfbLQdUmykKqB");

#[program]
pub mod aim_program {
    use super::*;

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
        farmer.bump = ctx.bumps.farmer;
        msg!("Farmer ID created for: {}", farmer.full_name);
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
        let loan = &mut ctx.accounts.loan;
        loan.farmer = farmer.key();
        loan.owner = ctx.accounts.owner.key();
        loan.amount = amount;
        loan.purpose = purpose;
        loan.repayment_weeks = repayment_weeks;
        loan.is_repaid = false;
        loan.created_at = Clock::get()?.unix_timestamp;
        loan.bump = ctx.bumps.loan;
        farmer.has_active_loan = true;
        msg!("Loan requested for {} lamports", amount);
        Ok(())
    }

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

#[account]
pub struct FarmerAccount {
    pub owner: Pubkey,         // 32
    pub full_name: String,     // 4 + 64
    pub crop_type: String,     // 4 + 32
    pub district: String,      // 4 + 32
    pub farm_size: f64,        // 8
    pub has_active_loan: bool, // 1
    pub created_at: i64,       // 8
    pub bump: u8,              // 1
}

#[account]
pub struct LoanAccount {
    pub farmer: Pubkey,      // 32
    pub owner: Pubkey,       // 32
    pub amount: u64,         // 8
    pub purpose: String,     // 4 + 64
    pub repayment_weeks: u8, // 1
    pub is_repaid: bool,     // 1
    pub created_at: i64,     // 8
    pub bump: u8,            // 1
}

#[derive(Accounts)]
pub struct CreateFarmerID<'info> {
    #[account(
        init,
        payer = owner,
        space = 8 + 32 + 68 + 36 + 36 + 8 + 1 + 8 + 1,
        seeds = [b"farmer", owner.key().as_ref()],
        bump
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
        space = 8 + 32 + 32 + 8 + 68 + 1 + 1 + 8 + 1,
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

    #[account(mut)]
    pub owner: Signer<'info>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct RepayLoan<'info> {
    #[account(
        mut,
        seeds = [b"loan", owner.key().as_ref()],
        bump = loan.bump,
        constraint = loan.owner == owner.key() @ AimError::LoanAlreadyRepaid
    )]
    pub loan: Account<'info, LoanAccount>,

    #[account(
        mut,
        seeds = [b"farmer", owner.key().as_ref()],
        bump = farmer.bump,
        has_one = owner
    )]
    pub farmer: Account<'info, FarmerAccount>,

    pub owner: Signer<'info>,
}

#[error_code]
pub enum AimError {
    #[msg("Farmer already has an active loan")]
    ActiveLoanExists,
    #[msg("Loan has already been repaid")]
    LoanAlreadyRepaid,
}
