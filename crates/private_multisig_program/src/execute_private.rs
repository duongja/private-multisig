use borsh::{from_slice, to_vec};
use lee_core::account::{Account, AccountWithMetadata};
use lee_core::program::{ChainedCall, PdaSeed};
use private_multisig_core::{
    verify_aggregate, AggregateApproval, PrivateMultisigState, PrivateProposalState,
    PrivateProposalStatus,
};

use crate::{ProgramError, ProgramResult};

pub fn handle(
    accounts: &[AccountWithMetadata],
    create_key: [u8; 32],
    proposal_index: u64,
    aggregate: AggregateApproval,
) -> ProgramResult<(Vec<AccountWithMetadata>, Vec<ChainedCall>)> {
    if accounts.len() < 2 {
        return Err(ProgramError::InvalidAccountCount);
    }

    let state_account = &accounts[0];
    let proposal_account = &accounts[1];
    let target_accounts = &accounts[2..];

    let state: PrivateMultisigState = from_slice(&Vec::from(state_account.account.data.clone()))
        .map_err(|_| ProgramError::DecodeState)?;
    if state.create_key != create_key {
        return Err(ProgramError::CreateKeyMismatch);
    }

    let mut proposal: PrivateProposalState =
        from_slice(&Vec::from(proposal_account.account.data.clone()))
            .map_err(|_| ProgramError::DecodeProposal)?;
    if proposal.multisig_create_key != create_key {
        return Err(ProgramError::CreateKeyMismatch);
    }
    if proposal.index != proposal_index {
        return Err(ProgramError::ProposalIndexMismatch);
    }
    if proposal.status != PrivateProposalStatus::Active {
        return Err(ProgramError::ProposalNotActive);
    }
    if target_accounts.len() != proposal.target_account_count as usize {
        return Err(ProgramError::TargetAccountCountMismatch);
    }

    verify_aggregate(&state.config(), &proposal.as_proposal(), &aggregate)
        .map_err(|_| ProgramError::InvalidAggregateProof)?;

    proposal.status = PrivateProposalStatus::Executed;
    proposal.executed_aggregate_hash = Some(aggregate.aggregate_hash);
    proposal.approval_count = aggregate.approval_count;

    let mut proposal_post = proposal_account.account.clone();
    proposal_post.data = to_vec(&proposal)
        .expect("proposal serialization")
        .try_into()
        .map_err(|_| ProgramError::AccountDataTooLarge)?;

    let target_pre_states = target_accounts
        .iter()
        .enumerate()
        .map(|(idx, account)| {
            let mut out = account.clone();
            if proposal.authorized_indices.contains(&(idx as u8)) {
                out.is_authorized = true;
            }
            out
        })
        .collect();

    let chained_call = ChainedCall {
        program_id: proposal.target_program_id,
        pre_states: target_pre_states,
        instruction_data: proposal.target_instruction_data.clone(),
        pda_seeds: proposal
            .pda_seeds
            .iter()
            .copied()
            .map(PdaSeed::new)
            .collect(),
    };

    let wrap = |account: Account, original: &AccountWithMetadata| AccountWithMetadata {
        account,
        account_id: original.account_id,
        is_authorized: false,
    };

    let mut accounts_out = vec![
        wrap(state_account.account.clone(), state_account),
        wrap(proposal_post, proposal_account),
    ];
    accounts_out.extend(target_accounts.iter().cloned());

    Ok((accounts_out, vec![chained_call]))
}

#[cfg(test)]
mod tests {
    use super::*;
    use borsh::to_vec;
    use lee_core::account::AccountId;
    use private_multisig_core::{approve, build_config, member_leaf, Hash32, MemberSecret};

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
    fn executes_private_threshold_approval_as_chained_call() {
        let create_key = h(42);
        let members = vec![
            member(create_key, 1),
            member(create_key, 2),
            member(create_key, 3),
        ];
        let leaves: Vec<Hash32> = members
            .iter()
            .map(|m| member_leaf(&m.multisig_id, &m.npk, &m.membership_secret))
            .collect();
        let config = build_config(create_key, 2, &leaves).unwrap();
        let state = PrivateMultisigState::new(
            config.multisig_id,
            config.threshold,
            config.member_count,
            config.member_root,
        );
        let proposal = PrivateProposalState::new(
            1,
            create_key,
            [10, 20, 30, 40, 50, 60, 70, 80],
            vec![700, 800],
            2,
            vec![h(9)],
            vec![1],
        );
        let proposal_view = proposal.as_proposal();
        let approvals = vec![
            approve(&members[0], &proposal_view).unwrap(),
            approve(&members[2], &proposal_view).unwrap(),
        ];
        let aggregate =
            private_multisig_core::aggregate(&config, &proposal_view, &leaves, &approvals).unwrap();

        let accounts = vec![
            account_with_data(1, to_vec(&state).unwrap()),
            account_with_data(2, to_vec(&proposal).unwrap()),
            empty_account(3),
            empty_account(4),
        ];

        let (accounts_out, calls) = handle(&accounts, create_key, 1, aggregate.clone()).unwrap();

        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0].program_id, proposal.target_program_id);
        assert_eq!(calls[0].instruction_data, proposal.target_instruction_data);
        assert_eq!(calls[0].pda_seeds.len(), 1);
        assert_eq!(calls[0].pda_seeds[0].as_bytes(), &h(9));
        assert_eq!(calls[0].pre_states.len(), 2);
        assert!(!calls[0].pre_states[0].is_authorized);
        assert!(calls[0].pre_states[1].is_authorized);

        let proposal_post: PrivateProposalState =
            borsh::from_slice(&Vec::from(accounts_out[1].account.data.clone())).unwrap();
        assert_eq!(proposal_post.status, PrivateProposalStatus::Executed);
        assert_eq!(
            proposal_post.executed_aggregate_hash,
            Some(aggregate.aggregate_hash)
        );
        assert_eq!(proposal_post.approval_count, 2);
    }

    #[test]
    fn invalid_aggregate_returns_stable_error_code() {
        let create_key = h(42);
        let members = vec![
            member(create_key, 1),
            member(create_key, 2),
            member(create_key, 3),
        ];
        let leaves: Vec<Hash32> = members
            .iter()
            .map(|m| member_leaf(&m.multisig_id, &m.npk, &m.membership_secret))
            .collect();
        let config = build_config(create_key, 2, &leaves).unwrap();
        let state = PrivateMultisigState::new(
            config.multisig_id,
            config.threshold,
            config.member_count,
            config.member_root,
        );
        let proposal = PrivateProposalState::new(
            1,
            create_key,
            [10, 20, 30, 40, 50, 60, 70, 80],
            vec![700, 800],
            0,
            vec![],
            vec![],
        );
        let proposal_view = proposal.as_proposal();
        let approvals = vec![
            approve(&members[0], &proposal_view).unwrap(),
            approve(&members[2], &proposal_view).unwrap(),
        ];
        let mut aggregate =
            private_multisig_core::aggregate(&config, &proposal_view, &leaves, &approvals).unwrap();
        aggregate.nullifiers.push(aggregate.nullifiers[0]);

        let accounts = vec![
            account_with_data(1, to_vec(&state).unwrap()),
            account_with_data(2, to_vec(&proposal).unwrap()),
        ];

        let err = handle(&accounts, create_key, 1, aggregate).unwrap_err();

        assert_eq!(err, ProgramError::InvalidAggregateProof);
        assert_eq!(err.code(), 2009);
    }
}
