use rstest::rstest;
use starknet_api::core::{ClassHash, ContractAddress, EthAddress, PatriciaKey};
use starknet_api::state::StorageKey;
use starknet_api::transaction::L2ToL1Payload;
use starknet_api::{class_hash, felt, patricia_key};
use starknet_types_core::felt::Felt;

use crate::execution::call_info::{
    CallExecution,
    CallInfo,
    EventSummary,
    ExecutionSummary,
    MessageToL1,
    OrderedEvent,
    OrderedL2ToL1Message,
};
use crate::execution::entry_point::CallEntryPoint;
use crate::transaction::objects::TransactionExecutionInfo;

#[derive(Debug, Default)]
pub struct TestExecutionSummary {
    pub num_of_events: usize,
    pub num_of_messages: usize,
    pub class_hash: ClassHash,
    pub storage_address: ContractAddress,
    pub storage_key: StorageKey,
}

impl TestExecutionSummary {
    pub fn new(
        num_of_events: usize,
        num_of_messages: usize,
        class_hash: ClassHash,
        storage_address: &str,
        storage_key: &str,
    ) -> Self {
        TestExecutionSummary {
            num_of_events,
            num_of_messages,
            class_hash,
            storage_address: ContractAddress(patricia_key!(storage_address)),
            storage_key: StorageKey(patricia_key!(storage_key)),
        }
    }

    pub fn to_call_info(&self) -> CallInfo {
        CallInfo {
            call: CallEntryPoint {
                class_hash: Some(self.class_hash),
                storage_address: self.storage_address,
                ..Default::default()
            },
            execution: CallExecution {
                events: (0..self.num_of_events).map(|_| OrderedEvent::default()).collect(),
                l2_to_l1_messages: (0..self.num_of_messages)
                    .map(|i| OrderedL2ToL1Message {
                        order: i,
                        message: MessageToL1 {
                            to_address: EthAddress::default(),
                            payload: L2ToL1Payload(vec![Felt::default()]),
                        },
                    })
                    .collect(),
                ..Default::default()
            },
            accessed_storage_keys: vec![self.storage_key].into_iter().collect(),
            ..Default::default()
        }
    }
}

fn shared_call_info() -> CallInfo {
    CallInfo {
        call: CallEntryPoint { class_hash: Some(class_hash!("0x1")), ..Default::default() },
        ..Default::default()
    }
}

fn call_info_with_x_events(n_events: usize, n_inner_calls: usize) -> CallInfo {
    CallInfo {
        execution: CallExecution {
            events: (0..n_events).map(|_| OrderedEvent::default()).collect(),
            ..Default::default()
        },
        inner_calls: (0..n_inner_calls).map(|_| call_info_with_x_events(1, 0)).collect(),
        ..shared_call_info()
    }
}

fn call_info_with_deep_inner_calls(
    n_events: usize,
    n_inner_calls: usize,
    n_events_of_each_inner_call: usize,
    n_inner_calls_of_each_inner_call: usize,
) -> CallInfo {
    let inner_calls = (0..n_inner_calls)
        .map(|_| {
            call_info_with_x_events(n_events_of_each_inner_call, n_inner_calls_of_each_inner_call)
        })
        .collect();

    CallInfo {
        inner_calls,
        execution: CallExecution {
            events: (0..n_events).map(|_| OrderedEvent::default()).collect(),
            ..Default::default()
        },
        ..shared_call_info()
    }
}

#[rstest]
#[case(0, 0)]
#[case(0, 2)]
#[case(1, 3)]
#[case(2, 0)]
fn test_events_counter_in_transaction_execution_info(
    #[case] n_execute_events: usize,
    #[case] n_inner_calls: usize,
) {
    let n_validate_events = 2;
    let n_fee_transfer_events = 1;

    let transaction_execution_info = TransactionExecutionInfo {
        validate_call_info: Some(call_info_with_x_events(n_validate_events, 0)),
        execute_call_info: Some(call_info_with_x_events(n_execute_events, n_inner_calls)),
        fee_transfer_call_info: Some(call_info_with_x_events(n_fee_transfer_events, 0)),
        ..Default::default()
    };

    assert_eq!(
        transaction_execution_info.summarize().event_summary.n_events,
        n_validate_events + n_execute_events + n_fee_transfer_events + n_inner_calls
    );
}

#[rstest]
#[case(0)]
#[case(1)]
#[case(20)]
fn test_events_counter_in_transaction_execution_info_with_inner_call_info(
    #[case] n_execute_events: usize,
) {
    let n_fee_transfer_events = 2;
    let n_inner_calls = 3;
    let n_execution_events = 1;
    let n_events_for_each_inner_call = 2;
    let n_inner_calls_of_each_inner_call = 1;

    let transaction_execution_info = TransactionExecutionInfo {
        validate_call_info: Some(call_info_with_deep_inner_calls(
            n_execution_events,
            n_inner_calls,
            n_events_for_each_inner_call,
            n_inner_calls_of_each_inner_call,
        )),
        execute_call_info: Some(call_info_with_x_events(n_execute_events, 0)),
        fee_transfer_call_info: Some(call_info_with_x_events(n_fee_transfer_events, 0)),
        ..Default::default()
    };

    assert_eq!(
        transaction_execution_info.summarize().event_summary.n_events,
        n_execute_events
            + n_fee_transfer_events
            + n_execution_events
            + n_inner_calls
            + n_events_for_each_inner_call * n_inner_calls
    );
}

#[rstest]
#[case(
    TestExecutionSummary::new(1, 2, class_hash!("0x1"), "0x1", "0x1"),
    TestExecutionSummary::new(2, 3, class_hash!("0x2"), "0x2", "0x2"),
    TestExecutionSummary::new(3, 4, class_hash!("0x3"), "0x3", "0x3")
)]
fn test_summarize(
    #[case] validate_params: TestExecutionSummary,
    #[case] execute_params: TestExecutionSummary,
    #[case] fee_transfer_params: TestExecutionSummary,
) {
    let validate_call_info = validate_params.to_call_info();
    let execute_call_info = execute_params.to_call_info();
    let fee_transfer_call_info = fee_transfer_params.to_call_info();

    let transaction_execution_info = TransactionExecutionInfo {
        validate_call_info: Some(validate_call_info),
        execute_call_info: Some(execute_call_info),
        fee_transfer_call_info: Some(fee_transfer_call_info),
        ..Default::default()
    };

    let expected_summary = ExecutionSummary {
        executed_class_hashes: vec![
            validate_params.class_hash,
            execute_params.class_hash,
            fee_transfer_params.class_hash,
        ]
        .into_iter()
        .collect(),
        visited_storage_entries: vec![
            (validate_params.storage_address, validate_params.storage_key),
            (execute_params.storage_address, execute_params.storage_key),
            (fee_transfer_params.storage_address, fee_transfer_params.storage_key),
        ]
        .into_iter()
        .collect(),
        l2_to_l1_payload_lengths: vec![
            1;
            validate_params.num_of_messages
                + execute_params.num_of_messages
                + fee_transfer_params.num_of_messages
        ],
        event_summary: EventSummary {
            n_events: validate_params.num_of_events
                + execute_params.num_of_events
                + fee_transfer_params.num_of_events,
            total_event_keys: 0,
            total_event_data_size: 0,
        },
    };

    // Call the summarize method
    let actual_summary = transaction_execution_info.summarize();

    // Compare the actual result with the expected result
    assert_eq!(actual_summary.executed_class_hashes, expected_summary.executed_class_hashes);
    assert_eq!(actual_summary.visited_storage_entries, expected_summary.visited_storage_entries);
    assert_eq!(actual_summary.event_summary.n_events, expected_summary.event_summary.n_events);
    assert_eq!(actual_summary.l2_to_l1_payload_lengths, expected_summary.l2_to_l1_payload_lengths);
}
