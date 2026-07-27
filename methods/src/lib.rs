include!(concat!(env!("OUT_DIR"), "/methods.rs"));

#[cfg(test)]
mod tests {
    use borsh::from_slice;
    use lee_core::{
        account::{Account, AccountId, AccountWithMetadata},
        program::{Claim, ProgramOutput},
    };
    use private_multisig_core::{
        approve, build_config, member_leaf, Hash32, MemberSecret, PrivateMultisigInstruction,
        PrivateMultisigState, PrivateProposalState, PrivateProposalStatus,
    };
    use private_multisig_program::{multisig_state_pda_seed, proposal_pda_seed};
    use risc0_zkvm::{default_executor, ExecutorEnv};

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

    fn account_with_metadata(
        account: Account,
        is_authorized: bool,
        id_byte: u8,
    ) -> AccountWithMetadata {
        AccountWithMetadata {
            account,
            is_authorized,
            account_id: AccountId::new(h(id_byte)),
        }
    }

    fn execute_private_multisig(
        pre_states: Vec<AccountWithMetadata>,
        instruction: PrivateMultisigInstruction,
    ) -> ProgramOutput {
        let program_id = super::PRIVATE_MULTISIG_ID;
        let caller_program_id: Option<[u32; 8]> = None;
        let instruction_words =
            risc0_zkvm::serde::to_vec(&instruction).expect("serialize test instruction");
        let env = ExecutorEnv::builder()
            .write(&program_id)
            .unwrap()
            .write(&caller_program_id)
            .unwrap()
            .write(&pre_states)
            .unwrap()
            .write(&instruction_words)
            .unwrap()
            .build()
            .unwrap();
        let session = default_executor()
            .execute(env, super::PRIVATE_MULTISIG_ELF)
            .expect("execute private multisig guest");
        session.journal.decode().expect("decode program output")
    }

    #[test]
    fn private_multisig_guest_executes_create_multisig() {
        let create_key = h(42);
        let pre_states = vec![AccountWithMetadata {
            account: Account::default(),
            is_authorized: true,
            account_id: AccountId::new(h(1)),
        }];
        let instruction = PrivateMultisigInstruction::CreateMultisig {
            create_key,
            threshold: 2,
            member_count: 3,
            member_root: h(7),
        };

        let output = execute_private_multisig(pre_states, instruction);

        assert_eq!(output.self_program_id, super::PRIVATE_MULTISIG_ID);
        assert_eq!(output.post_states.len(), 1);
        assert_eq!(
            output.post_states[0].required_claim(),
            Some(Claim::Pda(lee_core::program::PdaSeed::new(
                multisig_state_pda_seed(create_key)
            )))
        );
        let state: PrivateMultisigState =
            from_slice(output.post_states[0].account().data.as_ref()).unwrap();
        assert_eq!(state.create_key, create_key);
        assert_eq!(state.threshold, 2);
        assert_eq!(state.member_count, 3);
        assert_eq!(state.member_root, h(7));
    }

    #[test]
    fn private_multisig_guest_runs_full_propose_and_execute_flow() {
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

        let create_output = execute_private_multisig(
            vec![account_with_metadata(Account::default(), true, 1)],
            PrivateMultisigInstruction::CreateMultisig {
                create_key,
                threshold: config.threshold,
                member_count: config.member_count,
                member_root: config.member_root,
            },
        );
        let state_account = create_output.post_states[0].account().clone();

        let target_program_id = [10, 20, 30, 40, 50, 60, 70, 80];
        let target_instruction_data = vec![700, 800];
        let propose_output = execute_private_multisig(
            vec![
                account_with_metadata(state_account.clone(), false, 1),
                account_with_metadata(Account::default(), true, 2),
            ],
            PrivateMultisigInstruction::Propose {
                create_key,
                proposal_index: 1,
                target_program_id,
                target_instruction_data: target_instruction_data.clone(),
                target_account_count: 2,
                pda_seeds: vec![h(9)],
                authorized_indices: vec![1],
            },
        );

        assert_eq!(propose_output.post_states.len(), 2);
        assert_eq!(
            propose_output.post_states[1].required_claim(),
            Some(Claim::Pda(lee_core::program::PdaSeed::new(
                proposal_pda_seed(&create_key, 1)
            )))
        );
        let proposal_state: PrivateProposalState =
            from_slice(propose_output.post_states[1].account().data.as_ref()).unwrap();
        assert_eq!(proposal_state.status, PrivateProposalStatus::Active);
        let proposal_view = proposal_state.as_proposal();
        let approvals = vec![
            approve(&members[0], &proposal_view).unwrap(),
            approve(&members[2], &proposal_view).unwrap(),
        ];
        let aggregate =
            private_multisig_core::aggregate(&config, &proposal_view, &leaves, &approvals).unwrap();

        let execute_output = execute_private_multisig(
            vec![
                account_with_metadata(propose_output.post_states[0].account().clone(), false, 1),
                account_with_metadata(propose_output.post_states[1].account().clone(), false, 2),
                account_with_metadata(Account::default(), false, 3),
                account_with_metadata(Account::default(), false, 4),
            ],
            PrivateMultisigInstruction::ExecutePrivate {
                create_key,
                proposal_index: 1,
                aggregate: aggregate.clone(),
            },
        );

        assert_eq!(execute_output.post_states.len(), 4);
        let proposal_post: PrivateProposalState =
            from_slice(execute_output.post_states[1].account().data.as_ref()).unwrap();
        assert_eq!(proposal_post.status, PrivateProposalStatus::Executed);
        assert_eq!(
            proposal_post.executed_aggregate_hash,
            Some(aggregate.aggregate_hash)
        );
        assert_eq!(execute_output.chained_calls.len(), 1);
        assert_eq!(
            execute_output.chained_calls[0].program_id,
            target_program_id
        );
        assert_eq!(
            execute_output.chained_calls[0].instruction_data,
            target_instruction_data
        );
        assert_eq!(execute_output.chained_calls[0].pre_states.len(), 2);
        assert!(!execute_output.chained_calls[0].pre_states[0].is_authorized);
        assert!(execute_output.chained_calls[0].pre_states[1].is_authorized);
    }
}
