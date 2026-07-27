# Cost Evidence - 2026-06-26 UTC

This document records the current cost evidence for LP-0002.

The Logos team suggested the same measurement style used by
`fryorcraken/lez-signature-bench`: run real RISC0 proving with
`RISC0_DEV_MODE=0`, collect `ProveInfo.stats`, and keep end-to-end
transaction evidence separate.

## Command

```bash
./scripts/summarize-cost-evidence.py --out-dir .local/cost-evidence/latest
```

Inputs used:

- `.local/smoke/paths-20260625T222951Z/proof/proof-stats.json`
- `.local/localnet-evidence/20260625T233627Z/localnet-evidence.json`

## Private Threshold Proof

The 2-of-3 aggregate threshold proof was generated locally with
`RISC0_DEV_MODE=0`.

| Field | Value |
| --- | ---: |
| Total cycles | `262144` |
| User cycles | `198643` |
| Paging cycles | `38865` |
| Segments | `1` |
| Prove seconds | `93.86` |
| Image ID | `8330ee4acc3e4011a4ed8d919b119720f87eccd167c7abfe239021782bca9d7d` |

These are measured RISC0 proof-cycle values, not an official hosted-testnet CU
bill.

## LEZ Localnet Transactions

| Operation | Transaction hash | Included | Official CU |
| --- | --- | --- | --- |
| Private multisig deploy | `0f8f022f3150524ca404ce2b89309078c3c884a90f06164b8803ee0d63643586` | `true` | Not exposed by current RPC |
| Target deploy | `193a6eb47683fd9bdfa797f96ce956b9cd795f9aa68c1b4d6c704e90ec63e6a2` | `true` | Not exposed by current RPC |
| Create multisig | `b4ff34a4a7d398a0b24582957b4e481dfb381335755d5385d0c309c33ee7294f` | `true` | Not exposed by current RPC |
| Propose | `c6c05b2cee4957445dd9a037b1a3e8e45097bb4f0720e446db8548e5986f3e08` | `true` | Not exposed by current RPC |
| Execute private | `1a11405a66075f73a1e3448fe0097aa67ed50824a5113fa41ab6713c60542b2e` | `true` | Not exposed by current RPC |

The sequencer RPC currently used by this project exposes `sendTransaction`,
`getTransaction`, block lookup, account lookup, and program IDs. It does not
expose a transaction receipt containing per-transaction CU. In the current LEZ
state machine, public
program execution uses a `33,554,432` cycle session limit
(`MAX_NUM_CYCLES_PUBLIC_EXECUTION`), with a TODO noting that fees should make
this variable later.

## Submission Position

This is the cleanest evidence currently available without an official CU
endpoint:

- exact RISC0 proof stats for the private threshold proof;
- transaction hashes and inclusion evidence for all localnet operations;
- explicit note that official per-transaction CU is pending Logos RPC/explorer
  support or evaluator-approved mapping.

When hosted v0.2 testnet is back online, rerun localnet/testnet evidence and
replace the `Official CU` column if the explorer or RPC exposes official values.
