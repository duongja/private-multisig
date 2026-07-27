# Resumable Approval Evidence - 2026-06-26 UTC

This evidence run verifies the LP-0002 reliability requirement that partial
approval progress is durable and resumable across client runs.

Command:

```bash
./scripts/resumable-approval-smoke.sh .local/resumable-approval/latest
```

The script creates a 2-of-3 multisig, writes one approval share to disk, proves
that a single approval cannot satisfy the threshold, later writes a second
approval share, verifies the aggregate, and confirms that reusing the same
approval twice is rejected by the proposal-scoped nullifier check.

## Result

| Field | Value |
| --- | --- |
| Run root | `.local/resumable-approval/latest` |
| Partial approval file | `.local/resumable-approval/latest/approval-a.json` |
| Resumed approval file | `.local/resumable-approval/latest/approval-c.json` |
| Single approval rejected | `true` |
| Duplicate nullifier rejected | `true` |
| Threshold aggregate verified | `true` |
| Approval count | `2` |
| Aggregate hash | `41e81ceadf5f46fd9923e470258109ccdc7286574bf5003acda95ed849bb9c7b` |

## Expected Failure Evidence

With only one approval:

```text
Error: approval count is below threshold
```

With the same approval reused twice:

```text
Error: duplicate proposal nullifier
```

## Prize Relevance

This proves:

- a partial approval set can be preserved as ordinary JSON artifacts;
- a later approval can resume the same proposal and complete the threshold;
- below-threshold execution fails closed;
- duplicate approval reuse is caught before execution.
