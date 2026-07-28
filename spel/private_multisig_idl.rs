#![allow(dead_code, unused_imports, unused_variables)]

#[account_type]
pub struct PrivateMultisigState {
    pub create_key: [u8; 32],
    pub threshold: u8,
    pub member_count: u8,
    pub member_root: [u8; 32],
    pub transaction_index: u64,
}

#[account_type]
pub enum PrivateProposalStatus {
    Active,
    Executed,
    Cancelled,
}

#[account_type]
pub struct PrivateProposalState {
    pub index: u64,
    pub multisig_create_key: [u8; 32],
    pub target_program_id: [u32; 8],
    pub target_instruction_data: Vec<u32>,
    pub target_account_count: u8,
    pub pda_seeds: Vec<[u8; 32]>,
    pub authorized_indices: Vec<u8>,
    pub status: PrivateProposalStatus,
    pub executed_aggregate_hash: Option<[u8; 32]>,
    pub approval_count: u8,
}

#[account_type]
pub struct AggregateApproval {
    pub multisig_id: [u8; 32],
    pub proposal_id: u64,
    pub member_root: [u8; 32],
    pub threshold: u8,
    pub approval_count: u8,
    pub proposal_hash: [u8; 32],
    pub nullifiers: Vec<[u8; 32]>,
    pub aggregate_hash: [u8; 32],
}

#[lez_program(instruction = "private_multisig_core::PrivateMultisigInstruction")]
pub mod private_multisig {
    #[allow(unused_imports)]
    use super::*;

    #[instruction]
    pub fn create_multisig(
        #[account(init, mut, pda = arg("create_key"))] multisig_state: AccountWithMetadata,
        create_key: [u8; 32],
        threshold: u8,
        member_count: u8,
        member_root: [u8; 32],
    ) {
    }

    #[instruction]
    pub fn propose(
        #[account(mut)] multisig_state: AccountWithMetadata,
        #[account(init, mut, pda = [seed_const("private_ms_prop"), arg("create_key"), arg("proposal_index")])]
        proposal: AccountWithMetadata,
        create_key: [u8; 32],
        proposal_index: u64,
        target_program_id: [u32; 8],
        target_instruction_data: Vec<u32>,
        target_account_count: u8,
        pda_seeds: Vec<[u8; 32]>,
        authorized_indices: Vec<u8>,
    ) {
    }

    #[instruction]
    pub fn execute_private(
        #[account(mut)] multisig_state: AccountWithMetadata,
        #[account(mut, pda = [seed_const("private_ms_prop"), arg("create_key"), arg("proposal_index")])]
        proposal: AccountWithMetadata,
        #[account(mut)] target_accounts: Vec<AccountWithMetadata>,
        create_key: [u8; 32],
        proposal_index: u64,
        aggregate: AggregateApproval,
    ) {
    }
}
