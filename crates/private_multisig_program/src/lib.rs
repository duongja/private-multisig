pub mod create_multisig;
pub mod execute_private;
pub mod propose;

pub use private_multisig_core::{
    hash_chunks, Hash32, PrivateMultisigInstruction, PrivateMultisigState, PrivateProposalState,
    PrivateProposalStatus,
};

pub const PROPOSAL_PDA_DOMAIN: &[u8] = b"logos.lp0002.proposal.pda.v1";

pub type ProgramResult<T> = Result<T, ProgramError>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum ProgramError {
    #[error("invalid account count")]
    InvalidAccountCount,
    #[error("account is already initialized")]
    AlreadyInitialized,
    #[error("invalid threshold")]
    InvalidThreshold,
    #[error("could not decode private multisig state")]
    DecodeState,
    #[error("could not decode proposal state")]
    DecodeProposal,
    #[error("create_key mismatch")]
    CreateKeyMismatch,
    #[error("proposal index mismatch")]
    ProposalIndexMismatch,
    #[error("proposal is not active")]
    ProposalNotActive,
    #[error("target account count mismatch")]
    TargetAccountCountMismatch,
    #[error("invalid aggregate threshold proof")]
    InvalidAggregateProof,
    #[error("account data exceeds LEZ data limit")]
    AccountDataTooLarge,
}

impl ProgramError {
    #[must_use]
    pub const fn code(self) -> u32 {
        match self {
            Self::InvalidAccountCount => 2000,
            Self::AlreadyInitialized => 2001,
            Self::InvalidThreshold => 2002,
            Self::DecodeState => 2003,
            Self::DecodeProposal => 2004,
            Self::CreateKeyMismatch => 2005,
            Self::ProposalIndexMismatch => 2006,
            Self::ProposalNotActive => 2007,
            Self::TargetAccountCountMismatch => 2008,
            Self::InvalidAggregateProof => 2009,
            Self::AccountDataTooLarge => 2010,
        }
    }
}

#[must_use]
pub const fn multisig_state_pda_seed(create_key: Hash32) -> Hash32 {
    create_key
}

#[must_use]
pub fn proposal_pda_seed(create_key: &Hash32, proposal_index: u64) -> Hash32 {
    hash_chunks(
        PROPOSAL_PDA_DOMAIN,
        &[
            b"private_ms_prop",
            create_key,
            &proposal_index.to_le_bytes(),
        ],
    )
}

// The current local SPEL framework still generates wrappers against the older
// `nssa_core` API. These handlers intentionally target the v0.2 `lee_core`
// program model directly; the thin SPEL/IDL wrapper can be restored once the
// framework exposes matching v0.2 types.

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn proposal_pda_seed_is_proposal_scoped() {
        let create_key = [42u8; 32];

        assert_eq!(multisig_state_pda_seed(create_key), create_key);
        assert_ne!(
            proposal_pda_seed(&create_key, 1),
            proposal_pda_seed(&create_key, 2)
        );
        assert_ne!(proposal_pda_seed(&create_key, 1), create_key);
    }
}
