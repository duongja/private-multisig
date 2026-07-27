# Hosted Testnet Evidence - 2026-06-26 UTC

This evidence run proves the LP-0002 private multisig flow against the hosted
LEZ v0.2 testnet after the testnet came back online.

Command:

```bash
export RISC0_DEV_MODE=0
./scripts/testnet-evidence.sh .local/testnet-evidence/20260626T172906Z
```

Equivalent runner command:

```bash
./target/debug/private_multisig_runner localnet-evidence \
  --sequencer https://testnet.lez.logos.co/ \
  --out-dir .local/testnet-evidence/20260626T172906Z \
  --target-program-binary /home/agate/Projects/logos/logos-execution-zone-v0.2.0-testnet/target/riscv32im-risc0-zkvm-elf/docker/hello_world.bin \
  --poll-seconds 180
```

The runner deployed the private multisig program, deployed a threshold-gated
target program, created a 2-of-3 multisig, created one proposal, generated two
private approvals, aggregated them, and executed the proposal on hosted testnet.

## Result

| Field | Value |
| --- | --- |
| Sequencer | `https://testnet.lez.logos.co/` |
| `RISC0_DEV_MODE` | `0` |
| Private multisig program ID | `faefc3b0e2c1f74aa6d355ba8c5898cb4130eb09bf552159335da21ce882fffa` |
| Target program ID | `60a99ce96d83a894ca24f2c4dd39137071f08d9c13277617dfa8d0ebf3a4cf4d` |
| Multisig state account | `DUhZHN6fe2fYRrwZMv3BKaUDXDLS3jrnYxnJa9BEVykN` |
| Proposal account | `feBn7NUpG96vUX5Yx53gp6x6Nm9LmeDaZdJrRZSLXeT` |
| Member root | `732016fae47c3c22296fdfff4dfa14dcedd02ee6f1a37e453b3c2568025d1d2b` |
| Aggregate hash | `da264165bbaa2bdd097242df8357dc55aa81f502374dc01437ca847413e1fb56` |

## Transactions

| Step | Hash | Included |
| --- | --- | --- |
| Private multisig deploy | `4caec74f66b336014c0f4ebd596d2a8a693d95c6ea7a295c6d73b7bf76e51695` | `true` |
| Target deploy | `193a6eb47683fd9bdfa797f96ce956b9cd795f9aa68c1b4d6c704e90ec63e6a2` | `true` |
| Create multisig | `264cf90ab8e5473c794d1bc3306d08e6d385d2819e1db310ee1fcded499aecab` | `true` |
| Create proposal | `15399561bf70236689bf1b4380d7cea80bbbd9e3ea74082737534721d6ad2c41` | `true` |
| Execute private | `26d0f98000d45e51170ba6eac273724aefcd315e4f326e283f1bb92d322327ac` | `true` |

## Final State

```json
{
  "multisig_transaction_index": 1,
  "proposal_status": "Executed",
  "proposal_approval_count": 2,
  "proposal_executed_aggregate_hash": "da264165bbaa2bdd097242df8357dc55aa81f502374dc01437ca847413e1fb56"
}
```

This confirms that the hosted testnet accepted the private aggregate approval,
marked the proposal executed, and persisted only the threshold result: approval
count and aggregate hash. Individual approving member identities are not stored
in the on-chain proposal state.

## Raw Evidence

The raw local run directory is:

```text
.local/testnet-evidence/20260626T172906Z
```

It contains:

- `private_multisig.bin`
- `localnet-evidence.json`
- `runner-output.json`

The runner subcommand is still named `localnet-evidence` because it was first
implemented for standalone sequencer evidence. It accepts a hosted sequencer URL
and the evidence above was generated against `https://testnet.lez.logos.co/`.

The repository has since been retargeted to the local
`logos-execution-zone-v0.2.0-testnet` checkout and now defaults to LEZ's
shipped `hello_world.bin` target program for fresh reruns.
