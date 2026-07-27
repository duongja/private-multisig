use borsh::{from_slice, to_vec};
use lee_core::account::{Account, AccountWithMetadata};
use lee_core::program::{ChainedCall, ProgramId};
use private_multisig_core::{Hash32, PrivateMultisigState, PrivateProposalState};

use crate::{ProgramError, ProgramResult};

pub fn handle(
    accounts: &[AccountWithMetadata],
    create_key: Hash32,
    proposal_index: u64,
    target_program_id: ProgramId,
    target_instruction_data: Vec<u32>,
    target_account_count: u8,
    pda_seeds: Vec<Hash32>,
    authorized_indices: Vec<u8>,
) -> ProgramResult<(Vec<Account>, Vec<ChainedCall>)> {
    if accounts.len() != 2 {
        return Err(ProgramError::InvalidAccountCount);
    }
    if accounts[1].account != Account::default() {
        return Err(ProgramError::AlreadyInitialized);
    }

    let mut state: PrivateMultisigState = from_slice(&Vec::from(accounts[0].account.data.clone()))
        .map_err(|_| ProgramError::DecodeState)?;
    if state.create_key != create_key {
        return Err(ProgramError::CreateKeyMismatch);
    }
    if state.transaction_index + 1 != proposal_index {
        return Err(ProgramError::ProposalIndexMismatch);
    }
    state.transaction_index = proposal_index;

    let proposal = PrivateProposalState::new(
        proposal_index,
        create_key,
        target_program_id,
        target_instruction_data,
        target_account_count,
        pda_seeds,
        authorized_indices,
    );

    let mut state_post = accounts[0].account.clone();
    state_post.data = to_vec(&state)
        .expect("state serialization")
        .try_into()
        .map_err(|_| ProgramError::AccountDataTooLarge)?;

    let mut proposal_post = Account::default();
    proposal_post.data = to_vec(&proposal)
        .expect("proposal serialization")
        .try_into()
        .map_err(|_| ProgramError::AccountDataTooLarge)?;

    Ok((vec![state_post, proposal_post], vec![]))
}

#[cfg(test)]
mod tests {
    use super::*;
    use borsh::to_vec;
    use lee_core::account::AccountId;
    use private_multisig_core::PrivateProposalStatus;

    fn h(byte: u8) -> Hash32 {
        [byte; 32]
    }

    fn account_with_data(id_byte: u8, data: Vec<u8>) -> AccountWithMetadata {
        let mut account = Account::default();
        account.data = data.try_into().expect("test account data fits");
        AccountWithMetadata {
            account_id: AccountId::new(h(id_byte)),
            account,
            is_authorized: false,
        }
    }

    fn empty_account(id_byte: u8) -> AccountWithMetadata {
        AccountWithMetadata {
            account_id: AccountId::new(h(id_byte)),
            account: Account::default(),
            is_authorized: false,
        }
    }

    #[test]
    fn creates_active_proposal_and_advances_state_index() {
        let create_key = h(42);
        let state = PrivateMultisigState::new(create_key, 2, 3, h(7));
        let state_account = account_with_data(1, to_vec(&state).unwrap());
        let proposal_account = empty_account(2);
        let target_program_id = [1, 2, 3, 4, 5, 6, 7, 8];

        let (accounts, calls) = handle(
            &[state_account, proposal_account],
            create_key,
            1,
            target_program_id,
            vec![11, 22, 33],
            2,
            vec![h(9)],
            vec![1],
        )
        .unwrap();

        assert!(calls.is_empty());
        let state_post: PrivateMultisigState =
            borsh::from_slice(&Vec::from(accounts[0].data.clone())).unwrap();
        assert_eq!(state_post.transaction_index, 1);

        let proposal: PrivateProposalState =
            borsh::from_slice(&Vec::from(accounts[1].data.clone())).unwrap();
        assert_eq!(proposal.index, 1);
        assert_eq!(proposal.multisig_create_key, create_key);
        assert_eq!(proposal.target_program_id, target_program_id);
        assert_eq!(proposal.target_instruction_data, vec![11, 22, 33]);
        assert_eq!(proposal.target_account_count, 2);
        assert_eq!(proposal.pda_seeds, vec![h(9)]);
        assert_eq!(proposal.authorized_indices, vec![1]);
        assert_eq!(proposal.status, PrivateProposalStatus::Active);
    }

    #[test]
    fn stale_proposal_index_returns_stable_error_code() {
        let create_key = h(42);
        let state = PrivateMultisigState::new(create_key, 2, 3, h(7));
        let state_account = account_with_data(1, to_vec(&state).unwrap());
        let proposal_account = empty_account(2);

        let err = handle(
            &[state_account, proposal_account],
            create_key,
            2,
            [1, 2, 3, 4, 5, 6, 7, 8],
            vec![],
            0,
            vec![],
            vec![],
        )
        .unwrap_err();

        assert_eq!(err, ProgramError::ProposalIndexMismatch);
        assert_eq!(err.code(), 2006);
    }
}
