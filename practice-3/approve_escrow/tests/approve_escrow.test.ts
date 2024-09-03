import { expect, describe, beforeAll, test } from "@jest/globals";
import * as anchor from "@coral-xyz/anchor";
import { type Program, BN } from "@coral-xyz/anchor";
import { ApproveEscrow } from "../target/types/approve_escrow";
import {
  Connection,
  Keypair,
  LAMPORTS_PER_SOL,
  PublicKey,
  SystemProgram,
  Transaction,
  TransactionInstruction,
} from "@solana/web3.js";
import {
  ASSOCIATED_TOKEN_PROGRAM_ID,
  MINT_SIZE,
  TOKEN_2022_PROGRAM_ID,
  TOKEN_PROGRAM_ID,
  createAssociatedTokenAccountIdempotentInstruction,
  createInitializeMint2Instruction,
  createMintToInstruction,
  getAssociatedTokenAddressSync,
  getMinimumBalanceForRentExemptMint,
} from "@solana/spl-token";
import { createTransferInstruction } from "@solana/spl-token";
import { randomBytes } from "crypto";
import { confirmTransaction, makeKeypairs } from "@solana-developers/helpers";

const TOKEN_PROGRAM: typeof TOKEN_2022_PROGRAM_ID | typeof TOKEN_PROGRAM_ID =
  TOKEN_2022_PROGRAM_ID;

export const getRandomBigNumber = (size: number = 8) => {
  return new BN(randomBytes(size));
};

function areBnEqual(a: unknown, b: unknown): boolean | undefined {
  const isABn = a instanceof BN;
  const isBBn = b instanceof BN;

  if (isABn && isBBn) {
    return a.eq(b);
  } else if (isABn === isBBn) {
    return undefined;
  } else {
    return false;
  }
}
expect.addEqualityTesters([areBnEqual]);

const createTokenAndMintTo = async (
  connection: Connection,
  payer: PublicKey,
  tokenMint: PublicKey,
  decimals: number,
  mintAuthority: PublicKey,
  mintTo: Array<{ recepient: PublicKey; amount: number }>
): Promise<Array<TransactionInstruction>> => {
  let minimumLamports = await getMinimumBalanceForRentExemptMint(connection);

  let createTokeIxs = [
    SystemProgram.createAccount({
      fromPubkey: payer,
      newAccountPubkey: tokenMint,
      lamports: minimumLamports,
      space: MINT_SIZE,
      programId: TOKEN_PROGRAM,
    }),
    createInitializeMint2Instruction(
      tokenMint,
      decimals,
      mintAuthority,
      null,
      TOKEN_PROGRAM
    ),
  ];

  let mintToIxs = mintTo.flatMap(({ recepient, amount }) => {
    const ataAddress = getAssociatedTokenAddressSync(
      tokenMint,
      recepient,
      false,
      TOKEN_PROGRAM
    );

    return [
      createAssociatedTokenAccountIdempotentInstruction(
        payer,
        ataAddress,
        recepient,
        tokenMint,
        TOKEN_PROGRAM
      ),
      createMintToInstruction(
        tokenMint,
        ataAddress,
        mintAuthority,
        amount,
        [],
        TOKEN_PROGRAM
      ),
    ];
  });

  return [...createTokeIxs, ...mintToIxs];
};

const getTokenBalanceOn = (
  connection: Connection,
) => async (
  tokenAccountAddress: PublicKey,
): Promise<BN> => {
  const tokenBalance = await connection.getTokenAccountBalance(tokenAccountAddress);
  return new BN(tokenBalance.value.amount);
};

describe("approve_escrow", () => { 
  anchor.setProvider(anchor.AnchorProvider.env());

  const provider = anchor.getProvider();
  const connection = provider.connection;
  const program = anchor.workspace.ApproveEscrow as Program<ApproveEscrow>;

  const [alice, bob, usdcMint, wifMint] = makeKeypairs(4);

  const [aliceUsdcAccount, aliceWifAccount, bobUsdcAccount, bobWifAccount] = [
    alice,
    bob,
  ].flatMap((owner) =>
    [usdcMint, wifMint].map((tokenMint) =>
      getAssociatedTokenAddressSync(
        tokenMint.publicKey,
        owner.publicKey,
        false,
        TOKEN_PROGRAM
      )
    )
  );

  const offerId = getRandomBigNumber();

  beforeAll(async () => {
    const giveAliceAndBobSolIxs: Array<TransactionInstruction> = [
      alice,
      bob,
    ].map((owner) =>
      SystemProgram.transfer({
        fromPubkey: provider.publicKey,
        toPubkey: owner.publicKey,
        lamports: 10 * LAMPORTS_PER_SOL,
      })
    );

    const usdcSetupIxs = await createTokenAndMintTo(
      connection,
      provider.publicKey,
      usdcMint.publicKey,
      6,
      alice.publicKey,
      [
        { recepient: alice.publicKey, amount: 100_000_000 },
        { recepient: bob.publicKey, amount: 20_000_000 },
      ]
    );

    const wifSetupIxs = await createTokenAndMintTo(
      connection,
      provider.publicKey,
      wifMint.publicKey,
      6,
      bob.publicKey,
      [
        { recepient: alice.publicKey, amount: 5_000_000 },
        { recepient: bob.publicKey, amount: 300_000_000 },
      ]
    );

    let tx = new Transaction();
    tx.instructions = [
      ...giveAliceAndBobSolIxs,
      ...usdcSetupIxs,
      ...wifSetupIxs,
    ];

    const _setupTxSig = await provider.sendAndConfirm(tx, [
      alice,
      bob,
      usdcMint,
      wifMint,
    ]);
  });

  const makeOfferTx = async (
    maker: Keypair,
    offerId: BN,
    offeredTokenMint: PublicKey,
    offeredAmount: BN,
    wantedTokenMint: PublicKey,
    wantedAmount: BN
  ): Promise<{
    offerAddress: PublicKey;
    vaultAddress: PublicKey;
  }> => {
    const transactionSignature = await program.methods
      .makeOffer(offerId, offeredAmount, wantedAmount)
      .accounts({
        maker: maker.publicKey,
        tokenMintA: offeredTokenMint,
        tokenMintB: wantedTokenMint,
        tokenProgram: TOKEN_PROGRAM,
      })
      .signers([maker])
      .rpc();

    await confirmTransaction(connection, transactionSignature);

    const [offerAddress, _offerBump] = PublicKey.findProgramAddressSync(
      [
        Buffer.from("offer"),
        maker.publicKey.toBuffer(),
        offerId.toArrayLike(Buffer, "le", 8),
      ],
      program.programId
    );

    const vaultAddress = getAssociatedTokenAddressSync(
      offeredTokenMint,
      offerAddress,
      true,
      TOKEN_PROGRAM
    );

    return { offerAddress, vaultAddress };
  };

  const takeOfferTx = async (
    offerAddress: PublicKey,
    taker: Keypair,
  ): Promise<void> => {
    const transactionSignature = await program.methods
      .takeOffer()
      .accounts({
        taker: taker.publicKey,
        offer: offerAddress,
        tokenProgram: TOKEN_PROGRAM,
      })
      .signers([taker])
      .rpc();

    await confirmTransaction(connection, transactionSignature);
  };
  

  const takeOfferTxWithIncorrectMint = async (
    offerAddress: PublicKey,
    taker: Keypair,
    wrongMintAccount: PublicKey
  ): Promise<void> => {
    const transactionSignature = await program.methods
      .takeOffer()
      .accounts({
        taker: taker.publicKey,
        offer: offerAddress,
        tokenProgram: TOKEN_PROGRAM,
      })
      .signers([taker])
      .rpc();

    await confirmTransaction(connection, transactionSignature);
  };

  test("Offer created by Alice with token approval", async () => {
    const offeredUsdc = new BN(10_000_000);
    const wantedWif = new BN(100_000_000);

    const getTokenBalance = getTokenBalanceOn(connection);

    const { offerAddress, vaultAddress } = await makeOfferTx(
      alice,
      offerId,
      usdcMint.publicKey,
      offeredUsdc,
      wifMint.publicKey,
      wantedWif
    );

    expect(await getTokenBalance(aliceUsdcAccount)).toEqual(new BN(90_000_000));
    expect(await getTokenBalance(vaultAddress)).toEqual(offeredUsdc);

    const offerAccount = await program.account.offer.fetch(offerAddress);
    expect(offerAccount.maker).toEqual(alice.publicKey);
    expect(offerAccount.tokenMintA).toEqual(usdcMint.publicKey);
    expect(offerAccount.tokenMintB).toEqual(wifMint.publicKey);
    expect(offerAccount.tokenBWantedAmount).toEqual(wantedWif);
  });

  test("Offer taken by Bob with approved tokens", async () => {
    const getTokenBalance = getTokenBalanceOn(connection);

    const [offerAddress, _offerBump] = PublicKey.findProgramAddressSync(
      [
        Buffer.from("offer"),
        alice.publicKey.toBuffer(),
        offerId.toArrayLike(Buffer, "le", 8),
      ],
      program.programId
    );

    expect(await getTokenBalance(aliceUsdcAccount)).toEqual(new BN(90_000_000));
    expect(await getTokenBalance(aliceWifAccount)).toEqual(new BN(5_000_000));
    expect(await getTokenBalance(bobUsdcAccount)).toEqual(new BN(20_000_000));
    expect(await getTokenBalance(bobWifAccount)).toEqual(new BN(300_000_000));

    await takeOfferTx(offerAddress, bob);

    expect(await getTokenBalance(aliceUsdcAccount)).toEqual(new BN(90_000_000));
    expect(await getTokenBalance(aliceWifAccount)).toEqual(new BN(105_000_000));

    expect(await getTokenBalance(bobUsdcAccount)).toEqual(new BN(30_000_000));
    expect(await getTokenBalance(bobWifAccount)).toEqual(new BN(200_000_000));
  });
});
