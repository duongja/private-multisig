use borsh::to_vec;
use lee_core::account::{Account, AccountWithMetadata};
use lee_core::program::ChainedCall;
use private_multisig_core::{Hash32, PrivateMultisigState};

use crate::{ProgramError, ProgramResult};

pub fn handle(
    accounts: &[AccountWithMetadata],
    create_key: Hash32,
    threshold: u8,
    member_count: u8,
    member_root: Hash32,
) -> ProgramResult<(Vec<Account>, Vec<ChainedCall>)> {
    if accounts.len() != 1 {
        return Err(ProgramError::InvalidAccountCount);
    }
    if accounts[0].account != Account::default() {
        return Err(ProgramError::AlreadyInitialized);
    }
    if member_count == 0 || threshold == 0 || threshold > member_count {
        return Err(ProgramError::InvalidThreshold);
    }

    let state = PrivateMultisigState::new(create_key, threshold, member_count, member_root);
    let mut account = Account::default();
    account.data = to_vec(&state)
        .expect("state serialization")
        .try_into()
        .map_err(|_| ProgramError::AccountDataTooLarge)?;

    Ok((vec![account], vec![]))
}

#[cfg(test)]
mod tests {
    use super::*;
    use lee_core::account::AccountId;

    fn empty_account() -> AccountWithMetadata {
        AccountWithMetadata {
            account_id: AccountId::new([9u8; 32]),
            account: Account::default(),
            is_authorized: false,
        }
    }

    #[test]
    fn creates_private_multisig_state() {
        let (accounts, calls) = handle(&[empty_account()], [1u8; 32], 2, 3, [7u8; 32]).unwrap();

        assert!(calls.is_empty());
        let state: PrivateMultisigState =
            borsh::from_slice(&Vec::from(accounts[0].data.clone())).unwrap();
        assert_eq!(state.threshold, 2);
        assert_eq!(state.member_count, 3);
        assert_eq!(state.member_root, [7u8; 32]);
    }

    #[test]
    fn invalid_threshold_returns_stable_error_code() {
        let err = handle(&[empty_account()], [1u8; 32], 4, 3, [7u8; 32]).unwrap_err();

        assert_eq!(err, ProgramError::InvalidThreshold);
        assert_eq!(err.code(), 2002);
    }
}
