use lee_core::program::{
    read_lee_inputs, AccountPostState, Claim, PdaSeed, ProgramInput, ProgramOutput,
};
use private_multisig_core::PrivateMultisigInstruction;
use private_multisig_program::{
    create_multisig, execute_private, multisig_state_pda_seed, proposal_pda_seed, propose,
    ProgramError,
};

fn main() {
    let (
        ProgramInput {
            self_program_id,
            caller_program_id,
            pre_states,
            instruction,
        },
        instruction_words,
    ) = read_lee_inputs::<PrivateMultisigInstruction>();

    let pre_states_clone = pre_states.clone();
    let (post_states, chained_calls) = dispatch(pre_states, instruction)
        .unwrap_or_else(|err| panic!("private_multisig_error:{}:{err}", err.code()));

    ProgramOutput::new(
        self_program_id,
        caller_program_id,
        instruction_words,
        pre_states_clone,
        post_states,
    )
    .with_chained_calls(chained_calls)
    .write();
}

fn dispatch(
    pre_states: Vec<lee_core::account::AccountWithMetadata>,
    instruction: PrivateMultisigInstruction,
) -> private_multisig_program::ProgramResult<(
    Vec<AccountPostState>,
    Vec<lee_core::program::ChainedCall>,
)> {
    match instruction {
        PrivateMultisigInstruction::CreateMultisig {
            create_key,
            threshold,
            member_count,
            member_root,
        } => {
            let (accounts, chained_calls) = create_multisig::handle(
                &pre_states,
                create_key,
                threshold,
                member_count,
                member_root,
            )?;
            let [state_account] =
                <[_; 1]>::try_from(accounts).map_err(|_| ProgramError::InvalidAccountCount)?;
            Ok((
                vec![AccountPostState::new_claimed(
                    state_account,
                    Claim::Pda(PdaSeed::new(multisig_state_pda_seed(create_key))),
                )],
                chained_calls,
            ))
        }
        PrivateMultisigInstruction::Propose {
            create_key,
            proposal_index,
            target_program_id,
            target_instruction_data,
            target_account_count,
            pda_seeds,
            authorized_indices,
        } => {
            let (accounts, chained_calls) = propose::handle(
                &pre_states,
                create_key,
                proposal_index,
                target_program_id,
                target_instruction_data,
                target_account_count,
                pda_seeds,
                authorized_indices,
            )?;
            let [state_account, proposal_account] =
                <[_; 2]>::try_from(accounts).map_err(|_| ProgramError::InvalidAccountCount)?;
            Ok((
                vec![
                    AccountPostState::new(state_account),
                    AccountPostState::new_claimed(
                        proposal_account,
                        Claim::Pda(PdaSeed::new(proposal_pda_seed(&create_key, proposal_index))),
                    ),
                ],
                chained_calls,
            ))
        }
        PrivateMultisigInstruction::ExecutePrivate {
            create_key,
            proposal_index,
            aggregate,
        } => {
            let (accounts, chained_calls) =
                execute_private::handle(&pre_states, create_key, proposal_index, aggregate)?;
            Ok((
                accounts
                    .into_iter()
                    .map(|account| AccountPostState::new(account.account))
                    .collect(),
                chained_calls,
            ))
        }
    }
}
