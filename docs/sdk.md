# SDK / Module Integration

The reusable SDK surface is the `private_multisig_core` crate, especially
`private_multisig_core::sdk`.

It is intentionally transport-neutral. A Logos module, CLI, or Basecamp helper
can use it to create local approval artifacts and LEZ program instructions, then
submit those instructions through its own wallet/sequencer integration.

## Main Types

- `PrivateMultisigClient`: bound to one multisig create key.
- `MemberEnrollment`: local member secret plus public commitment leaf.
- `PreparedMultisig`: LEZ `CreateMultisig` instruction plus member root.
- `PreparedProposal`: public proposal plus LEZ `Propose` instruction.
- `ApprovalShare`: private member approval artifact containing a nullifier.
- `PreparedExecution`: verified aggregate plus LEZ `ExecutePrivate` instruction.

## Example

```rust
use private_multisig_core::sdk::{PrivateMultisigClient, ProposalTemplate};
use private_multisig_core::{Hash32, PrivateMultisigInstruction};

fn h(byte: u8) -> Hash32 {
    [byte; 32]
}

let client = PrivateMultisigClient::new(h(42));

let alice = client.enroll_member(h(1), h(21));
let bob = client.enroll_member(h(2), h(22));
let carol = client.enroll_member(h(3), h(23));

let commitments = vec![
    alice.commitment.clone(),
    bob.commitment.clone(),
    carol.commitment.clone(),
];
let multisig = client.prepare_multisig(2, &commitments)?;

let proposal = client.prepare_proposal(ProposalTemplate {
    proposal_id: 1,
    target_program_id: [1, 2, 3, 4, 5, 6, 7, 8],
    target_instruction_data: vec![42],
    target_account_count: 0,
    pda_seeds: vec![],
    authorized_indices: vec![],
});

let approval_a = client.approve_proposal(&alice.secret, &proposal.proposal)?;
let approval_c = client.approve_proposal(&carol.secret, &proposal.proposal)?;

let execution = client.prepare_execution(
    &multisig.config,
    &proposal.proposal,
    &multisig.member_leaves,
    &[approval_a, approval_c],
)?;

match execution.execute_instruction {
    PrivateMultisigInstruction::ExecutePrivate { aggregate, .. } => {
        assert_eq!(aggregate.approval_count, 2);
    }
    _ => unreachable!(),
}
# Ok::<(), private_multisig_core::MultisigError>(())
```

## Integration Contract

The SDK guarantees that all module clients use the same:

- member commitment hash domain;
- proposal-scoped nullifier domain;
- Merkle root and Merkle path construction;
- aggregate hash construction;
- `PrivateMultisigInstruction` payloads consumed by the LEZ program guest.

It does not manage wallet keys, Basecamp state, or network submission. Those are
expected to live in the caller's Logos module. Store `ApprovalShare` files or
equivalent serialized records durably so partial approvals can be resumed after
restart.

## Failure Handling

All validation failures return `MultisigError`:

- `BelowThreshold` for incomplete approval sets;
- `DuplicateNullifier` for repeated approvals on the same proposal;
- `InvalidMerkleProof` or `MemberNotInRoot` for invalid membership data;
- `MultisigMismatch` and `ProposalMismatch` for cross-multisig replay attempts.

The LEZ program maps deterministic execution failures to documented program
codes in [docs/lez-program-scaffold.md](docs/lez-program-scaffold.md).
