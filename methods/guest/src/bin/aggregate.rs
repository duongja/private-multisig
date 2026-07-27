use private_multisig_core::{aggregate_with_paths, AggregateWitness};

fn main() {
    let witness: AggregateWitness = risc0_zkvm::guest::env::read();
    let approval = aggregate_with_paths(&witness.config, &witness.proposal, &witness.approvals)
        .expect("invalid aggregate approval witness");

    risc0_zkvm::guest::env::commit(&approval);
}
