use std::{fs, path::PathBuf, time::Duration};

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use common::{transaction::LeeTransaction, HashType};
use lee::program::Program;
use lee_core::{
    account::AccountId,
    program::{PdaSeed, ProgramId},
};
use private_multisig_core::{
    approve, build_config, hash_chunks, member_leaf, to_hex, Hash32, MemberSecret,
    PrivateMultisigInstruction, PrivateMultisigState, PrivateProposalState, PrivateProposalStatus,
};
use private_multisig_methods::{PRIVATE_MULTISIG_ELF, PRIVATE_MULTISIG_ID};
use private_multisig_program::{multisig_state_pda_seed, proposal_pda_seed};
use sequencer_service_rpc::{RpcClient as _, SequencerClientBuilder};
use serde::Serialize;

const RPC_HEALTH_TIMEOUT: Duration = Duration::from_secs(20);
const RPC_SEND_TIMEOUT: Duration = Duration::from_secs(45);
const RPC_GET_ACCOUNT_TIMEOUT: Duration = Duration::from_secs(20);
const RPC_GET_TX_TIMEOUT: Duration = Duration::from_secs(10);

#[derive(Parser, Debug)]
#[command(name = "private-multisig-runner")]
#[command(about = "LP-0002 localnet/testnet evidence runner")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
    LocalnetEvidence {
        #[arg(long, default_value = "http://127.0.0.1:3040")]
        sequencer: String,
        #[arg(long)]
        out_dir: PathBuf,
        #[arg(long)]
        target_program_binary: PathBuf,
        #[arg(long)]
        config: Option<PathBuf>,
        #[arg(long)]
        proposal: Option<PathBuf>,
        #[arg(long)]
        aggregate: Option<PathBuf>,
        #[arg(long, default_value_t = 30)]
        poll_seconds: u64,
    },
    WriteProposalTemplate {
        #[arg(long)]
        multisig_id: String,
        #[arg(long)]
        proposal_id: u64,
        #[arg(long)]
        target_program_binary: PathBuf,
        #[arg(long)]
        target_program_id: Option<String>,
        #[arg(long)]
        instruction_words: Option<String>,
        #[arg(long, default_value_t = 1)]
        target_account_count: u8,
        #[arg(long)]
        out: PathBuf,
    },
    WriteProgram {
        #[arg(long)]
        out: PathBuf,
    },
}

#[derive(Debug, Serialize)]
struct Evidence {
    ok: bool,
    sequencer: String,
    program_id: ProgramId,
    program_id_hex: String,
    program_binary: String,
    target_program_id: ProgramId,
    target_program_id_hex: String,
    target_program_binary: String,
    multisig_state_account: String,
    proposal_account: String,
    target_accounts: Vec<String>,
    member_root: String,
    aggregate_hash: String,
    txs: EvidenceTxs,
    final_state: FinalState,
}

#[derive(Debug, Serialize)]
struct EvidenceTxs {
    private_multisig_deploy: TxEvidence,
    target_deploy: TxEvidence,
    create_multisig: TxEvidence,
    propose: TxEvidence,
    execute_private: TxEvidence,
}

#[derive(Debug, Serialize)]
struct TxEvidence {
    hash: String,
    included: bool,
}

#[derive(Debug, Serialize)]
struct FinalState {
    multisig_transaction_index: u64,
    proposal_status: String,
    proposal_approval_count: u8,
    proposal_executed_aggregate_hash: Option<String>,
    target_account_program_id_hex: String,
    target_account_nonce: u128,
    target_account_data_utf8: String,
}

#[derive(Debug, Serialize, serde::Deserialize)]
struct ConfigFile {
    multisig_id: String,
    threshold: u8,
    member_count: u8,
    member_root: String,
}

#[derive(Debug, Serialize, serde::Deserialize)]
struct ProposalFile {
    multisig_id: String,
    proposal_id: u64,
    target_program_id: [u32; 8],
    target_instruction_data: Vec<u32>,
    target_account_count: u8,
    pda_seeds: Vec<String>,
    authorized_indices: Vec<u8>,
}

#[derive(Debug, Serialize, serde::Deserialize)]
struct AggregateFile {
    multisig_id: String,
    proposal_id: u64,
    member_root: String,
    threshold: u8,
    approval_count: u8,
    proposal_hash: String,
    nullifiers: Vec<String>,
    aggregate_hash: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    match Cli::parse().command {
        Command::LocalnetEvidence {
            sequencer,
            out_dir,
            target_program_binary,
            config,
            proposal,
            aggregate,
            poll_seconds,
        } => {
            run_localnet_evidence(
                sequencer,
                out_dir,
                target_program_binary,
                config,
                proposal,
                aggregate,
                poll_seconds,
            )
            .await
        }
        Command::WriteProposalTemplate {
            multisig_id,
            proposal_id,
            target_program_binary,
            target_program_id,
            instruction_words,
            target_account_count,
            out,
        } => write_proposal_template(
            multisig_id,
            proposal_id,
            target_program_binary,
            target_program_id,
            instruction_words,
            target_account_count,
            out,
        ),
        Command::WriteProgram { out } => {
            write_program_binary(&out)?;
            println!("{}", out.display());
            Ok(())
        }
    }
}

async fn run_localnet_evidence(
    sequencer: String,
    out_dir: PathBuf,
    target_program_binary: PathBuf,
    config_path: Option<PathBuf>,
    proposal_path: Option<PathBuf>,
    aggregate_path: Option<PathBuf>,
    poll_seconds: u64,
) -> Result<()> {
    fs::create_dir_all(&out_dir).with_context(|| format!("create {}", out_dir.display()))?;
    let program_path = out_dir.join("private_multisig.bin");
    write_program_binary(&program_path)?;
    eprintln!(
        "runner: wrote embedded private multisig ELF to {}",
        program_path.display()
    );
    let target_program_bytes = fs::read(&target_program_binary)
        .with_context(|| format!("read {}", target_program_binary.display()))?;
    eprintln!(
        "runner: loaded target program binary {} ({} bytes)",
        target_program_binary.display(),
        target_program_bytes.len()
    );

    let client = SequencerClientBuilder::default()
        .build(&sequencer)
        .context("build sequencer RPC client")?;
    eprintln!("runner: checking sequencer health at {sequencer}");
    tokio::time::timeout(RPC_HEALTH_TIMEOUT, client.check_health())
        .await
        .context("sequencer health check timed out")?
        .context("sequencer health check")?;
    eprintln!("runner: sequencer health check passed");

    let program = Program::new(PRIVATE_MULTISIG_ELF.to_vec().into())
        .context("load private multisig program")?;
    anyhow::ensure!(
        program.id() == PRIVATE_MULTISIG_ID,
        "embedded program id mismatch"
    );
    let target_program =
        Program::new(target_program_bytes.clone().into()).context("load target program")?;

    if let (Some(config_path), Some(proposal_path), Some(aggregate_path)) =
        (config_path, proposal_path, aggregate_path)
    {
        return run_workspace_evidence(
            sequencer,
            out_dir,
            poll_seconds,
            client,
            program,
            program_path,
            target_program,
            target_program_binary,
            config_path,
            proposal_path,
            aggregate_path,
        )
        .await;
    }

    eprintln!(
        "runner: deploying private multisig program {}",
        program_id_hex(program.id())
    );
    let deploy_hash = deploy_program(&client, PRIVATE_MULTISIG_ELF.to_vec())
        .await
        .context("submit private multisig program deployment")?;
    let deploy_included = poll_tx(&client, deploy_hash, poll_seconds).await?;
    eprintln!(
        "runner: private multisig deploy {} included={deploy_included}",
        deploy_hash
    );
    eprintln!(
        "runner: deploying target program {}",
        program_id_hex(target_program.id())
    );
    let target_deploy_hash = deploy_program(&client, target_program_bytes)
        .await
        .context("submit target program deployment")?;
    let target_deploy_included = poll_tx(&client, target_deploy_hash, poll_seconds).await?;
    eprintln!(
        "runner: target program deploy {} included={target_deploy_included}",
        target_deploy_hash
    );

    let create_key = create_key_for_run(&sequencer, &out_dir);
    let members = vec![
        member(create_key, 1),
        member(create_key, 2),
        member(create_key, 3),
    ];
    let leaves: Vec<Hash32> = members
        .iter()
        .map(|m| member_leaf(&m.multisig_id, &m.npk, &m.membership_secret))
        .collect();
    let config = build_config(create_key, 2, &leaves)?;

    let state_account = AccountId::for_public_pda(
        &program.id(),
        &PdaSeed::new(multisig_state_pda_seed(create_key)),
    );
    let create_instruction = PrivateMultisigInstruction::CreateMultisig {
        create_key,
        threshold: config.threshold,
        member_count: config.member_count,
        member_root: config.member_root,
    };
    eprintln!("runner: creating multisig state account {state_account}");
    let create_hash = send_instruction(&client, &program, vec![state_account], create_instruction)
        .await
        .context("submit create_multisig")?;
    eprintln!("runner: create_multisig submitted as {create_hash}");
    let create_included = poll_tx(&client, create_hash, poll_seconds).await?;
    ensure_included("create_multisig", create_hash, create_included)?;

    let proposal_account = AccountId::for_public_pda(
        &program.id(),
        &PdaSeed::new(proposal_pda_seed(&create_key, 1)),
    );
    let target_pda_seed = hash_chunks(
        b"logos.lp0002.runner.target-pda.v1",
        &[&create_key, b"hello-world-target"],
    );
    let target_account = AccountId::for_public_pda(&program.id(), &PdaSeed::new(target_pda_seed));
    let target_accounts = vec![target_account];
    let target_program_id = target_program.id();
    let target_instruction_data = Program::serialize_instruction(b"threshold-approved".to_vec())?;
    let propose_instruction = PrivateMultisigInstruction::Propose {
        create_key,
        proposal_index: 1,
        target_program_id,
        target_instruction_data: target_instruction_data.clone(),
        target_account_count: target_accounts.len() as u8,
        pda_seeds: vec![target_pda_seed],
        authorized_indices: vec![0],
    };
    eprintln!("runner: creating proposal account {proposal_account}");
    let propose_hash = send_instruction(
        &client,
        &program,
        vec![state_account, proposal_account],
        propose_instruction,
    )
    .await
    .context("submit propose")?;
    eprintln!("runner: propose submitted as {propose_hash}");
    let propose_included = poll_tx(&client, propose_hash, poll_seconds).await?;
    ensure_included("propose", propose_hash, propose_included)?;

    let proposal_state_before = get_proposal_state(&client, proposal_account).await?;
    let proposal_view = proposal_state_before.as_proposal();
    let approvals = vec![
        approve(&members[0], &proposal_view)?,
        approve(&members[2], &proposal_view)?,
    ];
    eprintln!(
        "runner: generated {} private approvals, aggregating threshold proof",
        approvals.len()
    );
    let aggregate = private_multisig_core::aggregate(&config, &proposal_view, &leaves, &approvals)?;
    let execute_instruction = PrivateMultisigInstruction::ExecutePrivate {
        create_key,
        proposal_index: 1,
        aggregate: aggregate.clone(),
    };
    eprintln!("runner: executing proposal against target account {target_account}");
    let execute_hash = send_instruction(
        &client,
        &program,
        vec![state_account, proposal_account, target_account],
        execute_instruction,
    )
    .await
    .context("submit execute_private")?;
    eprintln!("runner: execute_private submitted as {execute_hash}");
    let execute_included = poll_tx(&client, execute_hash, poll_seconds).await?;
    ensure_included("execute_private", execute_hash, execute_included)?;

    eprintln!("runner: fetching final multisig, proposal, and target account state");
    let state = get_multisig_state(&client, state_account).await?;
    let proposal_state = get_proposal_state(&client, proposal_account).await?;
    let target_account_state = get_account(&client, target_account).await?;
    let evidence = Evidence {
        ok: deploy_included && create_included && propose_included && execute_included,
        sequencer,
        program_id: program.id(),
        program_id_hex: program_id_hex(program.id()),
        program_binary: program_path.display().to_string(),
        target_program_id: target_program.id(),
        target_program_id_hex: program_id_hex(target_program.id()),
        target_program_binary: target_program_binary.display().to_string(),
        multisig_state_account: state_account.to_string(),
        proposal_account: proposal_account.to_string(),
        target_accounts: target_accounts.iter().map(ToString::to_string).collect(),
        member_root: to_hex(&config.member_root),
        aggregate_hash: to_hex(&aggregate.aggregate_hash),
        txs: EvidenceTxs {
            private_multisig_deploy: tx_evidence(deploy_hash, deploy_included),
            target_deploy: tx_evidence(target_deploy_hash, target_deploy_included),
            create_multisig: tx_evidence(create_hash, create_included),
            propose: tx_evidence(propose_hash, propose_included),
            execute_private: tx_evidence(execute_hash, execute_included),
        },
        final_state: FinalState {
            multisig_transaction_index: state.transaction_index,
            proposal_status: format!("{:?}", proposal_state.status),
            proposal_approval_count: proposal_state.approval_count,
            proposal_executed_aggregate_hash: proposal_state
                .executed_aggregate_hash
                .map(|hash| to_hex(&hash)),
            target_account_program_id_hex: program_id_hex(target_account_state.program_owner),
            target_account_nonce: target_account_state.nonce.0,
            target_account_data_utf8: String::from_utf8_lossy(target_account_state.data.as_ref())
                .into_owned(),
        },
    };

    anyhow::ensure!(
        evidence.ok,
        "one or more transactions were not included before timeout"
    );
    anyhow::ensure!(
        state.transaction_index == 1,
        "unexpected multisig transaction index"
    );
    anyhow::ensure!(
        proposal_state.status == PrivateProposalStatus::Executed,
        "proposal was not executed"
    );
    anyhow::ensure!(
        proposal_state.executed_aggregate_hash == Some(aggregate.aggregate_hash),
        "aggregate hash not stored on proposal"
    );
    anyhow::ensure!(
        target_account_state.program_owner == target_program.id(),
        "target account owner does not match target program"
    );
    anyhow::ensure!(
        !target_account_state.data.as_ref().is_empty(),
        "target account data was not updated by the chained call"
    );

    let evidence_path = out_dir.join("localnet-evidence.json");
    fs::write(&evidence_path, serde_json::to_vec_pretty(&evidence)?)?;
    println!("{}", serde_json::to_string_pretty(&evidence)?);
    Ok(())
}

async fn run_workspace_evidence(
    sequencer: String,
    out_dir: PathBuf,
    poll_seconds: u64,
    client: sequencer_service_rpc::SequencerClient,
    program: Program,
    program_path: PathBuf,
    target_program: Program,
    target_program_binary: PathBuf,
    config_path: PathBuf,
    proposal_path: PathBuf,
    aggregate_path: PathBuf,
) -> Result<()> {
    let config = config_from_file(&read_json::<ConfigFile>(&config_path)?)?;
    let proposal = proposal_from_file(&read_json::<ProposalFile>(&proposal_path)?)?;
    let aggregate = aggregate_from_file(&read_json::<AggregateFile>(&aggregate_path)?)?;

    private_multisig_core::verify_aggregate(&config, &proposal, &aggregate)
        .context("verify workspace aggregate against config/proposal")?;
    anyhow::ensure!(
        proposal.multisig_id == config.multisig_id,
        "workspace proposal multisig id does not match config"
    );
    anyhow::ensure!(
        proposal.target_program_id == target_program.id(),
        "workspace proposal target program id does not match target program binary"
    );
    anyhow::ensure!(
        proposal.target_account_count as usize <= proposal.pda_seeds.len(),
        "workspace proposal target_account_count exceeds pda_seeds length"
    );
    anyhow::ensure!(
        proposal.authorized_indices.len() <= proposal.target_account_count as usize,
        "workspace proposal authorized_indices exceeds target account count"
    );

    eprintln!(
        "runner: deploying private multisig program {}",
        program_id_hex(program.id())
    );
    let deploy_hash = deploy_program(&client, PRIVATE_MULTISIG_ELF.to_vec())
        .await
        .context("submit private multisig program deployment")?;
    let deploy_included = poll_tx(&client, deploy_hash, poll_seconds).await?;
    eprintln!(
        "runner: private multisig deploy {} included={deploy_included}",
        deploy_hash
    );
    eprintln!(
        "runner: deploying target program {}",
        program_id_hex(target_program.id())
    );
    let target_deploy_hash = deploy_program(
        &client,
        fs::read(&target_program_binary)
            .with_context(|| format!("read {}", target_program_binary.display()))?,
    )
    .await
    .context("submit target program deployment")?;
    let target_deploy_included = poll_tx(&client, target_deploy_hash, poll_seconds).await?;
    eprintln!(
        "runner: target program deploy {} included={target_deploy_included}",
        target_deploy_hash
    );

    let create_key = config.multisig_id;
    let state_account = AccountId::for_public_pda(
        &program.id(),
        &PdaSeed::new(multisig_state_pda_seed(create_key)),
    );
    let create_instruction = PrivateMultisigInstruction::CreateMultisig {
        create_key,
        threshold: config.threshold,
        member_count: config.member_count,
        member_root: config.member_root,
    };
    eprintln!("runner: creating multisig state account {state_account}");
    let create_hash = send_instruction(&client, &program, vec![state_account], create_instruction)
        .await
        .context("submit create_multisig")?;
    eprintln!("runner: create_multisig submitted as {create_hash}");
    let create_included = poll_tx(&client, create_hash, poll_seconds).await?;
    ensure_included("create_multisig", create_hash, create_included)?;

    let proposal_account = AccountId::for_public_pda(
        &program.id(),
        &PdaSeed::new(proposal_pda_seed(&create_key, proposal.proposal_id)),
    );
    let target_accounts = proposal
        .pda_seeds
        .iter()
        .take(proposal.target_account_count as usize)
        .map(|seed| AccountId::for_public_pda(&program.id(), &PdaSeed::new(*seed)))
        .collect::<Vec<_>>();
    let propose_instruction = PrivateMultisigInstruction::Propose {
        create_key,
        proposal_index: proposal.proposal_id,
        target_program_id: proposal.target_program_id,
        target_instruction_data: proposal.target_instruction_data.clone(),
        target_account_count: proposal.target_account_count,
        pda_seeds: proposal.pda_seeds.clone(),
        authorized_indices: proposal.authorized_indices.clone(),
    };
    eprintln!("runner: creating proposal account {proposal_account}");
    let propose_hash = send_instruction(
        &client,
        &program,
        vec![state_account, proposal_account],
        propose_instruction,
    )
    .await
    .context("submit propose")?;
    eprintln!("runner: propose submitted as {propose_hash}");
    let propose_included = poll_tx(&client, propose_hash, poll_seconds).await?;
    ensure_included("propose", propose_hash, propose_included)?;

    let mut execute_accounts = vec![state_account, proposal_account];
    execute_accounts.extend(target_accounts.iter().copied());
    eprintln!(
        "runner: executing proposal against target account{} {}",
        if target_accounts.len() == 1 { "" } else { "s" },
        target_accounts
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join(", ")
    );
    let execute_instruction = PrivateMultisigInstruction::ExecutePrivate {
        create_key,
        proposal_index: proposal.proposal_id,
        aggregate: aggregate.clone(),
    };
    let execute_hash = send_instruction(&client, &program, execute_accounts, execute_instruction)
        .await
        .context("submit execute_private")?;
    eprintln!("runner: execute_private submitted as {execute_hash}");
    let execute_included = poll_tx(&client, execute_hash, poll_seconds).await?;
    ensure_included("execute_private", execute_hash, execute_included)?;

    eprintln!("runner: fetching final multisig, proposal, and target account state");
    let state = get_multisig_state(&client, state_account).await?;
    let proposal_state = get_proposal_state(&client, proposal_account).await?;
    let primary_target_account = *target_accounts
        .first()
        .context("workspace proposal produced no target accounts")?;
    let target_account_state = get_account(&client, primary_target_account).await?;
    let evidence = Evidence {
        ok: deploy_included && create_included && propose_included && execute_included,
        sequencer,
        program_id: program.id(),
        program_id_hex: program_id_hex(program.id()),
        program_binary: program_path.display().to_string(),
        target_program_id: target_program.id(),
        target_program_id_hex: program_id_hex(target_program.id()),
        target_program_binary: target_program_binary.display().to_string(),
        multisig_state_account: state_account.to_string(),
        proposal_account: proposal_account.to_string(),
        target_accounts: target_accounts.iter().map(ToString::to_string).collect(),
        member_root: to_hex(&config.member_root),
        aggregate_hash: to_hex(&aggregate.aggregate_hash),
        txs: EvidenceTxs {
            private_multisig_deploy: tx_evidence(deploy_hash, deploy_included),
            target_deploy: tx_evidence(target_deploy_hash, target_deploy_included),
            create_multisig: tx_evidence(create_hash, create_included),
            propose: tx_evidence(propose_hash, propose_included),
            execute_private: tx_evidence(execute_hash, execute_included),
        },
        final_state: FinalState {
            multisig_transaction_index: state.transaction_index,
            proposal_status: format!("{:?}", proposal_state.status),
            proposal_approval_count: proposal_state.approval_count,
            proposal_executed_aggregate_hash: proposal_state
                .executed_aggregate_hash
                .map(|hash| to_hex(&hash)),
            target_account_program_id_hex: program_id_hex(target_account_state.program_owner),
            target_account_nonce: target_account_state.nonce.0,
            target_account_data_utf8: String::from_utf8_lossy(target_account_state.data.as_ref())
                .into_owned(),
        },
    };

    anyhow::ensure!(
        evidence.ok,
        "one or more transactions were not included before timeout"
    );
    anyhow::ensure!(
        state.transaction_index == 1,
        "unexpected multisig transaction index"
    );
    anyhow::ensure!(
        proposal_state.status == PrivateProposalStatus::Executed,
        "proposal was not executed"
    );
    anyhow::ensure!(
        proposal_state.executed_aggregate_hash == Some(aggregate.aggregate_hash),
        "aggregate hash not stored on proposal"
    );
    anyhow::ensure!(
        target_account_state.program_owner == target_program.id(),
        "target account owner does not match target program"
    );
    anyhow::ensure!(
        !target_account_state.data.as_ref().is_empty(),
        "target account data was not updated by the chained call"
    );

    let evidence_path = out_dir.join("localnet-evidence.json");
    fs::write(&evidence_path, serde_json::to_vec_pretty(&evidence)?)?;
    println!("{}", serde_json::to_string_pretty(&evidence)?);
    Ok(())
}

fn write_program_binary(out: &PathBuf) -> Result<()> {
    if let Some(parent) = out.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(out, PRIVATE_MULTISIG_ELF)?;
    Ok(())
}

fn write_proposal_template(
    multisig_id: String,
    proposal_id: u64,
    target_program_binary: PathBuf,
    target_program_id_override: Option<String>,
    instruction_words_override: Option<String>,
    target_account_count: u8,
    out: PathBuf,
) -> Result<()> {
    let multisig_id = from_hex32(&multisig_id).context("invalid --multisig-id hex")?;
    let target_program_bytes = fs::read(&target_program_binary)
        .with_context(|| format!("read {}", target_program_binary.display()))?;
    let target_program =
        Program::new(target_program_bytes.into()).context("load target program")?;
    let target_program_id = if let Some(value) = target_program_id_override {
        parse_program_id(&value)?
    } else {
        target_program.id()
    };
    let target_instruction_data = if let Some(value) = instruction_words_override {
        parse_words(&value)?
    } else {
        Program::serialize_instruction(b"threshold-approved".to_vec())?
    };
    let pda_seeds = (0..target_account_count)
        .map(|idx| {
            hash_chunks(
                b"logos.lp0002.basecamp.target-pda.v1",
                &[&multisig_id, &proposal_id.to_le_bytes(), &[idx]],
            )
        })
        .collect::<Vec<_>>();
    let authorized_indices = (0..target_account_count).collect::<Vec<_>>();
    let file = ProposalFile {
        multisig_id: to_hex(&multisig_id),
        proposal_id,
        target_program_id,
        target_instruction_data,
        target_account_count,
        pda_seeds: pda_seeds.iter().map(to_hex).collect(),
        authorized_indices,
    };
    write_json(&out, &file)?;
    println!("{}", serde_json::to_string_pretty(&file)?);
    Ok(())
}

fn member(multisig_id: Hash32, byte: u8) -> MemberSecret {
    MemberSecret {
        multisig_id,
        npk: [byte; 32],
        membership_secret: [byte + 20; 32],
    }
}

fn create_key_for_run(sequencer: &str, out_dir: &PathBuf) -> Hash32 {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos()
        .to_le_bytes();
    hash_chunks(
        b"logos.lp0002.runner.create-key.v1",
        &[
            sequencer.as_bytes(),
            out_dir.to_string_lossy().as_bytes(),
            &now,
        ],
    )
}

fn read_json<T: serde::de::DeserializeOwned>(path: &PathBuf) -> Result<T> {
    let bytes = fs::read(path).with_context(|| format!("read {}", path.display()))?;
    serde_json::from_slice(&bytes).with_context(|| format!("parse {}", path.display()))
}

fn write_json<T: Serialize>(path: &PathBuf, value: &T) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    let bytes = serde_json::to_vec_pretty(value)?;
    fs::write(path, bytes).with_context(|| format!("write {}", path.display()))
}

fn from_hex32(value: &str) -> Result<Hash32> {
    let bytes = hex::decode(value.trim_start_matches("0x"))?;
    anyhow::ensure!(bytes.len() == 32, "expected 32-byte hex value");
    let mut out = [0u8; 32];
    out.copy_from_slice(&bytes);
    Ok(out)
}

fn parse_words(value: &str) -> Result<Vec<u32>> {
    if value.trim().is_empty() {
        return Ok(Vec::new());
    }
    value
        .split(',')
        .map(|part| {
            let part = part.trim();
            if let Some(hex) = part.strip_prefix("0x") {
                u32::from_str_radix(hex, 16).context("invalid hex u32 word")
            } else {
                part.parse::<u32>().context("invalid decimal u32 word")
            }
        })
        .collect()
}

fn parse_program_id(value: &str) -> Result<[u32; 8]> {
    let words = parse_words(value)?;
    anyhow::ensure!(
        words.len() == 8,
        "program id must contain exactly 8 u32 words"
    );
    Ok(words.try_into().expect("checked length"))
}

fn config_from_file(file: &ConfigFile) -> Result<private_multisig_core::MultisigConfig> {
    Ok(private_multisig_core::MultisigConfig {
        multisig_id: from_hex32(&file.multisig_id).context("invalid config multisig_id")?,
        threshold: file.threshold,
        member_count: file.member_count,
        member_root: from_hex32(&file.member_root).context("invalid config member_root")?,
    })
}

fn proposal_from_file(file: &ProposalFile) -> Result<private_multisig_core::Proposal> {
    Ok(private_multisig_core::Proposal {
        multisig_id: from_hex32(&file.multisig_id).context("invalid proposal multisig_id")?,
        proposal_id: file.proposal_id,
        target_program_id: file.target_program_id,
        target_instruction_data: file.target_instruction_data.clone(),
        target_account_count: file.target_account_count,
        pda_seeds: file
            .pda_seeds
            .iter()
            .map(|seed| from_hex32(seed).context("invalid proposal pda seed"))
            .collect::<Result<Vec<_>>>()?,
        authorized_indices: file.authorized_indices.clone(),
    })
}

fn aggregate_from_file(file: &AggregateFile) -> Result<private_multisig_core::AggregateApproval> {
    Ok(private_multisig_core::AggregateApproval {
        multisig_id: from_hex32(&file.multisig_id).context("invalid aggregate multisig_id")?,
        proposal_id: file.proposal_id,
        member_root: from_hex32(&file.member_root).context("invalid aggregate member_root")?,
        threshold: file.threshold,
        approval_count: file.approval_count,
        proposal_hash: from_hex32(&file.proposal_hash)
            .context("invalid aggregate proposal_hash")?,
        nullifiers: file
            .nullifiers
            .iter()
            .map(|value| from_hex32(value).context("invalid aggregate nullifier"))
            .collect::<Result<Vec<_>>>()?,
        aggregate_hash: from_hex32(&file.aggregate_hash).context("invalid aggregate_hash")?,
    })
}

fn ensure_included(label: &str, hash: HashType, included: bool) -> Result<()> {
    anyhow::ensure!(
        included,
        "{label} transaction {hash} was not included before timeout"
    );
    Ok(())
}

async fn send_instruction(
    client: &sequencer_service_rpc::SequencerClient,
    program: &Program,
    account_ids: Vec<AccountId>,
    instruction: PrivateMultisigInstruction,
) -> Result<HashType> {
    let instruction_data = Program::serialize_instruction(instruction)?;
    let message = lee::public_transaction::Message::new_preserialized(
        program.id(),
        account_ids,
        vec![],
        instruction_data,
    );
    let witness_set = lee::public_transaction::WitnessSet::from_raw_parts(vec![]);
    let tx = lee::PublicTransaction::new(message, witness_set);
    Ok(tokio::time::timeout(
        RPC_SEND_TIMEOUT,
        client.send_transaction(LeeTransaction::Public(tx)),
    )
    .await
    .context("submit public transaction timed out")?
    .context("submit public transaction")?)
}

async fn deploy_program(
    client: &sequencer_service_rpc::SequencerClient,
    program_bytes: Vec<u8>,
) -> Result<HashType> {
    let tx = lee::ProgramDeploymentTransaction::new(
        lee::program_deployment_transaction::Message::new(program_bytes),
    );
    Ok(tokio::time::timeout(
        RPC_SEND_TIMEOUT,
        client.send_transaction(LeeTransaction::ProgramDeployment(tx)),
    )
    .await
    .context("submit program deployment transaction timed out")?
    .context("submit program deployment transaction")?)
}

async fn poll_tx(
    client: &sequencer_service_rpc::SequencerClient,
    tx_hash: HashType,
    timeout_seconds: u64,
) -> Result<bool> {
    let deadline = std::time::Instant::now() + Duration::from_secs(timeout_seconds.max(1));
    loop {
        match tokio::time::timeout(RPC_GET_TX_TIMEOUT, client.get_transaction(tx_hash)).await {
            Ok(Ok(Some(_))) => return Ok(true),
            Ok(Ok(None)) => {}
            Ok(Err(error)) => {
                eprintln!("runner: get_transaction {tx_hash} returned RPC error: {error:#}");
            }
            Err(_) => {
                eprintln!("runner: get_transaction {tx_hash} timed out, retrying");
            }
        }
        if std::time::Instant::now() >= deadline {
            return Ok(false);
        }
        tokio::time::sleep(Duration::from_secs(1)).await;
    }
}

async fn get_account(
    client: &sequencer_service_rpc::SequencerClient,
    account_id: AccountId,
) -> Result<lee_core::account::Account> {
    tokio::time::timeout(RPC_GET_ACCOUNT_TIMEOUT, client.get_account(account_id))
        .await
        .with_context(|| format!("get_account {account_id} timed out"))?
        .with_context(|| format!("get_account {account_id} failed"))
}

async fn get_multisig_state(
    client: &sequencer_service_rpc::SequencerClient,
    account_id: AccountId,
) -> Result<PrivateMultisigState> {
    let account = get_account(client, account_id).await?;
    Ok(borsh::from_slice(account.data.as_ref())?)
}

async fn get_proposal_state(
    client: &sequencer_service_rpc::SequencerClient,
    account_id: AccountId,
) -> Result<PrivateProposalState> {
    let account = get_account(client, account_id).await?;
    Ok(borsh::from_slice(account.data.as_ref())?)
}

fn tx_evidence(hash: HashType, included: bool) -> TxEvidence {
    TxEvidence {
        hash: hash.to_string(),
        included,
    }
}

fn program_id_hex(program_id: ProgramId) -> String {
    hex::encode(
        program_id
            .into_iter()
            .flat_map(u32::to_le_bytes)
            .collect::<Vec<u8>>(),
    )
}
