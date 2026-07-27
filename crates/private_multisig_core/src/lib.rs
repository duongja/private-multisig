use borsh::{BorshDeserialize, BorshSerialize};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::BTreeSet;

pub const MEMBER_LEAF_DOMAIN: &[u8] = b"logos.lp0002.member.v1";
pub const NULLIFIER_DOMAIN: &[u8] = b"logos.lp0002.nullifier.v1";
pub const MERKLE_NODE_DOMAIN: &[u8] = b"logos.lp0002.merkle.node.v1";
pub const PROPOSAL_DOMAIN: &[u8] = b"logos.lp0002.proposal.v1";
pub const AGGREGATE_DOMAIN: &[u8] = b"logos.lp0002.aggregate.v1";

pub type Hash32 = [u8; 32];

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MemberSecret {
    pub multisig_id: Hash32,
    pub npk: Hash32,
    pub membership_secret: Hash32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MemberCommitment {
    pub leaf: Hash32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub struct MultisigConfig {
    pub multisig_id: Hash32,
    pub threshold: u8,
    pub member_count: u8,
    pub member_root: Hash32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub struct Proposal {
    pub multisig_id: Hash32,
    pub proposal_id: u64,
    pub target_program_id: [u32; 8],
    pub target_instruction_data: Vec<u32>,
    pub target_account_count: u8,
    pub pda_seeds: Vec<Hash32>,
    pub authorized_indices: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub struct ApprovalShare {
    pub multisig_id: Hash32,
    pub proposal_id: u64,
    pub member_leaf: Hash32,
    pub nullifier: Hash32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub struct MerklePathNode {
    pub sibling: Hash32,
    pub sibling_is_left: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub struct MerkleProof {
    pub leaf: Hash32,
    pub path: Vec<MerklePathNode>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub struct ApprovalWitness {
    pub approval: ApprovalShare,
    pub membership_proof: MerkleProof,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub struct AggregateWitness {
    pub config: MultisigConfig,
    pub proposal: Proposal,
    pub approvals: Vec<ApprovalWitness>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub struct AggregateApproval {
    pub multisig_id: Hash32,
    pub proposal_id: u64,
    pub member_root: Hash32,
    pub threshold: u8,
    pub approval_count: u8,
    pub proposal_hash: Hash32,
    pub nullifiers: Vec<Hash32>,
    pub aggregate_hash: Hash32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PrivateMultisigInstruction {
    CreateMultisig {
        create_key: Hash32,
        threshold: u8,
        member_count: u8,
        member_root: Hash32,
    },
    Propose {
        create_key: Hash32,
        proposal_index: u64,
        target_program_id: [u32; 8],
        target_instruction_data: Vec<u32>,
        target_account_count: u8,
        pda_seeds: Vec<Hash32>,
        authorized_indices: Vec<u8>,
    },
    ExecutePrivate {
        create_key: Hash32,
        proposal_index: u64,
        aggregate: AggregateApproval,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, BorshSerialize, BorshDeserialize)]
pub struct PrivateMultisigState {
    pub create_key: Hash32,
    pub threshold: u8,
    pub member_count: u8,
    pub member_root: Hash32,
    pub transaction_index: u64,
}

impl PrivateMultisigState {
    pub fn new(create_key: Hash32, threshold: u8, member_count: u8, member_root: Hash32) -> Self {
        Self {
            create_key,
            threshold,
            member_count,
            member_root,
            transaction_index: 0,
        }
    }

    pub fn config(&self) -> MultisigConfig {
        MultisigConfig {
            multisig_id: self.create_key,
            threshold: self.threshold,
            member_count: self.member_count,
            member_root: self.member_root,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, BorshSerialize, BorshDeserialize)]
pub enum PrivateProposalStatus {
    Active,
    Executed,
    Cancelled,
}

#[derive(Debug, Clone, PartialEq, Eq, BorshSerialize, BorshDeserialize)]
pub struct PrivateProposalState {
    pub index: u64,
    pub multisig_create_key: Hash32,
    pub target_program_id: [u32; 8],
    pub target_instruction_data: Vec<u32>,
    pub target_account_count: u8,
    pub pda_seeds: Vec<Hash32>,
    pub authorized_indices: Vec<u8>,
    pub status: PrivateProposalStatus,
    pub executed_aggregate_hash: Option<Hash32>,
    pub approval_count: u8,
}

impl PrivateProposalState {
    pub fn new(
        index: u64,
        multisig_create_key: Hash32,
        target_program_id: [u32; 8],
        target_instruction_data: Vec<u32>,
        target_account_count: u8,
        pda_seeds: Vec<Hash32>,
        authorized_indices: Vec<u8>,
    ) -> Self {
        Self {
            index,
            multisig_create_key,
            target_program_id,
            target_instruction_data,
            target_account_count,
            pda_seeds,
            authorized_indices,
            status: PrivateProposalStatus::Active,
            executed_aggregate_hash: None,
            approval_count: 0,
        }
    }

    pub fn as_proposal(&self) -> Proposal {
        Proposal {
            multisig_id: self.multisig_create_key,
            proposal_id: self.index,
            target_program_id: self.target_program_id,
            target_instruction_data: self.target_instruction_data.clone(),
            target_account_count: self.target_account_count,
            pda_seeds: self.pda_seeds.clone(),
            authorized_indices: self.authorized_indices.clone(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum MultisigError {
    #[error("threshold must satisfy 1 <= threshold <= member_count")]
    InvalidThreshold,
    #[error("approval multisig id does not match proposal/config")]
    MultisigMismatch,
    #[error("approval proposal id does not match proposal")]
    ProposalMismatch,
    #[error("member leaf is not in the configured member root")]
    MemberNotInRoot,
    #[error("approval count is below threshold")]
    BelowThreshold,
    #[error("duplicate proposal nullifier")]
    DuplicateNullifier,
    #[error("invalid member Merkle proof")]
    InvalidMerkleProof,
    #[error("too many members or approvals for u8 count")]
    CountOverflow,
}

pub fn hash_chunks(domain: &[u8], chunks: &[&[u8]]) -> Hash32 {
    let mut hasher = Sha256::new();
    hasher.update((domain.len() as u64).to_le_bytes());
    hasher.update(domain);
    for chunk in chunks {
        hasher.update((chunk.len() as u64).to_le_bytes());
        hasher.update(chunk);
    }
    hasher.finalize().into()
}

pub fn to_hex(value: &Hash32) -> String {
    hex::encode(value)
}

pub fn from_hex(value: &str) -> Result<Hash32, hex::FromHexError> {
    let bytes = hex::decode(value.trim_start_matches("0x"))?;
    let mut out = [0u8; 32];
    out.copy_from_slice(&bytes);
    Ok(out)
}

pub fn member_leaf(multisig_id: &Hash32, npk: &Hash32, membership_secret: &Hash32) -> Hash32 {
    hash_chunks(MEMBER_LEAF_DOMAIN, &[multisig_id, npk, membership_secret])
}

pub fn proposal_nullifier(
    multisig_id: &Hash32,
    proposal_id: u64,
    membership_secret: &Hash32,
) -> Hash32 {
    hash_chunks(
        NULLIFIER_DOMAIN,
        &[multisig_id, &proposal_id.to_le_bytes(), membership_secret],
    )
}

pub fn proposal_hash(proposal: &Proposal) -> Hash32 {
    let encoded = encode_proposal(proposal);
    hash_chunks(PROPOSAL_DOMAIN, &[&encoded])
}

fn encode_proposal(proposal: &Proposal) -> Vec<u8> {
    let mut out = Vec::new();
    out.extend_from_slice(&proposal.multisig_id);
    out.extend_from_slice(&proposal.proposal_id.to_le_bytes());
    for word in proposal.target_program_id {
        out.extend_from_slice(&word.to_le_bytes());
    }
    encode_u32_vec(&mut out, &proposal.target_instruction_data);
    out.push(proposal.target_account_count);
    out.extend_from_slice(&(proposal.pda_seeds.len() as u32).to_le_bytes());
    for seed in &proposal.pda_seeds {
        out.extend_from_slice(seed);
    }
    out.extend_from_slice(&(proposal.authorized_indices.len() as u32).to_le_bytes());
    out.extend_from_slice(&proposal.authorized_indices);
    out
}

fn encode_u32_vec(out: &mut Vec<u8>, values: &[u32]) {
    out.extend_from_slice(&(values.len() as u32).to_le_bytes());
    for value in values {
        out.extend_from_slice(&value.to_le_bytes());
    }
}

pub fn merkle_root(leaves: &[Hash32]) -> Hash32 {
    if leaves.is_empty() {
        return hash_chunks(MERKLE_NODE_DOMAIN, &[b"empty"]);
    }

    let mut level = leaves.to_vec();
    level.sort_unstable();
    while level.len() > 1 {
        let mut next = Vec::with_capacity((level.len() + 1) / 2);
        for pair in level.chunks(2) {
            let left = pair[0];
            let right = if pair.len() == 2 { pair[1] } else { pair[0] };
            next.push(hash_chunks(MERKLE_NODE_DOMAIN, &[&left, &right]));
        }
        level = next;
    }
    level[0]
}

pub fn merkle_proof(leaves: &[Hash32], leaf: &Hash32) -> Result<MerkleProof, MultisigError> {
    if leaves.is_empty() {
        return Err(MultisigError::MemberNotInRoot);
    }

    let mut level = leaves.to_vec();
    level.sort_unstable();
    let mut index = level
        .iter()
        .position(|candidate| candidate == leaf)
        .ok_or(MultisigError::MemberNotInRoot)?;
    let mut path = Vec::new();

    while level.len() > 1 {
        let sibling_index = if index % 2 == 0 {
            if index + 1 < level.len() {
                index + 1
            } else {
                index
            }
        } else {
            index - 1
        };
        path.push(MerklePathNode {
            sibling: level[sibling_index],
            sibling_is_left: index % 2 == 1,
        });

        let mut next = Vec::with_capacity((level.len() + 1) / 2);
        for pair in level.chunks(2) {
            let left = pair[0];
            let right = if pair.len() == 2 { pair[1] } else { pair[0] };
            next.push(hash_chunks(MERKLE_NODE_DOMAIN, &[&left, &right]));
        }
        index /= 2;
        level = next;
    }

    Ok(MerkleProof { leaf: *leaf, path })
}

pub fn verify_merkle_proof(root: &Hash32, proof: &MerkleProof) -> bool {
    let mut node = proof.leaf;
    for path_node in &proof.path {
        node = if path_node.sibling_is_left {
            hash_chunks(MERKLE_NODE_DOMAIN, &[&path_node.sibling, &node])
        } else {
            hash_chunks(MERKLE_NODE_DOMAIN, &[&node, &path_node.sibling])
        };
    }
    &node == root
}

pub fn build_config(
    multisig_id: Hash32,
    threshold: u8,
    member_leaves: &[Hash32],
) -> Result<MultisigConfig, MultisigError> {
    let member_count =
        u8::try_from(member_leaves.len()).map_err(|_| MultisigError::CountOverflow)?;
    if threshold == 0 || threshold > member_count {
        return Err(MultisigError::InvalidThreshold);
    }
    Ok(MultisigConfig {
        multisig_id,
        threshold,
        member_count,
        member_root: merkle_root(member_leaves),
    })
}

pub fn approve(secret: &MemberSecret, proposal: &Proposal) -> Result<ApprovalShare, MultisigError> {
    if secret.multisig_id != proposal.multisig_id {
        return Err(MultisigError::MultisigMismatch);
    }
    Ok(ApprovalShare {
        multisig_id: secret.multisig_id,
        proposal_id: proposal.proposal_id,
        member_leaf: member_leaf(&secret.multisig_id, &secret.npk, &secret.membership_secret),
        nullifier: proposal_nullifier(
            &secret.multisig_id,
            proposal.proposal_id,
            &secret.membership_secret,
        ),
    })
}

pub fn aggregate(
    config: &MultisigConfig,
    proposal: &Proposal,
    all_member_leaves: &[Hash32],
    approvals: &[ApprovalShare],
) -> Result<AggregateApproval, MultisigError> {
    let approval_witnesses = approvals
        .iter()
        .map(|approval| {
            let membership_proof = merkle_proof(all_member_leaves, &approval.member_leaf)?;
            Ok(ApprovalWitness {
                approval: approval.clone(),
                membership_proof,
            })
        })
        .collect::<Result<Vec<_>, MultisigError>>()?;
    aggregate_with_paths(config, proposal, &approval_witnesses)
}

pub fn aggregate_with_paths(
    config: &MultisigConfig,
    proposal: &Proposal,
    approvals: &[ApprovalWitness],
) -> Result<AggregateApproval, MultisigError> {
    if config.multisig_id != proposal.multisig_id {
        return Err(MultisigError::MultisigMismatch);
    }

    let mut nullifiers = BTreeSet::new();
    for approval_witness in approvals {
        let approval = &approval_witness.approval;
        if approval.multisig_id != config.multisig_id {
            return Err(MultisigError::MultisigMismatch);
        }
        if approval.proposal_id != proposal.proposal_id {
            return Err(MultisigError::ProposalMismatch);
        }
        if approval_witness.membership_proof.leaf != approval.member_leaf {
            return Err(MultisigError::InvalidMerkleProof);
        }
        if !verify_merkle_proof(&config.member_root, &approval_witness.membership_proof) {
            return Err(MultisigError::InvalidMerkleProof);
        }
        if !nullifiers.insert(approval.nullifier) {
            return Err(MultisigError::DuplicateNullifier);
        }
    }

    let approval_count =
        u8::try_from(nullifiers.len()).map_err(|_| MultisigError::CountOverflow)?;
    if approval_count < config.threshold {
        return Err(MultisigError::BelowThreshold);
    }

    let nullifiers: Vec<Hash32> = nullifiers.into_iter().collect();
    let proposal_hash = proposal_hash(proposal);
    let aggregate_hash = aggregate_hash(
        &config.multisig_id,
        proposal.proposal_id,
        &config.member_root,
        config.threshold,
        approval_count,
        &proposal_hash,
        &nullifiers,
    );

    Ok(AggregateApproval {
        multisig_id: config.multisig_id,
        proposal_id: proposal.proposal_id,
        member_root: config.member_root,
        threshold: config.threshold,
        approval_count,
        proposal_hash,
        nullifiers,
        aggregate_hash,
    })
}

pub fn verify_aggregate(
    config: &MultisigConfig,
    proposal: &Proposal,
    aggregate: &AggregateApproval,
) -> Result<(), MultisigError> {
    if config.multisig_id != proposal.multisig_id || aggregate.multisig_id != config.multisig_id {
        return Err(MultisigError::MultisigMismatch);
    }
    if aggregate.proposal_id != proposal.proposal_id {
        return Err(MultisigError::ProposalMismatch);
    }
    if aggregate.member_root != config.member_root {
        return Err(MultisigError::MemberNotInRoot);
    }
    if aggregate.threshold != config.threshold || aggregate.approval_count < config.threshold {
        return Err(MultisigError::BelowThreshold);
    }
    let mut seen = BTreeSet::new();
    for nullifier in &aggregate.nullifiers {
        if !seen.insert(*nullifier) {
            return Err(MultisigError::DuplicateNullifier);
        }
    }
    if aggregate.approval_count as usize != aggregate.nullifiers.len() {
        return Err(MultisigError::BelowThreshold);
    }
    let expected_proposal_hash = proposal_hash(proposal);
    if aggregate.proposal_hash != expected_proposal_hash {
        return Err(MultisigError::ProposalMismatch);
    }
    let expected_aggregate_hash = aggregate_hash(
        &aggregate.multisig_id,
        aggregate.proposal_id,
        &aggregate.member_root,
        aggregate.threshold,
        aggregate.approval_count,
        &aggregate.proposal_hash,
        &aggregate.nullifiers,
    );
    if aggregate.aggregate_hash != expected_aggregate_hash {
        return Err(MultisigError::ProposalMismatch);
    }
    Ok(())
}

fn aggregate_hash(
    multisig_id: &Hash32,
    proposal_id: u64,
    member_root: &Hash32,
    threshold: u8,
    approval_count: u8,
    proposal_hash: &Hash32,
    nullifiers: &[Hash32],
) -> Hash32 {
    let mut encoded_nullifiers = Vec::with_capacity(nullifiers.len() * 32);
    for nullifier in nullifiers {
        encoded_nullifiers.extend_from_slice(nullifier);
    }
    hash_chunks(
        AGGREGATE_DOMAIN,
        &[
            multisig_id,
            &proposal_id.to_le_bytes(),
            member_root,
            &[threshold],
            &[approval_count],
            proposal_hash,
            &encoded_nullifiers,
        ],
    )
}

pub mod sdk {
    use super::{
        aggregate, approve, build_config, member_leaf, merkle_proof, AggregateApproval,
        ApprovalShare, ApprovalWitness, Hash32, MemberCommitment, MemberSecret, MerkleProof,
        MultisigConfig, MultisigError, PrivateMultisigInstruction, Proposal,
    };

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct MemberEnrollment {
        pub secret: MemberSecret,
        pub commitment: MemberCommitment,
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct ProposalTemplate {
        pub proposal_id: u64,
        pub target_program_id: [u32; 8],
        pub target_instruction_data: Vec<u32>,
        pub target_account_count: u8,
        pub pda_seeds: Vec<Hash32>,
        pub authorized_indices: Vec<u8>,
    }

    impl ProposalTemplate {
        #[must_use]
        pub fn into_proposal(self, multisig_id: Hash32) -> Proposal {
            Proposal {
                multisig_id,
                proposal_id: self.proposal_id,
                target_program_id: self.target_program_id,
                target_instruction_data: self.target_instruction_data,
                target_account_count: self.target_account_count,
                pda_seeds: self.pda_seeds,
                authorized_indices: self.authorized_indices,
            }
        }
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct PreparedMultisig {
        pub config: MultisigConfig,
        pub member_leaves: Vec<Hash32>,
        pub create_instruction: PrivateMultisigInstruction,
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct PreparedProposal {
        pub proposal: Proposal,
        pub propose_instruction: PrivateMultisigInstruction,
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct PreparedExecution {
        pub aggregate: AggregateApproval,
        pub execute_instruction: PrivateMultisigInstruction,
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct PrivateMultisigClient {
        pub multisig_id: Hash32,
    }

    impl PrivateMultisigClient {
        #[must_use]
        pub const fn new(multisig_id: Hash32) -> Self {
            Self { multisig_id }
        }

        #[must_use]
        pub fn enroll_member(&self, npk: Hash32, membership_secret: Hash32) -> MemberEnrollment {
            let secret = MemberSecret {
                multisig_id: self.multisig_id,
                npk,
                membership_secret,
            };
            let leaf = member_leaf(&secret.multisig_id, &secret.npk, &secret.membership_secret);
            MemberEnrollment {
                secret,
                commitment: MemberCommitment { leaf },
            }
        }

        pub fn prepare_multisig(
            &self,
            threshold: u8,
            member_commitments: &[MemberCommitment],
        ) -> Result<PreparedMultisig, MultisigError> {
            let member_leaves: Vec<Hash32> = member_commitments
                .iter()
                .map(|commitment| commitment.leaf)
                .collect();
            let config = build_config(self.multisig_id, threshold, &member_leaves)?;
            let create_instruction = PrivateMultisigInstruction::CreateMultisig {
                create_key: self.multisig_id,
                threshold: config.threshold,
                member_count: config.member_count,
                member_root: config.member_root,
            };
            Ok(PreparedMultisig {
                config,
                member_leaves,
                create_instruction,
            })
        }

        #[must_use]
        pub fn prepare_proposal(&self, template: ProposalTemplate) -> PreparedProposal {
            let proposal = template.into_proposal(self.multisig_id);
            let propose_instruction = PrivateMultisigInstruction::Propose {
                create_key: self.multisig_id,
                proposal_index: proposal.proposal_id,
                target_program_id: proposal.target_program_id,
                target_instruction_data: proposal.target_instruction_data.clone(),
                target_account_count: proposal.target_account_count,
                pda_seeds: proposal.pda_seeds.clone(),
                authorized_indices: proposal.authorized_indices.clone(),
            };
            PreparedProposal {
                proposal,
                propose_instruction,
            }
        }

        pub fn approve_proposal(
            &self,
            member: &MemberSecret,
            proposal: &Proposal,
        ) -> Result<ApprovalShare, MultisigError> {
            approve(member, proposal)
        }

        pub fn membership_proof(
            &self,
            member_leaves: &[Hash32],
            approval: &ApprovalShare,
        ) -> Result<MerkleProof, MultisigError> {
            merkle_proof(member_leaves, &approval.member_leaf)
        }

        pub fn approval_witness(
            &self,
            member_leaves: &[Hash32],
            approval: ApprovalShare,
        ) -> Result<ApprovalWitness, MultisigError> {
            let membership_proof = self.membership_proof(member_leaves, &approval)?;
            Ok(ApprovalWitness {
                approval,
                membership_proof,
            })
        }

        pub fn prepare_execution(
            &self,
            config: &MultisigConfig,
            proposal: &Proposal,
            member_leaves: &[Hash32],
            approvals: &[ApprovalShare],
        ) -> Result<PreparedExecution, MultisigError> {
            let aggregate = aggregate(config, proposal, member_leaves, approvals)?;
            let execute_instruction = PrivateMultisigInstruction::ExecutePrivate {
                create_key: self.multisig_id,
                proposal_index: proposal.proposal_id,
                aggregate: aggregate.clone(),
            };
            Ok(PreparedExecution {
                aggregate,
                execute_instruction,
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn h(byte: u8) -> Hash32 {
        [byte; 32]
    }

    fn member(multisig_id: Hash32, byte: u8) -> MemberSecret {
        MemberSecret {
            multisig_id,
            npk: h(byte),
            membership_secret: h(byte + 20),
        }
    }

    fn proposal(multisig_id: Hash32) -> Proposal {
        Proposal {
            multisig_id,
            proposal_id: 7,
            target_program_id: [1, 2, 3, 4, 5, 6, 7, 8],
            target_instruction_data: vec![11, 22, 33],
            target_account_count: 2,
            pda_seeds: vec![h(9)],
            authorized_indices: vec![0],
        }
    }

    #[test]
    fn two_of_three_aggregate_verifies() {
        let multisig_id = h(42);
        let members = vec![
            member(multisig_id, 1),
            member(multisig_id, 2),
            member(multisig_id, 3),
        ];
        let leaves: Vec<Hash32> = members
            .iter()
            .map(|m| member_leaf(&m.multisig_id, &m.npk, &m.membership_secret))
            .collect();
        let config = build_config(multisig_id, 2, &leaves).unwrap();
        let proposal = proposal(multisig_id);
        let approvals = vec![
            approve(&members[0], &proposal).unwrap(),
            approve(&members[2], &proposal).unwrap(),
        ];

        let aggregate = aggregate(&config, &proposal, &leaves, &approvals).unwrap();

        assert_eq!(aggregate.approval_count, 2);
        verify_aggregate(&config, &proposal, &aggregate).unwrap();
    }

    #[test]
    fn below_threshold_fails() {
        let multisig_id = h(42);
        let members = vec![
            member(multisig_id, 1),
            member(multisig_id, 2),
            member(multisig_id, 3),
        ];
        let leaves: Vec<Hash32> = members
            .iter()
            .map(|m| member_leaf(&m.multisig_id, &m.npk, &m.membership_secret))
            .collect();
        let config = build_config(multisig_id, 2, &leaves).unwrap();
        let proposal = proposal(multisig_id);
        let approvals = vec![approve(&members[0], &proposal).unwrap()];

        let err = aggregate(&config, &proposal, &leaves, &approvals).unwrap_err();

        assert_eq!(err, MultisigError::BelowThreshold);
    }

    #[test]
    fn duplicate_nullifier_fails() {
        let multisig_id = h(42);
        let members = vec![
            member(multisig_id, 1),
            member(multisig_id, 2),
            member(multisig_id, 3),
        ];
        let leaves: Vec<Hash32> = members
            .iter()
            .map(|m| member_leaf(&m.multisig_id, &m.npk, &m.membership_secret))
            .collect();
        let config = build_config(multisig_id, 2, &leaves).unwrap();
        let proposal = proposal(multisig_id);
        let approval = approve(&members[0], &proposal).unwrap();
        let approvals = vec![approval.clone(), approval];

        let err = aggregate(&config, &proposal, &leaves, &approvals).unwrap_err();

        assert_eq!(err, MultisigError::DuplicateNullifier);
    }

    #[test]
    fn aggregate_with_paths_verifies_membership_without_all_leaves() {
        let multisig_id = h(42);
        let members = vec![
            member(multisig_id, 1),
            member(multisig_id, 2),
            member(multisig_id, 3),
        ];
        let leaves: Vec<Hash32> = members
            .iter()
            .map(|m| member_leaf(&m.multisig_id, &m.npk, &m.membership_secret))
            .collect();
        let config = build_config(multisig_id, 2, &leaves).unwrap();
        let proposal = proposal(multisig_id);
        let approvals: Vec<ApprovalWitness> = [0usize, 2]
            .iter()
            .map(|idx| {
                let approval = approve(&members[*idx], &proposal).unwrap();
                let membership_proof = merkle_proof(&leaves, &approval.member_leaf).unwrap();
                ApprovalWitness {
                    approval,
                    membership_proof,
                }
            })
            .collect();

        let aggregate = aggregate_with_paths(&config, &proposal, &approvals).unwrap();

        assert_eq!(aggregate.approval_count, 2);
        verify_aggregate(&config, &proposal, &aggregate).unwrap();
    }

    #[test]
    fn tampered_merkle_path_fails() {
        let multisig_id = h(42);
        let members = vec![
            member(multisig_id, 1),
            member(multisig_id, 2),
            member(multisig_id, 3),
        ];
        let leaves: Vec<Hash32> = members
            .iter()
            .map(|m| member_leaf(&m.multisig_id, &m.npk, &m.membership_secret))
            .collect();
        let config = build_config(multisig_id, 2, &leaves).unwrap();
        let proposal = proposal(multisig_id);
        let mut approvals: Vec<ApprovalWitness> = [0usize, 2]
            .iter()
            .map(|idx| {
                let approval = approve(&members[*idx], &proposal).unwrap();
                let membership_proof = merkle_proof(&leaves, &approval.member_leaf).unwrap();
                ApprovalWitness {
                    approval,
                    membership_proof,
                }
            })
            .collect();
        approvals[0].membership_proof.path[0].sibling[0] ^= 0xff;

        let err = aggregate_with_paths(&config, &proposal, &approvals).unwrap_err();

        assert_eq!(err, MultisigError::InvalidMerkleProof);
    }

    #[test]
    fn sdk_prepares_full_private_multisig_flow() {
        let multisig_id = h(90);
        let client = sdk::PrivateMultisigClient::new(multisig_id);
        let members = vec![
            client.enroll_member(h(1), h(21)),
            client.enroll_member(h(2), h(22)),
            client.enroll_member(h(3), h(23)),
        ];
        let commitments: Vec<MemberCommitment> = members
            .iter()
            .map(|member| member.commitment.clone())
            .collect();
        let prepared = client.prepare_multisig(2, &commitments).unwrap();

        let proposal = client.prepare_proposal(sdk::ProposalTemplate {
            proposal_id: 5,
            target_program_id: [8, 7, 6, 5, 4, 3, 2, 1],
            target_instruction_data: vec![100, 200],
            target_account_count: 1,
            pda_seeds: vec![h(55)],
            authorized_indices: vec![0],
        });
        let approvals = vec![
            client
                .approve_proposal(&members[0].secret, &proposal.proposal)
                .unwrap(),
            client
                .approve_proposal(&members[2].secret, &proposal.proposal)
                .unwrap(),
        ];
        let execution = client
            .prepare_execution(
                &prepared.config,
                &proposal.proposal,
                &prepared.member_leaves,
                &approvals,
            )
            .unwrap();

        assert!(matches!(
            prepared.create_instruction,
            PrivateMultisigInstruction::CreateMultisig { threshold: 2, .. }
        ));
        assert!(matches!(
            proposal.propose_instruction,
            PrivateMultisigInstruction::Propose {
                proposal_index: 5,
                ..
            }
        ));
        assert_eq!(execution.aggregate.approval_count, 2);
        assert!(matches!(
            execution.execute_instruction,
            PrivateMultisigInstruction::ExecutePrivate {
                proposal_index: 5,
                ..
            }
        ));
        verify_aggregate(&prepared.config, &proposal.proposal, &execution.aggregate).unwrap();
    }
}
