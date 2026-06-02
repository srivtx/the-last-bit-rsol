import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { Zeroxcounter } from "../../../target/types/zeroxcounter";

describe("zeroxcounter", () => {

  const provider = anchor.AnchorProvider.env();

  anchor.setProvider(provider);

  const program = anchor.workspace.zeroxcounter as Program<Zeroxcounter>;

  const counter = anchor.web3.Keypair.generate();

  it("Initialize Counter", async () => {

    await program.methods
      .initialize()
      .accounts({
        authority: provider.wallet.publicKey,
        counter: counter.publicKey,
      })
      .signers([counter])
      .rpc();


    const counterAccount = await program.account.counter.fetch(
      counter.publicKey
    );

    console.log("Authority:", counterAccount.authority.toBase58());

    console.log("Count:", counterAccount.count.toString());

  });

});