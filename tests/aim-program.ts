import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { AimProgram } from "../target/types/aim_program";
import { assert } from "chai";

describe("aim-program", () => {
  anchor.setProvider(anchor.AnchorProvider.env());
  const program = anchor.workspace.AimProgram as Program<AimProgram>;
  const provider = anchor.getProvider() as anchor.AnchorProvider;
  const owner = provider.wallet.publicKey;

  const [farmerPDA] = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from("farmer"), owner.toBuffer()],
    program.programId
  );
  const [loanPDA] = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from("loan"), owner.toBuffer()],
    program.programId
  );
  const [lenderPDA] = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from("lender"), owner.toBuffer()],
    program.programId
  );

  // A second wallet used to test the lender role independently of the farmer wallet
  const lenderWallet = anchor.web3.Keypair.generate();
  const [lenderPDA2] = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from("lender"), lenderWallet.publicKey.toBuffer()],
    program.programId
  );
  const [farmerCheckForLenderWallet] = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from("farmer"), lenderWallet.publicKey.toBuffer()],
    program.programId
  );

  // Capital budget for lenderPDA2. Was previously 500 lamports — a
  // leftover placeholder that happened not to matter because nothing
  // enforced it. Now that request_loan/repay_loan actually move this
  // counter, it needs to be large enough to plausibly cover the loan
  // amounts used throughout this suite. Chosen so the numbers stay easy
  // to eyeball: 3,000,000 lamports budget, 1,000,000 lamport first loan.
  const LENDER_CAPITAL_BUDGET = 3_000_000;

  it("Airdrops SOL to the second test wallet", async () => {
    const sig = await provider.connection.requestAirdrop(
      lenderWallet.publicKey,
      anchor.web3.LAMPORTS_PER_SOL
    );
    await provider.connection.confirmTransaction(sig);
    console.log("✅ Airdropped 1 SOL to lender test wallet:", lenderWallet.publicKey.toBase58());
  });

  it("Creates a farmer ID", async () => {
    await program.methods
      .createFarmerId("Amjad Kamara", "Rice", "Bo", "Sierra Leone", "+232 76123456", 2.5)
      .accounts({
        farmer: farmerPDA,
        lenderCheck: lenderPDA,
        owner: owner,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .rpc();

    const farmer = await program.account.farmerAccount.fetch(farmerPDA);
    assert.equal(farmer.fullName, "Amjad Kamara");
    assert.equal(farmer.cropType, "Rice");
    assert.equal(farmer.district, "Bo");
    assert.equal(farmer.country, "Sierra Leone");
    assert.equal(farmer.phoneNumber, "+232 76123456");
    assert.equal(farmer.farmSize, 2.5);
    assert.equal(farmer.hasActiveLoan, false);
    assert.equal(farmer.loanCounter.toNumber(), 0);
    assert.equal(farmer.creditScore.toNumber(), 0);
    assert.equal(farmer.owner.toBase58(), owner.toBase58());
    console.log("✅ Farmer ID created at PDA:", farmerPDA.toBase58());
  });

  it("Blocks duplicate farmer registration", async () => {
    try {
      await program.methods
        .createFarmerId("Duplicate", "Maize", "Kenema", "Sierra Leone", "+232 77000000", 1.0)
        .accounts({
          farmer: farmerPDA,
          lenderCheck: lenderPDA,
          owner: owner,
          systemProgram: anchor.web3.SystemProgram.programId,
        })
        .rpc();
      assert.fail("Should have thrown — duplicate registration");
    } catch (err) {
      console.log("✅ Duplicate blocked as expected:", err.message);
    }
  });

  it("Registers a lender on a separate wallet", async () => {
    await program.methods
      .registerLender(
        "Freetown Agricultural Cooperative",
        "Cooperative / SACCO",
        "Sierra Leone",
        "Freetown",
        "contact@fac.sl",
        new anchor.BN(5),
        900, // 9.00% APR in basis points
        12,
        new anchor.BN(0),
        new anchor.BN(LENDER_CAPITAL_BUDGET)
      )
      .accounts({
        lender: lenderPDA2,
        farmerCheck: farmerCheckForLenderWallet,
        owner: lenderWallet.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .signers([lenderWallet])
      .rpc();

    const lender = await program.account.lenderAccount.fetch(lenderPDA2);
    assert.equal(lender.name, "Freetown Agricultural Cooperative");
    assert.equal(lender.isActive, false);
    assert.equal(lender.capitalBudgetLamports.toNumber(), LENDER_CAPITAL_BUDGET);
    console.log("✅ Lender registered (pending approval) at PDA:", lenderPDA2.toBase58());
  });

  it("Blocks a farmer wallet from also registering as a lender", async () => {
    try {
      await program.methods
        .registerLender(
          "Should Not Work",
          "MFI",
          "Sierra Leone",
          "Bo",
          "fail@example.com",
          new anchor.BN(1),
          500,
          8,
          new anchor.BN(0),
          new anchor.BN(100)
        )
        .accounts({
          lender: lenderPDA,
          farmerCheck: farmerPDA, // already has FarmerAccount data
          owner: owner,
          systemProgram: anchor.web3.SystemProgram.programId,
        })
        .rpc();
      assert.fail("Should have thrown — wallet already has Farmer role");
    } catch (err) {
      console.log("✅ Cross-role registration correctly blocked:", err.message);
    }
  });

  it("Blocks loan request from an unapproved lender", async () => {
    try {
      await program.methods
        .requestLoan(new anchor.BN(1_000_000), "Buy fertilizer", 8)
        .accounts({
          loan: loanPDA,
          farmer: farmerPDA,
          lender: lenderPDA2,
          owner: owner,
          systemProgram: anchor.web3.SystemProgram.programId,
        })
        .rpc();
      assert.fail("Should have thrown — lender not yet approved");
    } catch (err) {
      console.log("✅ Loan correctly blocked — lender not active:", err.message);
    }
  });

  it("Admin approves the lender", async () => {
    await program.methods
      .approveLender()
      .accounts({
        lender: lenderPDA2,
        admin: owner, // NOTE: replace with actual ADMIN_PUBKEY wallet on real devnet test
      })
      .rpc();

    const lender = await program.account.lenderAccount.fetch(lenderPDA2);
    assert.equal(lender.isActive, true);
    console.log("✅ Lender approved and now active");
  });

  it("Requests a loan from the approved lender and decrements capital budget", async () => {
    await program.methods
      .requestLoan(new anchor.BN(1_000_000), "Buy fertilizer", 8)
      .accounts({
        loan: loanPDA,
        farmer: farmerPDA,
        lender: lenderPDA2,
        owner: owner,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .rpc();

    const loan = await program.account.loanAccount.fetch(loanPDA);
    assert.equal(loan.amount.toNumber(), 1_000_000);
    assert.equal(loan.purpose, "Buy fertilizer");
    assert.equal(loan.repaymentWeeks, 8);
    assert.equal(loan.isRepaid, false);
    assert.equal(loan.lender.toBase58(), lenderPDA2.toBase58());

    const farmer = await program.account.farmerAccount.fetch(farmerPDA);
    assert.equal(farmer.hasActiveLoan, true);
    assert.equal(farmer.loanCounter.toNumber(), 1);

    // Capital bookkeeping: budget should have decremented by exactly the
    // loan amount.
    const lender = await program.account.lenderAccount.fetch(lenderPDA2);
    assert.equal(
      lender.capitalBudgetLamports.toNumber(),
      LENDER_CAPITAL_BUDGET - 1_000_000,
      "Capital budget should decrement by the exact loan amount"
    );

    console.log(
      "✅ Loan requested successfully, capital budget now:",
      lender.capitalBudgetLamports.toNumber()
    );
  });

  it("Blocks second loan while one is active", async () => {
    try {
      await program.methods
        .requestLoan(new anchor.BN(500_000), "Second loan attempt", 4)
        .accounts({
          loan: loanPDA,
          farmer: farmerPDA,
          lender: lenderPDA2,
          owner: owner,
          systemProgram: anchor.web3.SystemProgram.programId,
        })
        .rpc();
      assert.fail("Should have thrown — active loan exists");
    } catch (err) {
      console.log("✅ Second loan blocked as expected:", err.message);
    }
  });

  it("Repays the loan, restores capital budget, and leaves the account on-chain", async () => {
    const lenderBefore = await program.account.lenderAccount.fetch(lenderPDA2);

    await program.methods
      .repayLoan()
      .accounts({
        loan: loanPDA,
        farmer: farmerPDA,
        lender: lenderPDA2,
        owner: owner,
      })
      .rpc();

    // V2.3.1 removed close = owner from RepayLoan — the loan account stays
    // on-chain, correctly flagged isRepaid, until the farmer explicitly
    // clears it via close_loan (tested below). This replaces the old
    // assertion that the account was closed on repayment, which no longer
    // reflects the deployed contract.
    const loan = await program.account.loanAccount.fetch(loanPDA);
    assert.equal(loan.isRepaid, true);

    const farmer = await program.account.farmerAccount.fetch(farmerPDA);
    assert.equal(farmer.hasActiveLoan, false);
    assert.equal(farmer.creditScore.toNumber(), 10);

    // Capital bookkeeping: the repaid amount should be restored to the
    // lender's budget, bringing it back to LENDER_CAPITAL_BUDGET.
    const lenderAfter = await program.account.lenderAccount.fetch(lenderPDA2);
    assert.equal(
      lenderAfter.capitalBudgetLamports.toNumber(),
      lenderBefore.capitalBudgetLamports.toNumber() + 1_000_000,
      "Capital budget should be restored by the repaid loan amount"
    );
    assert.equal(
      lenderAfter.capitalBudgetLamports.toNumber(),
      LENDER_CAPITAL_BUDGET,
      "Capital budget should be back to its original registered value"
    );

    console.log(
      "✅ Loan repaid, flagged isRepaid, credit score up, capital restored to:",
      lenderAfter.capitalBudgetLamports.toNumber()
    );
  });

  it("Closes the repaid loan account, reclaiming rent", async () => {
    await program.methods
      .closeLoan()
      .accounts({
        loan: loanPDA,
        owner: owner,
      })
      .rpc();

    const loan = await program.account.loanAccount.fetchNullable(loanPDA);
    assert.isNull(loan, "Loan account should be closed after close_loan");
    console.log("✅ close_loan reclaimed rent, PDA freed for re-borrow");
  });

  it("Blocks a loan request that exceeds remaining capital budget", async () => {
    const lenderBefore = await program.account.lenderAccount.fetch(lenderPDA2);

    try {
      await program.methods
        .requestLoan(new anchor.BN(5_000_000), "Loan larger than remaining capital", 6)
        .accounts({
          loan: loanPDA, // PDA freed by close_loan above
          farmer: farmerPDA,
          lender: lenderPDA2,
          owner: owner,
          systemProgram: anchor.web3.SystemProgram.programId,
        })
        .rpc();
      assert.fail("Should have thrown — amount exceeds lender's capital budget");
    } catch (err) {
      console.log("✅ Loan correctly blocked — exceeds capital budget:", err.message);
    }

    // Solana transactions are atomic — a failed request_loan call must
    // leave the lender's budget completely untouched, not partially
    // decremented.
    const lenderAfter = await program.account.lenderAccount.fetch(lenderPDA2);
    assert.equal(
      lenderAfter.capitalBudgetLamports.toNumber(),
      lenderBefore.capitalBudgetLamports.toNumber(),
      "Capital budget must be unchanged after a rejected request"
    );

    const farmer = await program.account.farmerAccount.fetch(farmerPDA);
    assert.equal(
      farmer.hasActiveLoan,
      false,
      "Farmer should not show an active loan after a rejected request"
    );
  });

  it("Requests a second loan within remaining capital, decrementing correctly", async () => {
    await program.methods
      .requestLoan(new anchor.BN(2_000_000), "Second season inputs", 10)
      .accounts({
        loan: loanPDA,
        farmer: farmerPDA,
        lender: lenderPDA2,
        owner: owner,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .rpc();

    const lender = await program.account.lenderAccount.fetch(lenderPDA2);
    assert.equal(
      lender.capitalBudgetLamports.toNumber(),
      LENDER_CAPITAL_BUDGET - 2_000_000,
      "Capital budget should decrement to 1,000,000 after this loan"
    );

    const farmer = await program.account.farmerAccount.fetch(farmerPDA);
    assert.equal(farmer.hasActiveLoan, true);
    assert.equal(farmer.loanCounter.toNumber(), 2);

    console.log(
      "✅ Second loan issued, capital budget correctly at:",
      lender.capitalBudgetLamports.toNumber()
    );
  });
});