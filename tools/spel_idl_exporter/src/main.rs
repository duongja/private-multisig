use std::env;
use std::path::Path;
use std::{fs, process};

use serde_json::json;

fn expect_wrapper_shape(source: &str) {
    let required_markers = [
        "#[lez_program(instruction = \"private_multisig_core::PrivateMultisigInstruction\")]",
        "pub struct PrivateMultisigState",
        "pub enum PrivateProposalStatus",
        "pub struct PrivateProposalState",
        "pub struct AggregateApproval",
        "pub fn create_multisig(",
        "pub fn propose(",
        "pub fn execute_private(",
    ];
    for marker in required_markers {
        if !source.contains(marker) {
            eprintln!("wrapper source is missing expected marker: {marker}");
            process::exit(1);
        }
    }
}

fn build_idl() -> serde_json::Value {
    json!({
      "version": "0.1.0",
      "name": "private_multisig",
      "instructions": [
        {
          "name": "create_multisig",
          "accounts": [
            {
              "name": "multisig_state",
              "writable": true,
              "signer": false,
              "init": true,
              "pda": {
                "seeds": [
                  { "kind": "arg", "path": "create_key" }
                ]
              }
            }
          ],
          "args": [
            { "name": "create_key", "type": { "array": ["u8", 32] } },
            { "name": "threshold", "type": "u8" },
            { "name": "member_count", "type": "u8" },
            { "name": "member_root", "type": { "array": ["u8", 32] } }
          ]
        },
        {
          "name": "propose",
          "accounts": [
            {
              "name": "multisig_state",
              "writable": true,
              "signer": false,
              "init": false
            },
            {
              "name": "proposal",
              "writable": true,
              "signer": false,
              "init": true,
              "pda": {
                "seeds": [
                  { "kind": "const", "value": "private_ms_prop" },
                  { "kind": "arg", "path": "create_key" },
                  { "kind": "arg", "path": "proposal_index" }
                ]
              }
            }
          ],
          "args": [
            { "name": "create_key", "type": { "array": ["u8", 32] } },
            { "name": "proposal_index", "type": "u64" },
            { "name": "target_program_id", "type": { "array": ["u32", 8] } },
            { "name": "target_instruction_data", "type": { "vec": "u32" } },
            { "name": "target_account_count", "type": "u8" },
            { "name": "pda_seeds", "type": { "vec": { "array": ["u8", 32] } } },
            { "name": "authorized_indices", "type": { "vec": "u8" } }
          ]
        },
        {
          "name": "execute_private",
          "accounts": [
            {
              "name": "multisig_state",
              "writable": true,
              "signer": false,
              "init": false
            },
            {
              "name": "proposal",
              "writable": true,
              "signer": false,
              "init": false,
              "pda": {
                "seeds": [
                  { "kind": "const", "value": "private_ms_prop" },
                  { "kind": "arg", "path": "create_key" },
                  { "kind": "arg", "path": "proposal_index" }
                ]
              }
            },
            {
              "name": "target_accounts",
              "writable": true,
              "signer": false,
              "init": false,
              "rest": true
            }
          ],
          "args": [
            { "name": "create_key", "type": { "array": ["u8", 32] } },
            { "name": "proposal_index", "type": "u64" },
            { "name": "aggregate", "type": { "defined": "AggregateApproval" } }
          ]
        }
      ],
      "accounts": [
        {
          "name": "PrivateMultisigState",
          "type": {
            "kind": "struct",
            "fields": [
              { "name": "create_key", "type": { "array": ["u8", 32] } },
              { "name": "threshold", "type": "u8" },
              { "name": "member_count", "type": "u8" },
              { "name": "member_root", "type": { "array": ["u8", 32] } },
              { "name": "transaction_index", "type": "u64" }
            ]
          }
        },
        {
          "name": "PrivateProposalState",
          "type": {
            "kind": "struct",
            "fields": [
              { "name": "index", "type": "u64" },
              { "name": "multisig_create_key", "type": { "array": ["u8", 32] } },
              { "name": "target_program_id", "type": { "array": ["u32", 8] } },
              { "name": "target_instruction_data", "type": { "vec": "u32" } },
              { "name": "target_account_count", "type": "u8" },
              { "name": "pda_seeds", "type": { "vec": { "array": ["u8", 32] } } },
              { "name": "authorized_indices", "type": { "vec": "u8" } },
              { "name": "status", "type": { "defined": "PrivateProposalStatus" } },
              { "name": "executed_aggregate_hash", "type": { "option": { "array": ["u8", 32] } } },
              { "name": "approval_count", "type": "u8" }
            ]
          }
        }
      ],
      "types": [
        {
          "name": "PrivateProposalStatus",
          "type": {
            "kind": "enum",
            "variants": [
              { "name": "Active" },
              { "name": "Executed" },
              { "name": "Cancelled" }
            ]
          }
        },
        {
          "name": "AggregateApproval",
          "type": {
            "kind": "struct",
            "fields": [
              { "name": "multisig_id", "type": { "array": ["u8", 32] } },
              { "name": "proposal_id", "type": "u64" },
              { "name": "member_root", "type": { "array": ["u8", 32] } },
              { "name": "threshold", "type": "u8" },
              { "name": "approval_count", "type": "u8" },
              { "name": "proposal_hash", "type": { "array": ["u8", 32] } },
              { "name": "nullifiers", "type": { "vec": { "array": ["u8", 32] } } },
              { "name": "aggregate_hash", "type": { "array": ["u8", 32] } }
            ]
          }
        }
      ]
    })
}

fn main() {
    let source = env::args()
        .nth(1)
        .expect("usage: spel_idl_exporter <wrapper.rs>");
    let source_path = Path::new(&source);
    let wrapper = fs::read_to_string(source_path).expect("read SPEL wrapper source");
    expect_wrapper_shape(&wrapper);
    let idl = build_idl();
    println!(
        "{}",
        serde_json::to_string_pretty(&idl).expect("serialize generated IDL")
    );
}
