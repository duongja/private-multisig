use anyhow::{bail, Context, Result};
use clap::{Parser, Subcommand};
use private_multisig_core::{
    aggregate, approve, build_config, from_hex, member_leaf, to_hex, verify_aggregate,
    ApprovalShare, Hash32, MemberSecret, Proposal,
};
#[cfg(feature = "prove")]
use private_multisig_core::{
    aggregate_with_paths, merkle_proof, AggregateWitness, ApprovalWitness,
};
#[cfg(feature = "prove")]
use private_multisig_methods::{AGGREGATE_ELF, AGGREGATE_ID};
use rand::{rngs::OsRng, RngCore};
#[cfg(feature = "prove")]
use risc0_zkvm::{default_prover, ExecutorEnv};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
#[cfg(feature = "prove")]
use std::time::Instant;
use std::{fs, path::PathBuf};

#[derive(Parser, Debug)]
#[command(name = "private-multisig")]
#[command(about = "LP-0002 private M-of-N multisig CLI")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
    GenerateMember {
        #[arg(long)]
        multisig_id: Option<String>,
        #[arg(long)]
        out: PathBuf,
    },
    CreateConfig {
        #[arg(long)]
        multisig_id: String,
        #[arg(long)]
        threshold: u8,
        #[arg(long, required = true)]
        member: Vec<PathBuf>,
        #[arg(long)]
        out: PathBuf,
    },
    CreateProposal {
        #[arg(long)]
        multisig_id: String,
        #[arg(long)]
        proposal_id: u64,
        #[arg(long)]
        target_program_id: String,
        #[arg(long, default_value = "")]
        instruction_words: String,
        #[arg(long, default_value_t = 0)]
        target_account_count: u8,
        #[arg(long)]
        out: PathBuf,
    },
    Approve {
        #[arg(long)]
        member: PathBuf,
        #[arg(long)]
        proposal: PathBuf,
        #[arg(long)]
        out: PathBuf,
    },
    Aggregate {
        #[arg(long)]
        config: PathBuf,
        #[arg(long)]
        proposal: PathBuf,
        #[arg(long, required = true)]
        member: Vec<PathBuf>,
        #[arg(long, required = true)]
        approval: Vec<PathBuf>,
        #[arg(long)]
        out: PathBuf,
    },
    Verify {
        #[arg(long)]
        config: PathBuf,
        #[arg(long)]
        proposal: PathBuf,
        #[arg(long)]
        aggregate: PathBuf,
    },
    Prove {
        #[arg(long)]
        config: PathBuf,
        #[arg(long)]
        proposal: PathBuf,
        #[arg(long, required = true)]
        member: Vec<PathBuf>,
        #[arg(long, required = true)]
        approval: Vec<PathBuf>,
        #[arg(long)]
        out_dir: PathBuf,
    },
}

#[derive(Debug, Serialize, Deserialize)]
struct MemberFile {
    multisig_id: String,
    npk: String,
    membership_secret: String,
    leaf: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct ConfigFile {
    multisig_id: String,
    threshold: u8,
    member_count: u8,
    member_root: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct ProposalFile {
    multisig_id: String,
    proposal_id: u64,
    target_program_id: [u32; 8],
    target_instruction_data: Vec<u32>,
    target_account_count: u8,
    pda_seeds: Vec<String>,
    authorized_indices: Vec<u8>,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Command::GenerateMember { multisig_id, out } => {
            let multisig_id = match multisig_id {
                Some(value) => from_hex(&value).context("invalid --multisig-id hex")?,
                None => random_hash(),
            };
            let secret = MemberSecret {
                multisig_id,
                npk: random_hash(),
                membership_secret: random_hash(),
            };
            let leaf = member_leaf(&secret.multisig_id, &secret.npk, &secret.membership_secret);
            let file = MemberFile {
                multisig_id: to_hex(&secret.multisig_id),
                npk: to_hex(&secret.npk),
                membership_secret: to_hex(&secret.membership_secret),
                leaf: to_hex(&leaf),
            };
            write_json(&out, &file)?;
            println!("{}", serde_json::to_string_pretty(&file)?);
        }
        Command::CreateConfig {
            multisig_id,
            threshold,
            member,
            out,
        } => {
            let multisig_id = from_hex(&multisig_id).context("invalid --multisig-id hex")?;
            let members = read_members(&member)?;
            let leaves = members
                .iter()
                .map(|member| from_hex(&member.leaf))
                .collect::<Result<Vec<_>, _>>()
                .context("invalid member leaf")?;
            let config = build_config(multisig_id, threshold, &leaves)?;
            let file = ConfigFile {
                multisig_id: to_hex(&config.multisig_id),
                threshold: config.threshold,
                member_count: config.member_count,
                member_root: to_hex(&config.member_root),
            };
            write_json(&out, &file)?;
            println!("{}", serde_json::to_string_pretty(&file)?);
        }
        Command::CreateProposal {
            multisig_id,
            proposal_id,
            target_program_id,
            instruction_words,
            target_account_count,
            out,
        } => {
            let proposal = Proposal {
                multisig_id: from_hex(&multisig_id).context("invalid --multisig-id hex")?,
                proposal_id,
                target_program_id: parse_program_id(&target_program_id)?,
                target_instruction_data: parse_words(&instruction_words)?,
                target_account_count,
                pda_seeds: Vec::new(),
                authorized_indices: Vec::new(),
            };
            let file = proposal_to_file(&proposal);
            write_json(&out, &file)?;
            println!("{}", serde_json::to_string_pretty(&file)?);
        }
        Command::Approve {
            member,
            proposal,
            out,
        } => {
            let member = read_member_secret(&member)?;
            let proposal = proposal_from_file(&read_json::<ProposalFile>(&proposal)?)?;
            let approval = approve(&member, &proposal)?;
            write_json(&out, &approval_to_file(&approval))?;
            println!(
                "{}",
                serde_json::to_string_pretty(&approval_to_file(&approval))?
            );
        }
        Command::Aggregate {
            config,
            proposal,
            member,
            approval,
            out,
        } => {
            let config = config_from_file(&read_json::<ConfigFile>(&config)?)?;
            let proposal = proposal_from_file(&read_json::<ProposalFile>(&proposal)?)?;
            let members = read_members(&member)?;
            let leaves = members
                .iter()
                .map(|member| from_hex(&member.leaf))
                .collect::<Result<Vec<_>, _>>()
                .context("invalid member leaf")?;
            let approvals = approval
                .iter()
                .map(|path| {
                    read_json::<ApprovalFile>(path).and_then(|file| approval_from_file(&file))
                })
                .collect::<Result<Vec<_>>>()?;
            let aggregate = aggregate(&config, &proposal, &leaves, &approvals)?;
            write_json(&out, &aggregate_to_file(&aggregate))?;
            println!(
                "{}",
                serde_json::to_string_pretty(&aggregate_to_file(&aggregate))?
            );
        }
        Command::Verify {
            config,
            proposal,
            aggregate,
        } => {
            let config = config_from_file(&read_json::<ConfigFile>(&config)?)?;
            let proposal = proposal_from_file(&read_json::<ProposalFile>(&proposal)?)?;
            let aggregate = aggregate_from_file(&read_json::<AggregateFile>(&aggregate)?)?;
            verify_aggregate(&config, &proposal, &aggregate)?;
            println!(r#"{{"ok":true}}"#);
        }
        Command::Prove {
            config,
            proposal,
            member,
            approval,
            out_dir,
        } => {
            #[cfg(not(feature = "prove"))]
            {
                let _ = (config, proposal, member, approval, out_dir);
                bail!("the prove command requires building the CLI with --features prove");
            }
            #[cfg(feature = "prove")]
            {
                let witness = build_witness(&config, &proposal, &member, &approval)?;
                let expected =
                    aggregate_with_paths(&witness.config, &witness.proposal, &witness.approvals)?;
                fs::create_dir_all(&out_dir)
                    .with_context(|| format!("create {}", out_dir.display()))?;
                write_json(
                    &out_dir.join("witness-public.json"),
                    &witness_public_file(&witness),
                )?;

                let env = ExecutorEnv::builder()
                    .write(&witness)
                    .context("write witness to RISC0 executor")?
                    .build()
                    .context("build RISC0 executor env")?;
                let start = Instant::now();
                let prove_info = default_prover()
                    .prove(env, AGGREGATE_ELF)
                    .context("RISC0 aggregate proof failed")?;
                let prove_seconds = start.elapsed().as_secs_f64();
                prove_info
                    .receipt
                    .verify(AGGREGATE_ID)
                    .context("RISC0 aggregate receipt verification failed")?;
                let journal: private_multisig_core::AggregateApproval = prove_info
                    .receipt
                    .journal
                    .decode()
                    .context("decode aggregate proof journal")?;
                if journal != expected {
                    bail!("proof journal does not match host aggregate result");
                }

                write_json(&out_dir.join("journal.json"), &aggregate_to_file(&journal))?;
                write_json(
                    &out_dir.join("proof-stats.json"),
                    &ProofStatsFile {
                        ok: true,
                        image_id: hex::encode(bytemuck_words_to_bytes(&AGGREGATE_ID)),
                        prove_seconds,
                        total_cycles: prove_info.stats.total_cycles,
                        user_cycles: prove_info.stats.user_cycles,
                        paging_cycles: prove_info.stats.paging_cycles,
                        segments: prove_info.stats.segments,
                    },
                )?;
                println!(
                    "{}",
                    serde_json::to_string_pretty(&ProveOutputFile {
                        ok: true,
                        out_dir: out_dir.display().to_string(),
                        journal: out_dir.join("journal.json").display().to_string(),
                        stats: out_dir.join("proof-stats.json").display().to_string(),
                        aggregate_hash: to_hex(&journal.aggregate_hash),
                    })?
                );
            }
        }
    }
    Ok(())
}

fn random_hash() -> Hash32 {
    let mut value = [0u8; 32];
    OsRng.fill_bytes(&mut value);
    value
}

fn read_members(paths: &[PathBuf]) -> Result<Vec<MemberFile>> {
    paths.iter().map(read_json).collect()
}

fn read_member_secret(path: &PathBuf) -> Result<MemberSecret> {
    let file: MemberFile = read_json(path)?;
    Ok(MemberSecret {
        multisig_id: from_hex(&file.multisig_id).context("invalid member multisig_id")?,
        npk: from_hex(&file.npk).context("invalid member npk")?,
        membership_secret: from_hex(&file.membership_secret)
            .context("invalid member membership_secret")?,
    })
}

fn config_from_file(file: &ConfigFile) -> Result<private_multisig_core::MultisigConfig> {
    Ok(private_multisig_core::MultisigConfig {
        multisig_id: from_hex(&file.multisig_id).context("invalid config multisig_id")?,
        threshold: file.threshold,
        member_count: file.member_count,
        member_root: from_hex(&file.member_root).context("invalid config member_root")?,
    })
}

fn proposal_to_file(proposal: &Proposal) -> ProposalFile {
    ProposalFile {
        multisig_id: to_hex(&proposal.multisig_id),
        proposal_id: proposal.proposal_id,
        target_program_id: proposal.target_program_id,
        target_instruction_data: proposal.target_instruction_data.clone(),
        target_account_count: proposal.target_account_count,
        pda_seeds: proposal.pda_seeds.iter().map(to_hex).collect(),
        authorized_indices: proposal.authorized_indices.clone(),
    }
}

fn proposal_from_file(file: &ProposalFile) -> Result<Proposal> {
    Ok(Proposal {
        multisig_id: from_hex(&file.multisig_id).context("invalid proposal multisig_id")?,
        proposal_id: file.proposal_id,
        target_program_id: file.target_program_id,
        target_instruction_data: file.target_instruction_data.clone(),
        target_account_count: file.target_account_count,
        pda_seeds: file
            .pda_seeds
            .iter()
            .map(|seed| from_hex(seed).context("invalid proposal pda seed"))
            .collect::<Result<Vec<_>>>()?,
        authorized_indices: file.authorized_indices.clone(),
    })
}

#[derive(Debug, Serialize, Deserialize)]
struct ApprovalFile {
    multisig_id: String,
    proposal_id: u64,
    member_leaf: String,
    nullifier: String,
}

fn approval_to_file(approval: &ApprovalShare) -> ApprovalFile {
    ApprovalFile {
        multisig_id: to_hex(&approval.multisig_id),
        proposal_id: approval.proposal_id,
        member_leaf: to_hex(&approval.member_leaf),
        nullifier: to_hex(&approval.nullifier),
    }
}

fn approval_from_file(file: &ApprovalFile) -> Result<ApprovalShare> {
    Ok(ApprovalShare {
        multisig_id: from_hex(&file.multisig_id).context("invalid approval multisig_id")?,
        proposal_id: file.proposal_id,
        member_leaf: from_hex(&file.member_leaf).context("invalid approval member_leaf")?,
        nullifier: from_hex(&file.nullifier).context("invalid approval nullifier")?,
    })
}

#[derive(Debug, Serialize, Deserialize)]
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

fn aggregate_to_file(aggregate: &private_multisig_core::AggregateApproval) -> AggregateFile {
    AggregateFile {
        multisig_id: to_hex(&aggregate.multisig_id),
        proposal_id: aggregate.proposal_id,
        member_root: to_hex(&aggregate.member_root),
        threshold: aggregate.threshold,
        approval_count: aggregate.approval_count,
        proposal_hash: to_hex(&aggregate.proposal_hash),
        nullifiers: aggregate.nullifiers.iter().map(to_hex).collect(),
        aggregate_hash: to_hex(&aggregate.aggregate_hash),
    }
}

fn aggregate_from_file(file: &AggregateFile) -> Result<private_multisig_core::AggregateApproval> {
    Ok(private_multisig_core::AggregateApproval {
        multisig_id: from_hex(&file.multisig_id).context("invalid aggregate multisig_id")?,
        proposal_id: file.proposal_id,
        member_root: from_hex(&file.member_root).context("invalid aggregate member_root")?,
        threshold: file.threshold,
        approval_count: file.approval_count,
        proposal_hash: from_hex(&file.proposal_hash).context("invalid aggregate proposal_hash")?,
        nullifiers: file
            .nullifiers
            .iter()
            .map(|value| from_hex(value).context("invalid aggregate nullifier"))
            .collect::<Result<Vec<_>>>()?,
        aggregate_hash: from_hex(&file.aggregate_hash).context("invalid aggregate_hash")?,
    })
}

#[cfg(feature = "prove")]
fn build_witness(
    config: &PathBuf,
    proposal: &PathBuf,
    member: &[PathBuf],
    approval: &[PathBuf],
) -> Result<AggregateWitness> {
    let config = config_from_file(&read_json::<ConfigFile>(config)?)?;
    let proposal = proposal_from_file(&read_json::<ProposalFile>(proposal)?)?;
    let members = read_members(member)?;
    let all_member_leaves = members
        .iter()
        .map(|member| from_hex(&member.leaf).context("invalid member leaf"))
        .collect::<Result<Vec<_>>>()?;
    let approvals = approval
        .iter()
        .map(|path| read_json::<ApprovalFile>(path).and_then(|file| approval_from_file(&file)))
        .collect::<Result<Vec<_>>>()?;
    let approval_witnesses = approvals
        .into_iter()
        .map(|approval| {
            let membership_proof = merkle_proof(&all_member_leaves, &approval.member_leaf)
                .context("build approval Merkle proof")?;
            Ok(ApprovalWitness {
                approval,
                membership_proof,
            })
        })
        .collect::<Result<Vec<_>>>()?;
    Ok(AggregateWitness {
        config,
        proposal,
        approvals: approval_witnesses,
    })
}

#[cfg(feature = "prove")]
#[derive(Debug, Serialize, Deserialize)]
struct WitnessPublicFile {
    config: ConfigFile,
    proposal: ProposalFile,
    approvals: Vec<ApprovalWitnessFile>,
}

#[cfg(feature = "prove")]
#[derive(Debug, Serialize, Deserialize)]
struct ApprovalWitnessFile {
    member_leaf: String,
    nullifier: String,
    path_len: usize,
}

#[cfg(feature = "prove")]
fn witness_public_file(witness: &AggregateWitness) -> WitnessPublicFile {
    WitnessPublicFile {
        config: ConfigFile {
            multisig_id: to_hex(&witness.config.multisig_id),
            threshold: witness.config.threshold,
            member_count: witness.config.member_count,
            member_root: to_hex(&witness.config.member_root),
        },
        proposal: proposal_to_file(&witness.proposal),
        approvals: witness
            .approvals
            .iter()
            .map(|approval| ApprovalWitnessFile {
                member_leaf: to_hex(&approval.approval.member_leaf),
                nullifier: to_hex(&approval.approval.nullifier),
                path_len: approval.membership_proof.path.len(),
            })
            .collect(),
    }
}

#[cfg(feature = "prove")]
#[derive(Debug, Serialize, Deserialize)]
struct ProofStatsFile {
    ok: bool,
    image_id: String,
    prove_seconds: f64,
    total_cycles: u64,
    user_cycles: u64,
    paging_cycles: u64,
    segments: usize,
}

#[cfg(feature = "prove")]
#[derive(Debug, Serialize, Deserialize)]
struct ProveOutputFile {
    ok: bool,
    out_dir: String,
    journal: String,
    stats: String,
    aggregate_hash: String,
}

#[cfg(feature = "prove")]
fn bytemuck_words_to_bytes(words: &[u32; 8]) -> [u8; 32] {
    let mut out = [0u8; 32];
    for (idx, word) in words.iter().enumerate() {
        out[idx * 4..idx * 4 + 4].copy_from_slice(&word.to_le_bytes());
    }
    out
}

fn parse_program_id(value: &str) -> Result<[u32; 8]> {
    let words = parse_words(value)?;
    if words.len() != 8 {
        bail!("program id must contain exactly 8 u32 words");
    }
    Ok(words.try_into().expect("checked length"))
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

fn read_json<T: DeserializeOwned>(path: &PathBuf) -> Result<T> {
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
