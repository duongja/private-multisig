# Localnet Evidence - 2026-06-25 UTC

This evidence run proves the LP-0002 private multisig flow against a real LEZ
v0.2 standalone sequencer.

Command:

```bash
export RISC0_DEV_MODE=0
./scripts/localnet-evidence.sh .local/localnet-evidence/20260625T233627Z
```

The script rebuilt `sequencer_service` with `--features standalone`, started
localnet on `http://127.0.0.1:3040`, deployed the private multisig program,
deployed the threshold-gated target program, created a 2-of-3 multisig, created
a proposal, and executed it after aggregate private approval.

## Result

| Field | Value |
| --- | --- |
| Sequencer | `http://127.0.0.1:3040` |
| `RISC0_DEV_MODE` | `0` |
| Private multisig program ID | `924226e8b3f885853472df950d8069f83a3906e3462fd35d65f679e4e9ac3592` |
| Target program ID | `60a99ce96d83a894ca24f2c4dd39137071f08d9c13277617dfa8d0ebf3a4cf4d` |
| Multisig state account | `Dnz39Jvc9pVHVyRpUBPuAC6EL2TMxm4r59tTb7Y8bs1r` |
| Proposal account | `43eEwkSVBkgejMKdsh4q3Dgqa8d9XL9zF4bj2qeWp2CM` |
| Member root | `b75098b48814c2e05c8a7bf8bedfc566212447d35294946a730acd3f685a7a7c` |
| Aggregate hash | `a00c8b7e0b3f074cb3cbc0e471190524a42c171803e772b19762bb7f02e85db1` |

## Transactions

| Step | Hash | Included |
| --- | --- | --- |
| Private multisig deploy | `0f8f022f3150524ca404ce2b89309078c3c884a90f06164b8803ee0d63643586` | `true` |
| Target deploy | `193a6eb47683fd9bdfa797f96ce956b9cd795f9aa68c1b4d6c704e90ec63e6a2` | `true` |
| Create multisig | `b4ff34a4a7d398a0b24582957b4e481dfb381335755d5385d0c309c33ee7294f` | `true` |
| Create proposal | `c6c05b2cee4957445dd9a037b1a3e8e45097bb4f0720e446db8548e5986f3e08` | `true` |
| Execute private | `1a11405a66075f73a1e3448fe0097aa67ed50824a5113fa41ab6713c60542b2e` | `true` |

## Final State

```json
{
  "multisig_transaction_index": 1,
  "proposal_status": "Executed",
  "proposal_approval_count": 2,
  "proposal_executed_aggregate_hash": "a00c8b7e0b3f074cb3cbc0e471190524a42c171803e772b19762bb7f02e85db1"
}
```

This confirms that the on-chain program accepted the private aggregate approval,
marked the proposal executed, and emitted the configured threshold-gated target
call.

## Operational Note

The standalone sequencer must be built with:

```bash
cargo build --release --manifest-path "$LEZ_REPO/Cargo.toml" --features standalone -p sequencer_service
```

When built without `--features standalone`, the same binary waits for a Bedrock
node at `localhost:18080` and local scaffold readiness times out.
