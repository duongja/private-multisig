# Cryptographic Design

This project uses an aggregate threshold proof model for LP-0002.

## Public State

The LEZ program stores:

- `multisig_id`
- `threshold`
- `member_count`
- `member_root`
- proposal metadata and execution status

It does not store member account IDs or voter lists.

## Member Leaf

Each member has a shielded LEZ private account identified by its nullifier public
key (`npk`) and a local membership secret.

```text
leaf = SHA256(
  "logos.lp0002.member.v1" ||
  multisig_id ||
  npk ||
  membership_secret
)
```

The leaf is committed into the multisig Merkle root. The raw `npk` and
membership secret stay client-side.

## Proposal Nullifier

Each approval produces a proposal-scoped nullifier:

```text
nullifier = SHA256(
  "logos.lp0002.nullifier.v1" ||
  multisig_id ||
  proposal_id ||
  membership_secret
)
```

The same member cannot be counted twice for one proposal because duplicate
nullifiers are rejected. Nullifiers are scoped to the proposal, so approvals
across different proposals are not linkable by nullifier equality.

## Aggregate Approval

The aggregate approval payload contains only:

- multisig id
- proposal id
- member root
- threshold
- approval count
- sorted nullifiers
- proposal hash

The RISC0 guest will prove that each nullifier corresponds to a distinct member
leaf in the committed member root and that at least `threshold` approvals were
included.

## Current Proof Implementation

The `methods/guest` RISC0 guest accepts an `AggregateWitness` containing:

- public multisig config;
- proposal metadata;
- approval shares;
- one Merkle path per approval.

It verifies each approval's Merkle path against the public `member_root`,
checks proposal-scoped nullifier uniqueness, recomputes the aggregate threshold
approval, and commits the `AggregateApproval` journal. The host CLI verifies the
receipt against the embedded method image ID and checks that the journal matches
the host-side aggregate result.

Run:

```bash
RISC0_DEV_MODE=0 ./scripts/local-threshold-smoke.sh
```

The proof journal is written to `.local/smoke/latest/proof/journal.json`, the
public witness summary is written to `.local/smoke/latest/proof/witness-public.json`,
and prover metrics are written to `.local/smoke/latest/proof/proof-stats.json`.

The local CLI uses member files to construct Merkle paths before proving. The
proof itself receives only the approval leaves and paths needed for the
threshold statement.
