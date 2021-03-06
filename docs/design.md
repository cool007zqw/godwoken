# Design

This document explains the design of Godwoken from a high-level overview.

Godwoken is an experimental optimistic rollup implementation to provide a generalized account-based programming layer upon Nervos CKB.

> Some people refer to Rollup as layer-1.5; some people think it's layer-2, or even layer-1(by trust-level). This document refers to Rollup as layer-1.5 to distinguish it with the layer-1 CKB.

> Since we are still WIP, this document may not reflects every details in the project.

We try to solve two problems: scalability and aggregation.

### Scalability

Godwoken applies optimistic rollup architecture to promise scalability; the godwoken transactions designed to be light-weight; it takes less size and does not perform on-chain verification.

### Aggregation

Aggregation is hard on a UTXO-like chain, For example:

    * Voting
    * Randao
    * Decentralized Oracle
    * ... any other contracts that need shared state

In a UTXO-like model, the state is naturally separated.

In CKB, users can vote in separate cells. An off-chain actor collects voting cells and calculates the result.

It works fine when we only want to “see” the result. But we can’t use the voting result in another contract, for example, a voting based DAO contract. It’s hard to verify the aggregated result in an on-chain contract. Since we need to prove exists of voting cells, the transaction must refer to every voting cell; it could be costly.

Godwoken solves aggregation by providing an account programming layer upon cell model.

Godwoken shares the same tech stack with the native CKB contract. The only difference is that Godwoken contract is account-based; it verifies the state of account instead of the cells. The mapping relationship between account state and layer-1 cells is handled by the Godwoken main contract, which is transparent for layer-1.5 contracts.

## Architecture

Godwoken composited by the following parts:

### On-chain

* Main contract - a type script maintains the global state of all accounts and all blocks(layer-1.5).
* Challenge contract - a type script that handles challenge requests.

### Off-chain

* Aggregator - an off-chain program that collects layer-1.5 transactions and submits layer-1.5 blocks to the main contract regularly.
* Validator - an off-chain program that continuously watches the two contracts. The validator sends a challenge request when an invalid block is submitted and sends an invalid challenge request when a wrong challenge request is submitted.

Usually, an aggregator is also a validator.

## Layer 1.5 structures

### Account

```
table Account {
    index: Uint64, // address index
    script: AccountScriptOpt, // account's code
    nonce: Uint32, // nonce
    pubkey_hash: Byte20, // pubkey hash
}

table AccountScript {
    code_hash: Byte32, // hash code the code
    args: Bytes, // initialized args of the Account
}
```

To register an account, a user needs to send `register` action to the Godwoken contract, and deposit layer-1 assets.

`index` field used to indicates an account, for a newly registered account, `index` must equal to the `last_account.index + 1`.

`nonce` used to prevent the replay attack, each time a tx sent to an account, the nonce will increase by `1`;

`script` field used for account-model contract: when an account receives messages, the script code will be loaded and executed. A non-contract account uses none value.

`pubkey_hash` the pubkey hash, Godwoken use secp256k1 signature now, maybe the BLS signature will be used in the future.

### Block

```
table AgBlock {
    number: Uint64, // block number
    tx_root: Byte32,
    txs_count: Uint32,
    prev_account_root: Byte32, // account root before this block
    prev_account_count: Uint64,
    account_root: Byte32, // account root after this block
    ag_sig: Byte65, // Aggregator's signature
    ag_index: Uint64, // Aggregator's index
}
```

`number`, must equal to `last_block.numer + 1`.

`tx_root`, merkle root of transactions, the transactions are separated from block structure to make blocks small.

`prev_account_root`, merkle root of all accounts before this block.

`account_root`, merkle root of all accounts after this block.

`ag_sig`, aggregator's signature, the signed message is computed by filling zeros to the `ag_sig` field then hash the block.

`ag_index`, the index of the aggregator account.

### Tx

```
table Tx {
    sender_index: Uint64,
    to_index: Uint64,
    nonce: Uint32, // nonce
    amount: Payment, // amount
    fee: Payment, // fee
    args: Bytes, // pass args to contract
    witness: Bytes, // tx's signature
}

union Payment {
    Uint32,
    UDTPayment,
}

struct UDTPayment {
    type_hash: Byte32,
    amount: Uint32,
}
```

`nonce` must equals to `account.nonce + 1`.

`amount` can be either native token or UDT.

`fee` is transferred to the aggregator's account.

`args` is used for calling contract; it has no use when the recipient is a non-contract account.

`witness` contains the user's signature of the transaction; this field will be removed after the BLS signature.

## Main contract

### Global state

Godwoken contract maintains a global state:

``` txt
struct GlobalState {
    account_root: Byte32, // merkle root of accounts
    block_root: Byte32, // merkle root of blocks
    account_count: Uint64,
    block_count: Uint64,
}
```

We use [merkle mountain range](MMR for short) to calculate the block root; use [sparse merkle tree](SMT for short) to calculate the account root.

Both accumulators allow efficiently accumulate new elements, which suitable for our use case: continuously produces new blocks and adds new accounts.

### Supported actions

Godwoken contract supports several actions to update the global state:

* register
* deposit
* submit block
* revert block
* prepare_withdraw
* withdraw

`register`, deposit layer-1 assets, and register a new account on Godwoken contract, the `index` of the new account must be `last_account.index + 1`; the `nonce` must be `0`; `script` can be set to default script or a contract.

`deposit`, deposit layer-1 assets to `account_root`.

`submit block`, only an aggregator account with the required balance, can invoke this action. The aggregator needs to commit `block`, `transactions`, and merkle proof; the `transactions` will not verify on-chain; however other users can send an invalid block action to penalize the aggregator who committed an invalid block and take the deposited assets from the aggregator.

`revert block`, the challenge logic is handling by challenge contract, here we only care about the challenge result. Anyone who has an account can send a `revert block` request with a challenge result cell. If the challenge result is valid, the reverted block will be replaced with: `Block { (untouched fields: number, previous_account_root), tx_root: 0x00..00, ag_sig: 0x00..00, ag_index: challenger_index, account_root: new_account_root }`, in the `new_account_root`, part of the reverted aggregator's CKB is sent to challenger's account as the reward.

`prepare_withdraw`, move assets to a withdrawing state.

`withdraw`, after `WITHDRAW_WAIT` blocks of the `prepare_withdraw` action; a user can take assets from withdrawing state to layer-1.

## Challenge contract

The challenge contract verifies challenge request cells.

* Anyone who has an account can prepare a challenge request cell with challenge contract as cell type and deposited CKB as bond.
* Since off-chain validators continuously watch the chain, if incorrect challenge request cells are generated, validates can send context data to invalidate the challenge and get the bond.
* After some time, if no one invalidates the challenge request cell, the cell becomes valid.
* An valid challenge request cell can revert blocks in the main contract; the challenger will get a bond from the main contract.

[merkle mountain range]: https://github.com/nervosnetwork/merkle-mountain-range "merkle mountain range"
[sparse merkle tree]: https://github.com/jjyr/sparse-merkle-tree "sparse merkle tree"
