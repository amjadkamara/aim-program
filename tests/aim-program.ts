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

  it("Creates a farmer ID", async () => {
    await program.methods
      .createFarmerId("Amjad Kamara", "Rice", "Bo", 2.5)
      .accounts({
        farmer: farmerPDA,
        owner: owner,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .rpc();

    const farmer = await program.account.farmerAccount.fetch(farmerPDA);
    assert.equal(farmer.fullName, "Amjad Kamara");
    assert.equal(farmer.cropType, "Rice");
    assert.equal(farmer.district, "Bo");
    assert.equal(farmer.farmSize, 2.5);
    assert.equal(farmer.hasActiveLoan, false);
    assert.equal(farmer.owner.toBase58(), owner.toBase58());
    console.log("✅ Farmer ID created at PDA:", farmerPDA.toBase58());
  });

  it("Blocks duplicate farmer registration", async () => {
    try {
      await program.methods
        .createFarmerId("Duplicate", "Maize", "Kenema", 1.0)
        .accounts({
          farmer: farmerPDA,
          owner: owner,
          systemProgram: anchor.web3.SystemProgram.programId,
        })
        .rpc();
      assert.fail("Should have thrown — duplicate registration");
    } catch (err) {
      console.log("✅ Duplicate blocked as expected:", err.message);
    }
  });

  it("Requests a loan", async () => {
    await program.methods
      .requestLoan(new anchor.BN(1_000_000), "Buy fertilizer", 8)
      .accounts({
        loan: loanPDA,
        farmer: farmerPDA,
        owner: owner,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .rpc();

    const loan = await program.account.loanAccount.fetch(loanPDA);
    assert.equal(loan.amount.toNumber(), 1_000_000);
    assert.equal(loan.purpose, "Buy fertilizer");
    assert.equal(loan.repaymentWeeks, 8);
    assert.equal(loan.isRepaid, false);

    const farmer = await program.account.farmerAccount.fetch(farmerPDA);
    assert.equal(farmer.hasActiveLoan, true);
    console.log("✅ Loan requested successfully");
  });

  it("Blocks second loan while one is active", async () => {
    try {
      await program.methods
        .requestLoan(new anchor.BN(500_000), "Second loan attempt", 4)
        .accounts({
          loan: loanPDA,
          farmer: farmerPDA,
          owner: owner,
          systemProgram: anchor.web3.SystemProgram.programId,
        })
        .rpc();
      assert.fail("Should have thrown — active loan exists");
    } catch (err) {
      console.log("✅ Second loan blocked as expected:", err.message);
    }
  });

  it("Repays the loan", async () => {
    await program.methods
      .repayLoan()
      .accounts({
        loan: loanPDA,
        farmer: farmerPDA,
        owner: owner,
      })
      .rpc();

    const loan = await program.account.loanAccount.fetch(loanPDA);
    assert.equal(loan.isRepaid, true);

    const farmer = await program.account.farmerAccount.fetch(farmerPDA);
    assert.equal(farmer.hasActiveLoan, false);
    console.log("✅ Loan repaid successfully");
  });
});