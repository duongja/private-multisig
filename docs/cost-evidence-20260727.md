# Cost Evidence - 2026-07-27 UTC

This note records the current cost and execution-cost position for LP-0002 on
the current LEZ v0.2 stack.

## What Is Measured Today

The cleanest cost evidence currently available is still split into two parts:

1. measured RISC0 proof-cycle statistics from real proving with
   `RISC0_DEV_MODE=0`;
2. transaction inclusion evidence for localnet and hosted testnet execution.

Official per-transaction CU for deploy, `create_multisig`, `propose`, and
`execute_private` is still not exposed by the Logos sequencer RPC used by this
project.

## Proof-Cycle Evidence

Measured proof stats were generated from:

```text
.local/smoke/paths-20260625T222951Z/proof/proof-stats.json
```

Key values:

| Field | Value |
| --- | ---: |
| Total cycles | `262144` |
| User cycles | `198643` |
| Paging cycles | `38865` |
| Segments | `1` |
| Prove seconds | `93.86` |
| Image ID | `8330ee4acc3e4011a4ed8d919b119720f87eccd167c7abfe239021782bca9d7d` |

These are measured RISC0 proving costs for the private threshold proof. They
are not an official hosted-testnet CU bill.

## Localnet Transaction Inclusion Evidence

Current v0.2 localnet execution evidence is available from:

```text
.local/manual-localnet-check2/localnet-evidence.json
```

| Operation | Transaction hash | Included | Official CU |
| --- | --- | --- | --- |
| Private multisig deploy | `52cdb5dfb95522c3273aca74f45acfd5683fcc478edae18450bd0a5ac8f63a80` | `true` | Not exposed by current RPC |
| Target deploy | `5236a12978b154703679dc9788006a108edc776d692ca544090d36fa73001a3d` | `true` | Not exposed by current RPC |
| Create multisig | `7b6d4df450bab9865d98255e116acdf89596d1ce703ff66c73df50d4afba6262` | `true` | Not exposed by current RPC |
| Propose | `062cd7d0480d3654d904adc13d182207adc894c34988e36e5da88253efb2f844` | `true` | Not exposed by current RPC |
| Execute private | `156a9ea4006180a72bfcbf722ae6ec27abc2456e5a4403ad8fd058e0fb167268` | `true` | Not exposed by current RPC |

## Hosted Testnet Transaction Inclusion Evidence

The project now also has hosted testnet inclusion evidence for GUI-driven
threshold runs:

- threshold `1`:
  - `create_multisig`: `c50d8baf8bc84ad8cc7fc47948c02aa834759e411c40ec99ee82a7e5df461a1c`
  - `propose`: `817baa496677c40ea22e71d538816c571ed2112312b9cc4c6ebe3253c2be8375`
  - `execute_private`: `b746b177988f650855eac2833d66476d1fa1d962b0ffcce849a2d2f959726ce2`
- threshold `2`:
  - `create_multisig`: `5fff881389745eaf749615651a710bd3bccb39b72e98c03fdc1dcb275bab6037`
  - `propose`: `91fc0526249fb27baa7d3743c789f6ab4800c9edea519bbb84a9a0673d91a0ef`
  - `execute_private`: `19eed814a3df541a0bb94f9f0a9dfbae9431c8d8e61e9ffaf3607d6125087b5c`
- threshold `3`:
  - `create_multisig`: `017e413e31af3c883f122da9b65f4fd6a2483422db91290a1a952d780b2a4e49`
  - `propose`: `2ef2dac04126d9e24dcb375f6c0acb16072da2fe6ee57c4f2c88ab7baa746072`
  - `execute_private`: `1a614c701def0ca6b25d30a7611340c7f45062a579ecf575c065a5eb4dbb7790`

Those hashes prove end-to-end execution landed on hosted v0.2 testnet, but the
current RPC/explorer surface still does not provide the official per-transaction
CU values for those transactions.

## Current CU Position

What is done:

- measured proof-cycle evidence exists;
- localnet inclusion evidence exists;
- hosted testnet inclusion evidence exists for threshold `1`, `2`, and `3`;
- the project documents the exact transactions that would need official CU
  values once an endpoint is available.

What is still not done:

- official CU for:
  - private multisig deploy,
  - target deploy,
  - `create_multisig`,
  - `propose`,
  - `execute_private`.

## Reviewer Position

The correct reviewer-facing claim today is:

- LP-0002 has real proof-cycle measurements and real transaction inclusion
  evidence;
- LP-0002 does **not** yet have official hosted-testnet per-transaction CU,
  because the current Logos RPC/explorer used by this repository does not
  expose it.

That is the most defensible and accurate cost statement for the current stack.
