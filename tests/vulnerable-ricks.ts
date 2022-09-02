import * as anchor from "@project-serum/anchor";
import { Program } from "@project-serum/anchor";
import { VulnerableRicks } from "../target/types/vulnerable_ricks";

describe("vulnerable-ricks", () => {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.AnchorProvider.env());

  const program = anchor.workspace.VulnerableRicks as Program<VulnerableRicks>;

  it("Is initialized!", async () => {
    // Add your test here.
    const tx = await program.methods.initialize().rpc();
    console.log("Your transaction signature", tx);
  });
});
