import * as anchor from "@coral-xyz/anchor";
import { Program, BN } from "@coral-xyz/anchor";
import { TokenFactory } from "../target/types/token_factory";
import {
  Keypair,
  PublicKey,
  SystemProgram,
  SYSVAR_RENT_PUBKEY,
} from "@solana/web3.js";
import {
  TOKEN_2022_PROGRAM_ID,
  ASSOCIATED_TOKEN_PROGRAM_ID,
  getAssociatedTokenAddressSync,
} from "@solana/spl-token";
import assert from "assert";

const TOKEN_CONFIG_SEED = Buffer.from("token_config");
const PLATFORM_FEE_WALLET = new PublicKey("7LA1ZMrc4j19sCSnXFmmiLvjo6KVWENwv9aS4oXYKq2E");

function getTokenConfigPDA(mint: PublicKey, programId: PublicKey): [PublicKey, number] {
  return PublicKey.findProgramAddressSync(
    [TOKEN_CONFIG_SEED, mint.toBuffer()],
    programId
  );
}

function getATA(mint: PublicKey, owner: PublicKey): PublicKey {
  return getAssociatedTokenAddressSync(
    mint, owner, false, TOKEN_2022_PROGRAM_ID
  );
}

describe("token-factory", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.TokenFactory as Program<TokenFactory>;
  const creator = provider.wallet as anchor.Wallet;

  const mintKeypair = Keypair.generate();
  let tokenConfigPDA: PublicKey;
  let creatorATA: PublicKey;

  before(() => {
    [tokenConfigPDA] = getTokenConfigPDA(mintKeypair.publicKey, program.programId);
    creatorATA = getATA(mintKeypair.publicKey, creator.publicKey);

    console.log("\n── Accounts ──────────────────────────────────");
    console.log("Program ID  :", program.programId.toString());
    console.log("Creator     :", creator.publicKey.toString());
    console.log("Mint        :", mintKeypair.publicKey.toString());
    console.log("TokenConfig :", tokenConfigPDA.toString());
    console.log("Creator ATA :", creatorATA.toString());
    console.log("──────────────────────────────────────────────\n");
  });

  // ── Test 1: initialize_token ──────────────────────────────────
  it("initializes a Token-2022 token with metadata and transfer fee", async () => {
    const params = {
      name:           "Test Token",
      symbol:         "TEST",
      uri:            "https://example.com/metadata.json",
      decimals:       6,
      initialSupply:  new BN(1_000_000),
      maxSupply:      new BN(10_000_000),
      transferFeeBps: 100,
      transferFeeMax: new BN(1_000_000_000),
      website:        "https://example.com",
      twitter:        "https://twitter.com/test",
      telegram:       "https://t.me/test",
      discord:        "https://discord.gg/test",
    };

    const tx = await program.methods
      .initializeToken(params)
      .accountsPartial({
        creator:      creator.publicKey,
        feeWallet:    PLATFORM_FEE_WALLET,
        mint:         mintKeypair.publicKey,
        tokenConfig:  tokenConfigPDA,
        tokenProgram: TOKEN_2022_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
        rent:         SYSVAR_RENT_PUBKEY,
      })
      .signers([mintKeypair])
      .rpc({ commitment: "confirmed" });

    console.log("✓ initialize_token tx:", tx);
    console.log("  https://solscan.io/tx/" + tx + "?cluster=devnet");

    const config = await program.account.tokenConfig.fetch(tokenConfigPDA);
    assert.equal(config.creator.toString(), creator.publicKey.toString());
    assert.equal(config.decimals, 6);
    assert.equal(config.initialSupply.toString(), "1000000");
    assert.equal(config.currentSupply.toString(), "0");
    assert.equal(config.mintAuthorityRevoked, false);
    console.log("✓ TokenConfig PDA verified");
  });

  // ── Test 2: mint_initial_supply ───────────────────────────────
  it("mints initial supply to creator ATA", async () => {
    const tx = await program.methods
      .mintInitialSupply()
      .accountsPartial({
        creator:               creator.publicKey,
        mint:                  mintKeypair.publicKey,
        tokenConfig:           tokenConfigPDA,
        creatorAta:            creatorATA,
        tokenProgram:          TOKEN_2022_PROGRAM_ID,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        systemProgram:         SystemProgram.programId,
      })
      .rpc({ commitment: "confirmed" });

    console.log("✓ mint_initial_supply tx:", tx);

    const config = await program.account.tokenConfig.fetch(tokenConfigPDA);
    assert.equal(config.currentSupply.toString(), "1000000");
    console.log("✓ Current supply:", config.currentSupply.toString());
  });

  // ── Test 3: mint_additional ───────────────────────────────────
  it("mints additional tokens", async () => {
    const tx = await program.methods
      .mintAdditional(new BN(500_000))
      .accountsPartial({
        creator:               creator.publicKey,
        mint:                  mintKeypair.publicKey,
        tokenConfig:           tokenConfigPDA,
        creatorAta:            creatorATA,
        tokenProgram:          TOKEN_2022_PROGRAM_ID,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        systemProgram:         SystemProgram.programId,
      })
      .rpc({ commitment: "confirmed" });

    console.log("✓ mint_additional tx:", tx);

    const config = await program.account.tokenConfig.fetch(tokenConfigPDA);
    assert.equal(config.currentSupply.toString(), "1500000");
    console.log("✓ Supply after additional mint:", config.currentSupply.toString());
  });

  // ── Test 4: update_metadata ───────────────────────────────────
  it("updates token metadata", async () => {
    const tx = await program.methods
      .updateMetadata({
        name:     "Updated Token",
        symbol:   null,
        uri:      null,
        website:  "https://updated.com",
        twitter:  null,
        telegram: null,
        discord:  null,
      })
      .accountsPartial({
        creator:       creator.publicKey,
        mint:          mintKeypair.publicKey,
        tokenConfig:   tokenConfigPDA,
        tokenProgram:  TOKEN_2022_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
      })
      .rpc({ commitment: "confirmed" });

    console.log("✓ update_metadata tx:", tx);
  });

  // ── Test 5: revoke_authority ──────────────────────────────────
  it("revokes mint and update authority permanently", async () => {
    const tx = await program.methods
      .revokeAuthority({
        revokeMint:   true,
        revokeFreeze: false,
        revokeUpdate: true,
      })
      .accountsPartial({
        creator:       creator.publicKey,
        mint:          mintKeypair.publicKey,
        tokenConfig:   tokenConfigPDA,
        tokenProgram:  TOKEN_2022_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
      })
      .rpc({ commitment: "confirmed" });

    console.log("✓ revoke_authority tx:", tx);

    const config = await program.account.tokenConfig.fetch(tokenConfigPDA);
    assert.equal(config.mintAuthorityRevoked, true);
    assert.equal(config.updateAuthorityRevoked, true);
    assert.equal(config.freezeAuthorityRevoked, false);
    console.log("✓ Revocation flags verified");
  });

  // ── Test 6: mint blocked after revoke ─────────────────────────
  it("blocks mint_additional after revocation", async () => {
    try {
      await program.methods
        .mintAdditional(new BN(100_000))
        .accountsPartial({
          creator:               creator.publicKey,
          mint:                  mintKeypair.publicKey,
          tokenConfig:           tokenConfigPDA,
          creatorAta:            creatorATA,
          tokenProgram:          TOKEN_2022_PROGRAM_ID,
          associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
          systemProgram:         SystemProgram.programId,
        })
        .rpc({ commitment: "confirmed" });

      assert.fail("Should have thrown MintAuthorityRevoked");
    } catch (err: any) {
      assert.ok(
        err.toString().includes("MintAuthorityRevoked"),
        "Expected MintAuthorityRevoked, got: " + err.toString()
      );
      console.log("✓ Correctly blocked mint after revocation");
    }
  });

  // ── Test 7: harvest_fees ──────────────────────────────────────
  it("harvests withheld transfer fees", async () => {
    const tx = await program.methods
      .harvestFees()
      .accountsPartial({
        creator:      creator.publicKey,
        mint:         mintKeypair.publicKey,
        tokenConfig:  tokenConfigPDA,
        creatorAta:   creatorATA,
        tokenProgram: TOKEN_2022_PROGRAM_ID,
      })
      .rpc({ commitment: "confirmed" });

    console.log("✓ harvest_fees tx:", tx);
  });

  // ── Test 8: freeze and thaw ───────────────────────────────────
  it("freezes and thaws creator ATA", async () => {
    const freezeTx = await program.methods
      .freezeTokenAccount()
      .accountsPartial({
        creator:       creator.publicKey,
        mint:          mintKeypair.publicKey,
        targetAccount: creatorATA,
        tokenConfig:   tokenConfigPDA,
        tokenProgram:  TOKEN_2022_PROGRAM_ID,
      })
      .rpc({ commitment: "confirmed" });

    console.log("✓ freeze tx:", freezeTx);

    const thawTx = await program.methods
      .thawTokenAccount()
      .accountsPartial({
        creator:       creator.publicKey,
        mint:          mintKeypair.publicKey,
        targetAccount: creatorATA,
        tokenConfig:   tokenConfigPDA,
        tokenProgram:  TOKEN_2022_PROGRAM_ID,
      })
      .rpc({ commitment: "confirmed" });

    console.log("✓ thaw tx:", thawTx);
    console.log("✓ Freeze/thaw cycle complete");
  });
});
