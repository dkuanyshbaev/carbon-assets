use crate::mock::*;
use frame_support::{assert_ok, assert_noop, dispatch::{
    DispatchResult, 
}};
use crate::accounts::*;
use crate::Error;
type RuntimeError = Error<TestRuntime>;

// use crate::tests::mock::*;
use crate::{
    AUDITOR_ROLE_MASK, ISSUER_ROLE_MASK, MASTER_ROLE_MASK,
};
// use super::helpers::*;


#[test]
fn it_returns_true_for_correct_role_checks() {
    new_test_ext().execute_with(|| {
        assert_eq!(EvercityAccounts::account_is_master(&1), true);
        assert_eq!(EvercityAccounts::account_is_custodian(&2), true);
        assert_eq!(EvercityAccounts::account_is_issuer(&3), true);
        assert_eq!(EvercityAccounts::account_is_investor(&4), true);
        assert_eq!(EvercityAccounts::account_is_auditor(&5), true);
        assert_eq!(EvercityAccounts::account_is_manager(&6), true);
        assert_eq!(EvercityAccounts::account_is_bond_arranger(&7), true);
        assert_eq!(EvercityAccounts::account_is_impact_reporter(&8), true);
        assert_eq!(EvercityAccounts::account_is_cc_project_owner(&9), true);
        assert_eq!(EvercityAccounts::account_is_cc_auditor(&10), true);
        assert_eq!(EvercityAccounts::account_is_cc_standard(&11), true);
        assert_eq!(EvercityAccounts::account_is_cc_investor(&12), true);
        assert_eq!(EvercityAccounts::account_is_cc_registry(&13), true);

        assert_eq!(EvercityAccounts::account_is_master(&100), false);
        assert_eq!(EvercityAccounts::account_is_custodian(&100), false);
        assert_eq!(EvercityAccounts::account_is_issuer(&100), false);
        assert_eq!(EvercityAccounts::account_is_investor(&100), false);
        assert_eq!(EvercityAccounts::account_is_auditor(&100), false);
        assert_eq!(EvercityAccounts::account_token_mint_burn_allowed(&100), false);
    });
}

#[test]
fn it_returns_false_for_incorrect_role_checks() {
    new_test_ext().execute_with(|| {
        // Dispatch a signed extrinsic.
        //assert_ok!(AccountRegistry::insert(Origin::signed(1), EvercityAccountStruct {roles: 1u8, identity: 67u64}));
        // Read pallet storage and assert an expected result.
        assert_eq!(EvercityAccounts::account_is_auditor(&1), false);
        assert_eq!(EvercityAccounts::account_is_issuer(&2), false);
        assert_eq!(EvercityAccounts::account_is_investor(&3), false);
        assert_eq!(EvercityAccounts::account_is_custodian(&4), false);
        assert_eq!(EvercityAccounts::account_is_master(&5), false);
    });
}

#[test]
fn it_adds_new_account_with_correct_roles() {
    new_test_ext().execute_with(|| {
        Timestamp::set_timestamp(12345);

        assert_ok!(EvercityAccounts::account_add_with_role_and_data(
            Origin::signed(1),
            101,
            CUSTODIAN_ROLE_MASK,
            88u64
        ));
        assert_eq!(EvercityAccounts::account_is_custodian(&101), true);
        assert_eq!(EvercityAccounts::account_is_investor(&101), false);

        assert_ok!(EvercityAccounts::account_add_with_role_and_data(
            Origin::signed(1),
            102,
            AUDITOR_ROLE_MASK,
            89u64
        ));
        assert_eq!(EvercityAccounts::account_is_custodian(&102), false);
        assert_eq!(EvercityAccounts::account_is_auditor(&102), true);
    });
}

#[test]
fn it_correctly_sets_new_role_to_existing_account() {
    new_test_ext().execute_with(|| {
        // add new role to existing account (allowed only for master)
        assert_eq!(EvercityAccounts::account_is_issuer(&3), true);
        assert_ok!(EvercityAccounts::account_set_with_role_and_data(
            Origin::signed(1),
            3,
            AUDITOR_ROLE_MASK
        ));
        assert_eq!(EvercityAccounts::account_is_issuer(&3), true);
        assert_eq!(EvercityAccounts::account_is_auditor(&3), true);
        assert_eq!(EvercityAccounts::account_is_investor(&3), false);

        assert_eq!(EvercityAccounts::account_is_custodian(&2), true);
        assert_eq!(EvercityAccounts::account_is_issuer(&2), false);
        assert_ok!(EvercityAccounts::account_set_with_role_and_data(
            Origin::signed(1),
            2,
            ISSUER_ROLE_MASK
        ));
        assert_eq!(EvercityAccounts::account_is_custodian(&2), true);
        assert_eq!(EvercityAccounts::account_is_issuer(&2), true);
    });
}

#[test]
fn it_disable_account() {
    new_test_ext().execute_with(|| {
        assert_ok!(EvercityAccounts::account_add_with_role_and_data(
            Origin::signed(1),
            101,
            ISSUER_ROLE_MASK,
            88u64
        ));
        assert_eq!(EvercityAccounts::account_is_issuer(&101), true);
        assert_ok!(EvercityAccounts::account_disable(Origin::signed(1), 101));

        assert_eq!(EvercityAccounts::account_is_issuer(&101), false);
    });
}

#[test]
fn it_try_disable_yourself() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            EvercityAccounts::account_disable(Origin::signed(1), 1),
            RuntimeError::InvalidAction
        );
        assert_noop!(
            EvercityAccounts::account_set_with_role_and_data(Origin::signed(1), 1, 0),
            RuntimeError::InvalidAction
        );
    });
}

#[test]
fn it_denies_add_and_set_roles_for_non_master() {
    new_test_ext().execute_with(|| {
        // trying to add account form non-master account
        <pallet_timestamp::Module<TestRuntime>>::set_timestamp(12345);
        assert_noop!(
            EvercityAccounts::account_add_with_role_and_data(
                Origin::signed(2),
                101,
                MASTER_ROLE_MASK,
                88u64
            ),
            RuntimeError::AccountNotAuthorized
        );

        assert_noop!(
            EvercityAccounts::account_set_with_role_and_data(Origin::signed(2), 3, ISSUER_ROLE_MASK),
            RuntimeError::AccountNotAuthorized
        );
    });
}

#[test]
fn it_works_account_add_with_role_and_data() {
    new_test_ext().execute_with(|| {
        let some_new_account = 666;
        let assign_role_result = EvercityAccounts::account_add_with_role_and_data(
            Origin::signed(ROLES[0].0), some_new_account, CC_INVESTOR_ROLE_MASK, 0);
        assert_ok!(assign_role_result, ());
    });
}

#[test]
fn it_fails_account_add_with_role_and_data_not_master() {
    new_test_ext().execute_with(|| {
        let some_new_account = 666;
        let assign_role_result = EvercityAccounts::account_add_with_role_and_data(
            Origin::signed(ROLES[1].0), some_new_account, CC_INVESTOR_ROLE_MASK, 0);
        assert_ne!(assign_role_result, DispatchResult::Ok(()));
    });
}

#[test]
fn it_fails_account_set_with_role_and_data_not_exits() {
    new_test_ext().execute_with(|| {
        let some_new_account = 666;
        let assign_role_result = EvercityAccounts::account_set_with_role_and_data(
            Origin::signed(ROLES[0].0), some_new_account, CC_INVESTOR_ROLE_MASK);
        assert_ne!(assign_role_result, DispatchResult::Ok(()));
    });
}

#[test]
fn it_works_account_set_with_role_and_data() {
    new_test_ext().execute_with(|| {
        let some_new_account = 666;
        let _ = EvercityAccounts::account_add_with_role_and_data(
            Origin::signed(ROLES[0].0), some_new_account, CC_INVESTOR_ROLE_MASK, 0);
        let assign_role_result = EvercityAccounts::account_set_with_role_and_data(
            Origin::signed(ROLES[0].0), some_new_account, CC_AUDITOR_ROLE_MASK);
        assert!(EvercityAccounts::account_is_cc_investor(&some_new_account));
        assert_ok!(assign_role_result, ());
    });
}

#[test]
fn it_fails_account_set_with_role_and_data_not_master() {
    new_test_ext().execute_with(|| {
        let some_new_account = 666;
        let _ = EvercityAccounts::account_add_with_role_and_data(
            Origin::signed(ROLES[0].0), some_new_account, CC_INVESTOR_ROLE_MASK, 0);
        let assign_role_result = EvercityAccounts::account_set_with_role_and_data(
            Origin::signed(ROLES[1].0), some_new_account, CC_AUDITOR_ROLE_MASK);
        assert_ne!(assign_role_result, DispatchResult::Ok(()));
    });
}

#[test]
fn it_fails_account_set_with_master_role() {
    new_test_ext().execute_with(|| {
        let some_new_account = 666;
        let _ = EvercityAccounts::account_add_with_role_and_data(
            Origin::signed(ROLES[0].0), some_new_account, CC_INVESTOR_ROLE_MASK, 0);
        let assign_role_result = EvercityAccounts::account_set_with_role_and_data(
            Origin::signed(ROLES[0].0), some_new_account, MASTER_ROLE_MASK);
        assert_ne!(assign_role_result, DispatchResult::Ok(()));
    });
}

#[test]
fn it_works_roles_assigned_correctly_set_master() {
    new_test_ext().execute_with(|| {
        let some_new_account = 666;
        let _ = EvercityAccounts::account_add_with_role_and_data(
            Origin::signed(ROLES[0].0), some_new_account, CUSTODIAN_ROLE_MASK, 0);
        let all_roles = vec![
                CUSTODIAN_ROLE_MASK, 
                ISSUER_ROLE_MASK, 
                INVESTOR_ROLE_MASK, 
                AUDITOR_ROLE_MASK, 
                MANAGER_ROLE_MASK, 
                IMPACT_REPORTER_ROLE_MASK, 
                BOND_ARRANGER_ROLE_MASK,
                CC_PROJECT_OWNER_ROLE_MASK, 
                CC_AUDITOR_ROLE_MASK, 
                CC_STANDARD_ROLE_MASK, 
                CC_INVESTOR_ROLE_MASK, 
                CC_REGISTRY_ROLE_MASK
        ];

        all_roles.iter().for_each(|x| {
            let assign_role_result = EvercityAccounts::account_set_with_role_and_data(
                Origin::signed(ROLES[0].0), some_new_account, *x);
            assert_ok!(assign_role_result,());
        });

        assert!(EvercityAccounts::account_is_cc_project_owner(&some_new_account));
        assert!(EvercityAccounts::account_is_cc_auditor(&some_new_account));
        assert!(EvercityAccounts::account_is_cc_standard(&some_new_account));
        assert!(EvercityAccounts::account_is_cc_investor(&some_new_account));
        assert!(EvercityAccounts::account_is_cc_registry(&some_new_account));
    });
}

#[test]
fn it_works_account_set_with_master_role() {
    new_test_ext().execute_with(|| {
        let some_new_master_account = 666;
        let some_new_account = 1349;
        let set_master_result = EvercityAccounts::add_master_role(Origin::signed(ROLES[0].0), some_new_master_account);
        let assign_role_result = EvercityAccounts::account_add_with_role_and_data(Origin::signed(some_new_master_account), some_new_account, CC_PROJECT_OWNER_ROLE_MASK, 0);

        assert_ok!(set_master_result, ());
        assert_ok!(assign_role_result, ());
        assert!(EvercityAccounts::account_is_master(&some_new_master_account));
        assert!(EvercityAccounts::account_is_cc_project_owner(&some_new_account));
    });
}

#[test]
fn it_fails_account_set_with_master_role_already_master() {
    new_test_ext().execute_with(|| {
        let some_new_master_account = 666;
        let _ = EvercityAccounts::add_master_role(Origin::signed(ROLES[0].0), some_new_master_account);
        let set_master_result = EvercityAccounts::add_master_role(Origin::signed(ROLES[0].0), some_new_master_account);

        assert_ne!(set_master_result, DispatchResult::Ok(()));
    });
}

#[test]
fn it_works_account_withraw_role() {
    new_test_ext().execute_with(|| {
        let some_new_account = 666;
        let _ = EvercityAccounts::account_add_with_role_and_data(
            Origin::signed(ROLES[0].0), some_new_account, CC_INVESTOR_ROLE_MASK, 0);
        let assign_role_result = EvercityAccounts::account_set_with_role_and_data(
            Origin::signed(ROLES[0].0), some_new_account, CC_AUDITOR_ROLE_MASK);

        let withdraw_role_result = EvercityAccounts::account_withdraw_role(
            Origin::signed(ROLES[0].0), some_new_account, CC_INVESTOR_ROLE_MASK);

        assert_ok!(assign_role_result, ());
        assert_ok!(withdraw_role_result, ());
        assert!(!EvercityAccounts::account_is_cc_investor(&some_new_account));
    });
}

#[test]
fn it_works_check_events() {
    new_test_ext_with_event().execute_with(|| {
        let some_new_account = 666;
        let _ = EvercityAccounts::account_add_with_role_and_data(
            Origin::signed(ROLES[0].0), some_new_account, CC_INVESTOR_ROLE_MASK, 0);
        let add_account_event = last_event().unwrap();

        let _ = EvercityAccounts::account_set_with_role_and_data(
            Origin::signed(ROLES[0].0), some_new_account, CC_AUDITOR_ROLE_MASK);
        let set_account_event = last_event().unwrap();

        // let _ = EvercityAccounts::set_master(Origin::signed(ROLES[0].0), some_new_account);
        // let set_master_event = last_event().unwrap();

        let _ = EvercityAccounts::account_withdraw_role(Origin::signed(ROLES[0].0), some_new_account, CC_AUDITOR_ROLE_MASK);
        let withdraw_account_event = last_event().unwrap();

        assert_eq!(Event::pallet_evercity_accounts(crate::RawEvent::AccountAdd(ROLES[0].0, some_new_account, CC_INVESTOR_ROLE_MASK, 0)),
             add_account_event);
        assert_eq!(Event::pallet_evercity_accounts(crate::RawEvent::AccountSet(ROLES[0].0, some_new_account, CC_AUDITOR_ROLE_MASK)),
             set_account_event);
        // assert_eq!(Event::pallet_evercity_accounts(crate::RawEvent::MasterSet(ROLES[0].0, some_new_account)), set_master_event);
        assert_eq!(Event::pallet_evercity_accounts(crate::RawEvent::AccountWithdraw(ROLES[0].0, some_new_account, CC_AUDITOR_ROLE_MASK)),
             withdraw_account_event);
    });
}

#[test]
fn fuse_is_blone() {
    new_test_ext().execute_with(|| {
        let fuse = EvercityAccounts::fuse();
        assert_eq!(fuse, true);

        assert_noop!(
            EvercityAccounts::set_master(Origin::signed(2),),
            RuntimeError::InvalidAction
        );
    })
}

#[test]
fn fuse_is_intact_on_bare_storage() {
    let mut ext: sp_io::TestExternalities = frame_system::GenesisConfig::default()
        .build_storage::<TestRuntime>()
        .unwrap()
        .into();

    ext.execute_with(|| {
        assert_eq!(EvercityAccounts::fuse(), false);

        assert_noop!(
            EvercityAccounts::account_add_with_role_and_data(Origin::signed(1), 101, MASTER_ROLE_MASK, 0),
            RuntimeError::AccountNotAuthorized
        );
        assert_ok!(EvercityAccounts::set_master(Origin::signed(1),));
        // make amend
        // assert_ok!(EvercityAccounts::account_add_with_role_and_data(
        //     Origin::signed(1),
        //     101,
        //     MASTER_ROLE_MASK,
        //     0
        // ));

        assert_eq!(EvercityAccounts::fuse(), true);
        assert_noop!(
            EvercityAccounts::set_master(Origin::signed(2),),
            RuntimeError::InvalidAction
        );
    });
}

#[test]
fn it_checks_is_roles_mask_included() {
    // true
    assert!(is_roles_mask_included(MASTER_ROLE_MASK, MASTER_ROLE_MASK));
    assert!(is_roles_mask_included(MASTER_ROLE_MASK | CUSTODIAN_ROLE_MASK, MASTER_ROLE_MASK));
    assert!(is_roles_mask_included(ALL_ROLES_MASK, MASTER_ROLE_MASK));
    assert!(is_roles_mask_included(MASTER_ROLE_MASK | CC_AUDITOR_ROLE_MASK, MASTER_ROLE_MASK));
    // false
    assert!(!is_roles_mask_included(AUDITOR_ROLE_MASK, MASTER_ROLE_MASK));
    assert!(!is_roles_mask_included(CUSTODIAN_ROLE_MASK | CC_AUDITOR_ROLE_MASK, MASTER_ROLE_MASK));
    assert!(!is_roles_mask_included(CC_INVESTOR_ROLE_MASK, MASTER_ROLE_MASK));
    assert!(!is_roles_mask_included(CC_PROJECT_OWNER_ROLE_MASK | CC_STANDARD_ROLE_MASK, MASTER_ROLE_MASK));
    assert!(!is_roles_mask_included(BOND_ARRANGER_ROLE_MASK, MASTER_ROLE_MASK));
}
