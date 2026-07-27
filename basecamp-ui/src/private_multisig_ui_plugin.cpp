#include "private_multisig_ui_plugin.h"

#include "logos_api.h"

#include <algorithm>
#include <QCoreApplication>
#include <QDateTime>
#include <QDebug>
#include <QDir>
#include <QFile>
#include <QFileInfo>
#include <QJsonDocument>
#include <QJsonObject>
#include <QProcess>
#include <QStandardPaths>

PrivateMultisigUiPlugin::PrivateMultisigUiPlugin(QObject* parent)
    : PrivateMultisigUiSimpleSource(parent)
{
    setStatus(QStringLiteral("Ready"));
    setWorkspacePath(defaultWorkspacePath());
    setCliPath(resolveCliPath());
}

PrivateMultisigUiPlugin::~PrivateMultisigUiPlugin() = default;

void PrivateMultisigUiPlugin::initLogos(LogosAPI* api)
{
    logosAPI = api;
    m_logosAPI = api;
    setBackend(this);
    setCliPath(resolveCliPath());
    setWorkspacePath(defaultWorkspacePath());
    setStatus(QStringLiteral("Backend ready"));
    qDebug() << "PrivateMultisigUiPlugin initialized";
}

QString PrivateMultisigUiPlugin::health()
{
    const QString cli = resolveCliPath();
    setCliPath(cli);
    setWorkspacePath(defaultWorkspacePath());

    QVariantMap result;
    result.insert(QStringLiteral("ok"), QFileInfo::exists(cli));
    result.insert(QStringLiteral("cli_path"), cli);
    result.insert(QStringLiteral("runner_path"), resolveRunnerPath());
    result.insert(QStringLiteral("repo_root"), resolveRepoRoot());
    result.insert(QStringLiteral("target_program_binary"), resolveTargetProgramBinaryPath());
    result.insert(QStringLiteral("workspace_path"), workspacePath());
    result.insert(QStringLiteral("backend"), QStringLiteral("private_multisig_ui"));
    result.insert(QStringLiteral("status"), status());

    if (!QFileInfo::exists(cli)) {
        result.insert(
            QStringLiteral("error"),
            QStringLiteral("private_multisig_cli not found; set PRIVATE_MULTISIG_CLI or build target/debug/private_multisig_cli"));
        setStatus(QStringLiteral("CLI not found"));
    }

    return makeJson(result);
}

QString PrivateMultisigUiPlugin::loadWorkspaceState()
{
    const QString cli = resolveCliPath();
    setCliPath(cli);
    setWorkspacePath(defaultWorkspacePath());

    QVariantMap result;
    result.insert(QStringLiteral("ok"), QFileInfo::exists(cli));
    result.insert(QStringLiteral("cli_path"), cli);
    result.insert(QStringLiteral("workspace_path"), runRootPath());
    result.insert(QStringLiteral("artifacts"), artifactsMap());
    result.insert(QStringLiteral("status"), status());
    return makeJson(result);
}

QString PrivateMultisigUiPlugin::generateMembers(QString multisigId)
{
    const QString cli = resolveCliPath();
    setCliPath(cli);
    if (!QFileInfo::exists(cli)) {
        return health();
    }

    multisigId = multisigId.trimmed();
    if (multisigId.isEmpty()) {
        multisigId =
            QStringLiteral("1111111111111111111111111111111111111111111111111111111111111111");
    }

    QString error;
    if (!ensureWorkspace(&error)) {
        QVariantMap result;
        result.insert(QStringLiteral("ok"), false);
        result.insert(QStringLiteral("error"), error);
        return makeJson(result);
    }

    removeArtifacts(QStringList{QStringLiteral("alice.json"),
                                QStringLiteral("bob.json"),
                                QStringLiteral("carol.json"),
                                QStringLiteral("config.json"),
                                QStringLiteral("proposal.json"),
                                QStringLiteral("approval-alice.json"),
                                QStringLiteral("approval-bob.json"),
                                QStringLiteral("approval-carol.json"),
                                QStringLiteral("aggregate.json"),
                                QStringLiteral("duplicate-aggregate.json"),
                                QStringLiteral("proof")});

    QVariantMap commands;
    auto runStep = [&](const QString& step, const QString& memberId) {
        setStatus(QStringLiteral("Running %1").arg(step));
        const CommandResult command = runCli(
            QStringList{QStringLiteral("generate-member"),
                        QStringLiteral("--multisig-id"),
                        multisigId,
                        QStringLiteral("--out"),
                        memberKeyPath(memberId)});
        commands.insert(step, commandMap(step, command));
        return command;
    };

    const CommandResult alice = runStep(QStringLiteral("generate_alice"), QStringLiteral("alice"));
    const CommandResult bob = runStep(QStringLiteral("generate_bob"), QStringLiteral("bob"));
    const CommandResult carol = runStep(QStringLiteral("generate_carol"), QStringLiteral("carol"));
    const bool ok = alice.exitCode == 0 && bob.exitCode == 0 && carol.exitCode == 0;
    setStatus(ok ? QStringLiteral("Members generated") : QStringLiteral("Member generation failed"));

    QVariantMap result;
    result.insert(QStringLiteral("ok"), ok);
    result.insert(QStringLiteral("multisig_id"), multisigId);
    result.insert(QStringLiteral("cli_path"), cli);
    result.insert(QStringLiteral("workspace_path"), runRootPath());
    result.insert(QStringLiteral("commands"), commands);
    result.insert(QStringLiteral("artifacts"), artifactsMap());
    return makeJson(result);
}

QString PrivateMultisigUiPlugin::createConfig(QString multisigId, int threshold)
{
    const QString cli = resolveCliPath();
    setCliPath(cli);
    if (!QFileInfo::exists(cli)) {
        return health();
    }

    multisigId = multisigId.trimmed();
    if (multisigId.isEmpty()) {
        multisigId =
            QStringLiteral("1111111111111111111111111111111111111111111111111111111111111111");
    }
    if (threshold < 1 || threshold > 3) {
        QVariantMap result;
        result.insert(QStringLiteral("ok"), false);
        result.insert(QStringLiteral("error"), QStringLiteral("threshold must be between 1 and 3"));
        return makeJson(result);
    }

    QString error;
    if (!ensureWorkspace(&error)) {
        QVariantMap result;
        result.insert(QStringLiteral("ok"), false);
        result.insert(QStringLiteral("error"), error);
        return makeJson(result);
    }

    removeArtifacts(QStringList{QStringLiteral("config.json"),
                                QStringLiteral("proposal.json"),
                                QStringLiteral("approval-alice.json"),
                                QStringLiteral("approval-bob.json"),
                                QStringLiteral("approval-carol.json"),
                                QStringLiteral("aggregate.json"),
                                QStringLiteral("duplicate-aggregate.json"),
                                QStringLiteral("proof")});

    setStatus(QStringLiteral("Running create_config"));
    const CommandResult command = runCli(
        QStringList{QStringLiteral("create-config"),
                    QStringLiteral("--multisig-id"),
                    multisigId,
                    QStringLiteral("--threshold"),
                    QString::number(threshold),
                    QStringLiteral("--member"),
                    memberKeyPath(QStringLiteral("alice")),
                    QStringLiteral("--member"),
                    memberKeyPath(QStringLiteral("bob")),
                    QStringLiteral("--member"),
                    memberKeyPath(QStringLiteral("carol")),
                    QStringLiteral("--out"),
                    jsonFile(QStringLiteral("config.json"))});

    QVariantMap commands;
    commands.insert(QStringLiteral("create_config"), commandMap(QStringLiteral("create_config"), command));

    const bool ok = command.exitCode == 0;
    setStatus(ok ? QStringLiteral("Config created") : QStringLiteral("Config failed"));

    QVariantMap result;
    result.insert(QStringLiteral("ok"), ok);
    result.insert(QStringLiteral("multisig_id"), multisigId);
    result.insert(QStringLiteral("threshold"), threshold);
    result.insert(QStringLiteral("cli_path"), cli);
    result.insert(QStringLiteral("workspace_path"), runRootPath());
    result.insert(QStringLiteral("commands"), commands);
    result.insert(QStringLiteral("artifacts"), artifactsMap());
    return makeJson(result);
}

QString PrivateMultisigUiPlugin::createProposal(QString multisigId,
                                                QString proposalId,
                                                QString targetProgramId,
                                                QString instructionWords,
                                                int targetAccountCount)
{
    const QString cli = resolveCliPath();
    const QString runner = resolveRunnerPath();
    setCliPath(cli);
    if (!QFileInfo::exists(cli) && !QFileInfo::exists(runner)) {
        return health();
    }

    multisigId = multisigId.trimmed();
    if (multisigId.isEmpty()) {
        multisigId =
            QStringLiteral("1111111111111111111111111111111111111111111111111111111111111111");
    }
    proposalId = proposalId.trimmed().isEmpty() ? QStringLiteral("1") : proposalId.trimmed();
    targetProgramId = targetProgramId.trimmed().isEmpty()
        ? QStringLiteral("1,2,3,4,5,6,7,8")
        : targetProgramId.trimmed();

    QString error;
    if (!ensureWorkspace(&error)) {
        QVariantMap result;
        result.insert(QStringLiteral("ok"), false);
        result.insert(QStringLiteral("error"), error);
        return makeJson(result);
    }

    removeArtifacts(QStringList{QStringLiteral("proposal.json"),
                                QStringLiteral("approval-alice.json"),
                                QStringLiteral("approval-bob.json"),
                                QStringLiteral("approval-carol.json"),
                                QStringLiteral("aggregate.json"),
                                QStringLiteral("duplicate-aggregate.json"),
                                QStringLiteral("proof")});

    const QString programId = targetProgramId.trimmed();
    const QString words = instructionWords.trimmed();
    const bool hasExplicitProgram = !programId.isEmpty()
        && programId != QStringLiteral("1,2,3,4,5,6,7,8");
    const bool hasExplicitInstruction = !words.isEmpty() && words != QStringLiteral("9,10");

    setStatus(QStringLiteral("Running create_proposal"));
    CommandResult command;
    if (QFileInfo::exists(runner)) {
        QStringList args{QStringLiteral("write-proposal-template"),
                         QStringLiteral("--multisig-id"),
                         multisigId,
                         QStringLiteral("--proposal-id"),
                         proposalId,
                         QStringLiteral("--target-program-binary"),
                         resolveTargetProgramBinaryPath(),
                         QStringLiteral("--target-account-count"),
                         QString::number(std::max(0, targetAccountCount)),
                         QStringLiteral("--out"),
                         jsonFile(QStringLiteral("proposal.json"))};
        if (hasExplicitProgram) {
            args << QStringLiteral("--target-program-id") << programId;
        }
        if (hasExplicitInstruction) {
            args << QStringLiteral("--instruction-words") << words;
        }
        command = runProcess(
            runner,
            args,
            resolveRepoRoot(),
            sanitizedScriptEnvironment({}),
            30000);
    } else {
        command = runCli(
            QStringList{QStringLiteral("create-proposal"),
                        QStringLiteral("--multisig-id"),
                        multisigId,
                        QStringLiteral("--proposal-id"),
                        proposalId,
                        QStringLiteral("--target-program-id"),
                        targetProgramId,
                        QStringLiteral("--instruction-words"),
                        instructionWords.trimmed(),
                        QStringLiteral("--target-account-count"),
                        QString::number(std::max(0, targetAccountCount)),
                        QStringLiteral("--out"),
                        jsonFile(QStringLiteral("proposal.json"))});
    }

    QVariantMap commands;
    commands.insert(QStringLiteral("create_proposal"),
                    commandMap(QStringLiteral("create_proposal"), command));
    const bool ok = command.exitCode == 0;
    setStatus(ok ? QStringLiteral("Proposal created") : QStringLiteral("Proposal failed"));

    QVariantMap result;
    result.insert(QStringLiteral("ok"), ok);
    result.insert(QStringLiteral("multisig_id"), multisigId);
    result.insert(QStringLiteral("proposal_id"), proposalId);
    result.insert(QStringLiteral("cli_path"), cli);
    result.insert(QStringLiteral("workspace_path"), runRootPath());
    result.insert(QStringLiteral("commands"), commands);
    result.insert(QStringLiteral("artifacts"), artifactsMap());
    return makeJson(result);
}

QString PrivateMultisigUiPlugin::approveMember(QString memberId)
{
    const QString cli = resolveCliPath();
    setCliPath(cli);
    if (!QFileInfo::exists(cli)) {
        return health();
    }

    const QString memberKey = memberId.trimmed().replace(QStringLiteral("member-"), QString());
    if (memberKey.isEmpty()) {
        QVariantMap result;
        result.insert(QStringLiteral("ok"), false);
        result.insert(QStringLiteral("error"), QStringLiteral("member id is required"));
        return makeJson(result);
    }

    QString error;
    if (!ensureWorkspace(&error)) {
        QVariantMap result;
        result.insert(QStringLiteral("ok"), false);
        result.insert(QStringLiteral("error"), error);
        return makeJson(result);
    }

    const QString step = QStringLiteral("approve_%1").arg(memberKey);
    setStatus(QStringLiteral("Running %1").arg(step));
    const CommandResult command = runCli(
        QStringList{QStringLiteral("approve"),
                    QStringLiteral("--member"),
                    memberKeyPath(memberKey),
                    QStringLiteral("--proposal"),
                    jsonFile(QStringLiteral("proposal.json")),
                    QStringLiteral("--out"),
                    approvalKeyPath(memberKey)});

    QVariantMap commands;
    commands.insert(step, commandMap(step, command));
    const bool ok = command.exitCode == 0;
    setStatus(ok ? QStringLiteral("Approval stored") : QStringLiteral("Approval failed"));

    QVariantMap result;
    result.insert(QStringLiteral("ok"), ok);
    result.insert(QStringLiteral("member_id"), memberKey);
    result.insert(QStringLiteral("cli_path"), cli);
    result.insert(QStringLiteral("workspace_path"), runRootPath());
    result.insert(QStringLiteral("commands"), commands);
    result.insert(QStringLiteral("artifacts"), artifactsMap());
    return makeJson(result);
}

QString PrivateMultisigUiPlugin::aggregateApprovals()
{
    const QString cli = resolveCliPath();
    setCliPath(cli);
    if (!QFileInfo::exists(cli)) {
        return health();
    }

    QString error;
    if (!ensureWorkspace(&error)) {
        QVariantMap result;
        result.insert(QStringLiteral("ok"), false);
        result.insert(QStringLiteral("error"), error);
        return makeJson(result);
    }

    QStringList args{QStringLiteral("aggregate"),
                     QStringLiteral("--config"),
                     jsonFile(QStringLiteral("config.json")),
                     QStringLiteral("--proposal"),
                     jsonFile(QStringLiteral("proposal.json")),
                     QStringLiteral("--member"),
                     memberKeyPath(QStringLiteral("alice")),
                     QStringLiteral("--member"),
                     memberKeyPath(QStringLiteral("bob")),
                     QStringLiteral("--member"),
                     memberKeyPath(QStringLiteral("carol"))};

    const QStringList memberKeys{QStringLiteral("alice"), QStringLiteral("bob"), QStringLiteral("carol")};
    QStringList approvalsPresent;
    for (const QString& key : memberKeys) {
        const QString approvalPath = approvalKeyPath(key);
        if (QFileInfo::exists(approvalPath)) {
            args << QStringLiteral("--approval") << approvalPath;
            approvalsPresent << key;
        }
    }
    args << QStringLiteral("--out") << jsonFile(QStringLiteral("aggregate.json"));

    setStatus(QStringLiteral("Running aggregate"));
    const CommandResult command = runCli(args);
    QVariantMap commands;
    commands.insert(QStringLiteral("aggregate"), commandMap(QStringLiteral("aggregate"), command));

    const bool ok = command.exitCode == 0;
    setStatus(ok ? QStringLiteral("Aggregate ready") : QStringLiteral("Aggregate failed"));

    QVariantMap result;
    result.insert(QStringLiteral("ok"), ok);
    result.insert(QStringLiteral("approval_files"), approvalsPresent);
    result.insert(QStringLiteral("cli_path"), cli);
    result.insert(QStringLiteral("workspace_path"), runRootPath());
    result.insert(QStringLiteral("commands"), commands);
    result.insert(QStringLiteral("artifacts"), artifactsMap());
    return makeJson(result);
}

QString PrivateMultisigUiPlugin::verifyAggregate()
{
    const QString cli = resolveCliPath();
    setCliPath(cli);
    if (!QFileInfo::exists(cli)) {
        return health();
    }

    QString error;
    if (!ensureWorkspace(&error)) {
        QVariantMap result;
        result.insert(QStringLiteral("ok"), false);
        result.insert(QStringLiteral("error"), error);
        return makeJson(result);
    }

    setStatus(QStringLiteral("Running verify"));
    const CommandResult command = runCli(
        QStringList{QStringLiteral("verify"),
                    QStringLiteral("--config"),
                    jsonFile(QStringLiteral("config.json")),
                    QStringLiteral("--proposal"),
                    jsonFile(QStringLiteral("proposal.json")),
                    QStringLiteral("--aggregate"),
                    jsonFile(QStringLiteral("aggregate.json"))});

    QVariantMap commands;
    commands.insert(QStringLiteral("verify"), commandMap(QStringLiteral("verify"), command));
    const bool ok = command.exitCode == 0;
    setStatus(ok ? QStringLiteral("Aggregate verified") : QStringLiteral("Verify failed"));

    QVariantMap result;
    result.insert(QStringLiteral("ok"), ok);
    result.insert(QStringLiteral("cli_path"), cli);
    result.insert(QStringLiteral("workspace_path"), runRootPath());
    result.insert(QStringLiteral("commands"), commands);
    result.insert(QStringLiteral("artifacts"), artifactsMap());
    return makeJson(result);
}

QString PrivateMultisigUiPlugin::proveAggregate()
{
    const QString cli = resolveCliPath();
    setCliPath(cli);
    if (!QFileInfo::exists(cli)) {
        return health();
    }

    QString error;
    if (!ensureWorkspace(&error)) {
        QVariantMap result;
        result.insert(QStringLiteral("ok"), false);
        result.insert(QStringLiteral("error"), error);
        return makeJson(result);
    }

    QStringList args{QStringLiteral("prove"),
                     QStringLiteral("--config"),
                     jsonFile(QStringLiteral("config.json")),
                     QStringLiteral("--proposal"),
                     jsonFile(QStringLiteral("proposal.json")),
                     QStringLiteral("--member"),
                     memberKeyPath(QStringLiteral("alice")),
                     QStringLiteral("--member"),
                     memberKeyPath(QStringLiteral("bob")),
                     QStringLiteral("--member"),
                     memberKeyPath(QStringLiteral("carol"))};
    const QStringList memberKeys{QStringLiteral("alice"), QStringLiteral("bob"), QStringLiteral("carol")};
    for (const QString& key : memberKeys) {
        const QString approvalPath = approvalKeyPath(key);
        if (QFileInfo::exists(approvalPath)) {
            args << QStringLiteral("--approval") << approvalPath;
        }
    }
    args << QStringLiteral("--out-dir") << absolutePath(QStringLiteral("proof"));

    setStatus(QStringLiteral("Running prove"));
    const CommandResult command = runCli(args, 120000);
    QVariantMap commands;
    commands.insert(QStringLiteral("prove"), commandMap(QStringLiteral("prove"), command));
    const bool ok = command.exitCode == 0;
    setStatus(ok ? QStringLiteral("Proof generated") : QStringLiteral("Prove failed"));

    QVariantMap result;
    result.insert(QStringLiteral("ok"), ok);
    result.insert(QStringLiteral("cli_path"), cli);
    result.insert(QStringLiteral("workspace_path"), runRootPath());
    result.insert(QStringLiteral("proof_dir"), absolutePath(QStringLiteral("proof")));
    result.insert(QStringLiteral("commands"), commands);
    result.insert(QStringLiteral("artifacts"), artifactsMap());
    return makeJson(result);
}

QString PrivateMultisigUiPlugin::runLocalnetExecution()
{
    const QString cli = resolveCliPath();
    setCliPath(cli);
    if (!QFileInfo::exists(cli)) {
        return health();
    }

    QString error;
    if (!ensureWorkspace(&error)) {
        QVariantMap result;
        result.insert(QStringLiteral("ok"), false);
        result.insert(QStringLiteral("error"), error);
        return makeJson(result);
    }

    const QString runRoot = absolutePath(QStringLiteral("onchain/localnet"));
    removeArtifacts(QStringList{QStringLiteral("onchain/localnet")});
    QDir().mkpath(runRoot);

    setStatus(QStringLiteral("Running localnet execution"));
    const CommandResult command = runRepoScript(
        QStringLiteral("localnet-evidence.sh"),
        QStringList{runRoot},
        QVariantMap{{QStringLiteral("RISC0_DEV_MODE"), QStringLiteral("0")},
                    {QStringLiteral("PRIVATE_MULTISIG_GUI_WORKSPACE"), runRootPath()}},
        600000);

    QVariantMap commands;
    commands.insert(QStringLiteral("localnet_execute"),
                    commandMap(QStringLiteral("localnet_execute"), command));
    const bool ok = command.exitCode == 0;
    setStatus(ok ? QStringLiteral("Localnet execution complete")
                 : QStringLiteral("Localnet execution failed"));

    QVariantMap result;
    result.insert(QStringLiteral("ok"), ok);
    result.insert(QStringLiteral("run_root"), runRoot);
    result.insert(QStringLiteral("cli_path"), cli);
    result.insert(QStringLiteral("runner_path"), resolveRunnerPath());
    result.insert(QStringLiteral("workspace_path"), runRootPath());
    result.insert(QStringLiteral("commands"), commands);
    result.insert(QStringLiteral("artifacts"), artifactsMap());
    return makeJson(result);
}

QString PrivateMultisigUiPlugin::runHostedTestnetExecution()
{
    const QString cli = resolveCliPath();
    setCliPath(cli);
    if (!QFileInfo::exists(cli)) {
        return health();
    }

    QString error;
    if (!ensureWorkspace(&error)) {
        QVariantMap result;
        result.insert(QStringLiteral("ok"), false);
        result.insert(QStringLiteral("error"), error);
        return makeJson(result);
    }

    const QString runRoot = absolutePath(QStringLiteral("onchain/testnet"));
    removeArtifacts(QStringList{QStringLiteral("onchain/testnet")});
    QDir().mkpath(runRoot);

    setStatus(QStringLiteral("Running hosted testnet execution"));
    const CommandResult command = runRepoScript(
        QStringLiteral("testnet-evidence.sh"),
        QStringList{runRoot},
        QVariantMap{{QStringLiteral("RISC0_DEV_MODE"), QStringLiteral("0")},
                    {QStringLiteral("PRIVATE_MULTISIG_GUI_WORKSPACE"), runRootPath()}},
        900000);

    QVariantMap commands;
    commands.insert(QStringLiteral("hosted_testnet_execute"),
                    commandMap(QStringLiteral("hosted_testnet_execute"), command));
    const bool ok = command.exitCode == 0;
    setStatus(ok ? QStringLiteral("Hosted testnet execution complete")
                 : QStringLiteral("Hosted testnet execution failed"));

    QVariantMap result;
    result.insert(QStringLiteral("ok"), ok);
    result.insert(QStringLiteral("run_root"), runRoot);
    result.insert(QStringLiteral("cli_path"), cli);
    result.insert(QStringLiteral("runner_path"), resolveRunnerPath());
    result.insert(QStringLiteral("workspace_path"), runRootPath());
    result.insert(QStringLiteral("commands"), commands);
    result.insert(QStringLiteral("artifacts"), artifactsMap());
    return makeJson(result);
}

QString PrivateMultisigUiPlugin::testDuplicateAggregate(QString memberId)
{
    const QString cli = resolveCliPath();
    setCliPath(cli);
    if (!QFileInfo::exists(cli)) {
        return health();
    }

    QString error;
    if (!ensureWorkspace(&error)) {
        QVariantMap result;
        result.insert(QStringLiteral("ok"), false);
        result.insert(QStringLiteral("error"), error);
        return makeJson(result);
    }

    const QString memberKey = memberId.trimmed().replace(QStringLiteral("member-"), QString()).isEmpty()
        ? QStringLiteral("alice")
        : memberId.trimmed().replace(QStringLiteral("member-"), QString());
    setStatus(QStringLiteral("Running duplicate check"));
    const CommandResult command = runCli(
        QStringList{QStringLiteral("aggregate"),
                    QStringLiteral("--config"),
                    jsonFile(QStringLiteral("config.json")),
                    QStringLiteral("--proposal"),
                    jsonFile(QStringLiteral("proposal.json")),
                    QStringLiteral("--member"),
                    memberKeyPath(QStringLiteral("alice")),
                    QStringLiteral("--member"),
                    memberKeyPath(QStringLiteral("bob")),
                    QStringLiteral("--member"),
                    memberKeyPath(QStringLiteral("carol")),
                    QStringLiteral("--approval"),
                    approvalKeyPath(memberKey),
                    QStringLiteral("--approval"),
                    approvalKeyPath(memberKey),
                    QStringLiteral("--out"),
                    jsonFile(QStringLiteral("duplicate-aggregate.json"))});

    QVariantMap commands;
    commands.insert(QStringLiteral("duplicate_aggregate"),
                    commandMap(QStringLiteral("duplicate_aggregate"), command));
    const bool rejected = command.exitCode != 0;
    setStatus(rejected ? QStringLiteral("Duplicate rejected") : QStringLiteral("Duplicate test failed"));

    QVariantMap result;
    result.insert(QStringLiteral("ok"), rejected);
    result.insert(QStringLiteral("duplicate_rejected"), rejected);
    result.insert(QStringLiteral("member_id"), memberKey);
    result.insert(QStringLiteral("cli_path"), cli);
    result.insert(QStringLiteral("workspace_path"), runRootPath());
    result.insert(QStringLiteral("commands"), commands);
    result.insert(QStringLiteral("artifacts"), artifactsMap());
    return makeJson(result);
}

QString PrivateMultisigUiPlugin::runDemoFlow(int threshold,
                                             QString proposalId,
                                             QString targetProgramId,
                                             QString instructionWords,
                                             int targetAccountCount)
{
    if (threshold < 1 || threshold > 3) {
        QVariantMap result;
        result.insert(QStringLiteral("ok"), false);
        result.insert(QStringLiteral("error"), QStringLiteral("threshold must be between 1 and 3"));
        setStatus(QStringLiteral("Invalid threshold"));
        return makeJson(result);
    }

    const QString cli = resolveCliPath();
    setCliPath(cli);
    if (!QFileInfo::exists(cli)) {
        return health();
    }

    QString error;
    if (!ensureWorkspace(&error)) {
        QVariantMap result;
        result.insert(QStringLiteral("ok"), false);
        result.insert(QStringLiteral("error"), error);
        setStatus(QStringLiteral("Workspace error"));
        return makeJson(result);
    }

    const QString multisigId =
        QStringLiteral("1111111111111111111111111111111111111111111111111111111111111111");
    const QString programId = targetProgramId.trimmed().isEmpty()
        ? QStringLiteral("1,2,3,4,5,6,7,8")
        : targetProgramId.trimmed();
    const QString proposalWords = instructionWords.trimmed();
    const QString pid = proposalId.trimmed().isEmpty() ? QStringLiteral("1") : proposalId.trimmed();

    QVariantMap commands;

    auto runStep = [&](const QString& step, const QStringList& args) -> CommandResult {
        setStatus(QStringLiteral("Running %1").arg(step));
        CommandResult command = runCli(args);
        commands.insert(step, commandMap(step, command));
        return command;
    };

    const CommandResult alice = runStep(
        QStringLiteral("generate_alice"),
        QStringList{QStringLiteral("generate-member"),
                    QStringLiteral("--multisig-id"),
                    multisigId,
                    QStringLiteral("--out"),
                    jsonFile(QStringLiteral("alice.json"))});
    if (alice.exitCode != 0) {
        setStatus(QStringLiteral("generate-member failed"));
        QVariantMap result;
        result.insert(QStringLiteral("ok"), false);
        result.insert(QStringLiteral("commands"), commands);
        return makeJson(result);
    }

    runStep(QStringLiteral("generate_bob"),
            QStringList{QStringLiteral("generate-member"),
                        QStringLiteral("--multisig-id"),
                        multisigId,
                        QStringLiteral("--out"),
                        jsonFile(QStringLiteral("bob.json"))});
    runStep(QStringLiteral("generate_carol"),
            QStringList{QStringLiteral("generate-member"),
                        QStringLiteral("--multisig-id"),
                        multisigId,
                        QStringLiteral("--out"),
                        jsonFile(QStringLiteral("carol.json"))});

    const CommandResult config = runStep(
        QStringLiteral("create_config"),
        QStringList{QStringLiteral("create-config"),
                    QStringLiteral("--multisig-id"),
                    multisigId,
                    QStringLiteral("--threshold"),
                    QString::number(threshold),
                    QStringLiteral("--member"),
                    jsonFile(QStringLiteral("alice.json")),
                    QStringLiteral("--member"),
                    jsonFile(QStringLiteral("bob.json")),
                    QStringLiteral("--member"),
                    jsonFile(QStringLiteral("carol.json")),
                    QStringLiteral("--out"),
                    jsonFile(QStringLiteral("config.json"))});
    if (config.exitCode != 0) {
        setStatus(QStringLiteral("create-config failed"));
        QVariantMap result;
        result.insert(QStringLiteral("ok"), false);
        result.insert(QStringLiteral("commands"), commands);
        return makeJson(result);
    }

    const CommandResult proposal = runStep(
        QStringLiteral("create_proposal"),
        QStringList{QStringLiteral("create-proposal"),
                    QStringLiteral("--multisig-id"),
                    multisigId,
                    QStringLiteral("--proposal-id"),
                    pid,
                    QStringLiteral("--target-program-id"),
                    programId,
                    QStringLiteral("--instruction-words"),
                    proposalWords,
                    QStringLiteral("--target-account-count"),
                    QString::number(targetAccountCount),
                    QStringLiteral("--out"),
                    jsonFile(QStringLiteral("proposal.json"))});
    if (proposal.exitCode != 0) {
        setStatus(QStringLiteral("create-proposal failed"));
        QVariantMap result;
        result.insert(QStringLiteral("ok"), false);
        result.insert(QStringLiteral("commands"), commands);
        return makeJson(result);
    }

    runStep(QStringLiteral("approve_alice"),
            QStringList{QStringLiteral("approve"),
                        QStringLiteral("--member"),
                        jsonFile(QStringLiteral("alice.json")),
                        QStringLiteral("--proposal"),
                        jsonFile(QStringLiteral("proposal.json")),
                        QStringLiteral("--out"),
                        jsonFile(QStringLiteral("approval-alice.json"))});
    runStep(QStringLiteral("approve_bob"),
            QStringList{QStringLiteral("approve"),
                        QStringLiteral("--member"),
                        jsonFile(QStringLiteral("bob.json")),
                        QStringLiteral("--proposal"),
                        jsonFile(QStringLiteral("proposal.json")),
                        QStringLiteral("--out"),
                        jsonFile(QStringLiteral("approval-bob.json"))});

    const CommandResult duplicate = runStep(
        QStringLiteral("duplicate_alice"),
        QStringList{QStringLiteral("aggregate"),
                    QStringLiteral("--config"),
                    jsonFile(QStringLiteral("config.json")),
                    QStringLiteral("--proposal"),
                    jsonFile(QStringLiteral("proposal.json")),
                    QStringLiteral("--member"),
                    jsonFile(QStringLiteral("alice.json")),
                    QStringLiteral("--member"),
                    jsonFile(QStringLiteral("bob.json")),
                    QStringLiteral("--member"),
                    jsonFile(QStringLiteral("carol.json")),
                    QStringLiteral("--approval"),
                    jsonFile(QStringLiteral("approval-alice.json")),
                    QStringLiteral("--approval"),
                    jsonFile(QStringLiteral("approval-alice.json")),
                    QStringLiteral("--out"),
                    jsonFile(QStringLiteral("duplicate-aggregate.json"))});

    const CommandResult aggregate = runStep(
        QStringLiteral("aggregate"),
        QStringList{QStringLiteral("aggregate"),
                    QStringLiteral("--config"),
                    jsonFile(QStringLiteral("config.json")),
                    QStringLiteral("--proposal"),
                    jsonFile(QStringLiteral("proposal.json")),
                    QStringLiteral("--member"),
                    jsonFile(QStringLiteral("alice.json")),
                    QStringLiteral("--member"),
                    jsonFile(QStringLiteral("bob.json")),
                    QStringLiteral("--member"),
                    jsonFile(QStringLiteral("carol.json")),
                    QStringLiteral("--approval"),
                    jsonFile(QStringLiteral("approval-alice.json")),
                    QStringLiteral("--approval"),
                    jsonFile(QStringLiteral("approval-bob.json")),
                    QStringLiteral("--out"),
                    jsonFile(QStringLiteral("aggregate.json"))});

    CommandResult verify;
    if (aggregate.exitCode == 0) {
        verify = runStep(QStringLiteral("verify"),
                         QStringList{QStringLiteral("verify"),
                                     QStringLiteral("--config"),
                                     jsonFile(QStringLiteral("config.json")),
                                     QStringLiteral("--proposal"),
                                     jsonFile(QStringLiteral("proposal.json")),
                                     QStringLiteral("--aggregate"),
                                     jsonFile(QStringLiteral("aggregate.json"))});
    }

    const bool ok = aggregate.exitCode == 0 && verify.exitCode == 0 && duplicate.exitCode != 0;
    setStatus(ok ? QStringLiteral("Demo flow verified") : QStringLiteral("Demo flow failed"));

    QVariantMap artifacts;
    artifacts.insert(QStringLiteral("config"), readTextFile(jsonFile(QStringLiteral("config.json"))));
    artifacts.insert(QStringLiteral("proposal"), readTextFile(jsonFile(QStringLiteral("proposal.json"))));
    artifacts.insert(QStringLiteral("approval_alice"), readTextFile(jsonFile(QStringLiteral("approval-alice.json"))));
    artifacts.insert(QStringLiteral("approval_bob"), readTextFile(jsonFile(QStringLiteral("approval-bob.json"))));
    artifacts.insert(QStringLiteral("aggregate"), readTextFile(jsonFile(QStringLiteral("aggregate.json"))));

    QVariantMap result;
    result.insert(QStringLiteral("ok"), ok);
    result.insert(QStringLiteral("cli_path"), cli);
    result.insert(QStringLiteral("workspace_path"), runRootPath());
    result.insert(QStringLiteral("threshold"), threshold);
    result.insert(QStringLiteral("duplicate_rejected"), duplicate.exitCode != 0);
    result.insert(QStringLiteral("commands"), commands);
    result.insert(QStringLiteral("artifacts"), artifacts);
    return makeJson(result);
}

QString PrivateMultisigUiPlugin::resetWorkspace()
{
    const QString root = runRootPath();
    QDir dir(root);
    bool removed = true;
    if (dir.exists()) {
        removed = dir.removeRecursively();
    }
    QString error;
    const bool created = ensureWorkspace(&error);
    QVariantMap result;
    result.insert(QStringLiteral("ok"), removed && created);
    result.insert(QStringLiteral("workspace_path"), root);
    if (!error.isEmpty()) {
        result.insert(QStringLiteral("error"), error);
    }
    setStatus(result.value(QStringLiteral("ok")).toBool() ? QStringLiteral("Workspace reset")
                                                          : QStringLiteral("Reset failed"));
    return makeJson(result);
}

QString PrivateMultisigUiPlugin::resolveCliPath() const
{
    const QString envPath = QString::fromLocal8Bit(qgetenv("PRIVATE_MULTISIG_CLI"));
    if (!envPath.isEmpty() && QFileInfo::exists(envPath)) {
        return QFileInfo(envPath).absoluteFilePath();
    }

    const QString appDir = QCoreApplication::applicationDirPath();
    const QStringList candidates = {
        QDir(appDir).filePath(QStringLiteral("private_multisig_cli")),
        QDir::current().filePath(QStringLiteral("target/debug/private_multisig_cli")),
        QDir::current().filePath(QStringLiteral("../target/debug/private_multisig_cli")),
        QDir::home().filePath(QStringLiteral("Projects/logos/lp-0002-private-multisig/target/debug/private_multisig_cli")),
    };

    for (const QString& candidate : candidates) {
        if (QFileInfo::exists(candidate)) {
            return QFileInfo(candidate).absoluteFilePath();
        }
    }

    return envPath.isEmpty() ? QStringLiteral("private_multisig_cli") : envPath;
}

QString PrivateMultisigUiPlugin::resolveRunnerPath() const
{
    const QString envPath = QString::fromLocal8Bit(qgetenv("PRIVATE_MULTISIG_RUNNER"));
    if (!envPath.isEmpty() && QFileInfo::exists(envPath)) {
        return QFileInfo(envPath).absoluteFilePath();
    }

    const QString appDir = QCoreApplication::applicationDirPath();
    const QStringList candidates = {
        QDir(appDir).filePath(QStringLiteral("private_multisig_runner")),
        QDir::current().filePath(QStringLiteral("target/debug/private_multisig_runner")),
        QDir::current().filePath(QStringLiteral("../target/debug/private_multisig_runner")),
        QDir::home().filePath(QStringLiteral("Projects/logos/lp-0002-private-multisig/target/debug/private_multisig_runner")),
    };

    for (const QString& candidate : candidates) {
        if (QFileInfo::exists(candidate)) {
            return QFileInfo(candidate).absoluteFilePath();
        }
    }

    return envPath.isEmpty() ? QStringLiteral("private_multisig_runner") : envPath;
}

QString PrivateMultisigUiPlugin::resolveRepoRoot() const
{
    const QString envPath = QString::fromLocal8Bit(qgetenv("PRIVATE_MULTISIG_REPO_ROOT"));
    if (!envPath.isEmpty() && QFileInfo(envPath).isDir()) {
        return QFileInfo(envPath).absoluteFilePath();
    }

    QFileInfo cliInfo(resolveCliPath());
    if (cliInfo.exists()) {
        const QString candidate = QDir(cliInfo.absolutePath()).absoluteFilePath(QStringLiteral("../.."));
        if (QFileInfo(candidate).isDir()) {
            return QDir(candidate).absolutePath();
        }
    }

    return QDir::current().absolutePath();
}

QString PrivateMultisigUiPlugin::resolveTargetProgramBinaryPath() const
{
    const QString envPath = QString::fromLocal8Bit(qgetenv("PRIVATE_MULTISIG_TARGET_PROGRAM_BINARY"));
    if (!envPath.isEmpty() && QFileInfo::exists(envPath)) {
        return QFileInfo(envPath).absoluteFilePath();
    }

    const QString repoRoot = resolveRepoRoot();
    const QStringList candidates = {
        QDir::home().filePath(QStringLiteral("Projects/logos/logos-execution-zone-v0.2.0-testnet/target/riscv32im-risc0-zkvm-elf/docker/hello_world.bin")),
        QDir(repoRoot).absoluteFilePath(QStringLiteral("../logos-execution-zone-v0.2.0-testnet/target/riscv32im-risc0-zkvm-elf/docker/hello_world.bin")),
    };
    for (const QString& candidate : candidates) {
        if (QFileInfo::exists(candidate)) {
            return QFileInfo(candidate).absoluteFilePath();
        }
    }
    return candidates.isEmpty() ? QString() : candidates.first();
}

QString PrivateMultisigUiPlugin::defaultWorkspacePath() const
{
    const QString envPath = QString::fromLocal8Bit(qgetenv("PRIVATE_MULTISIG_UI_WORKSPACE"));
    if (!envPath.isEmpty()) {
        return QFileInfo(envPath).absoluteFilePath();
    }
    const QString base = QStandardPaths::writableLocation(QStandardPaths::AppDataLocation);
    if (!base.isEmpty()) {
        return QDir(base).filePath(QStringLiteral("private-multisig-ui"));
    }
    return QDir::home().filePath(QStringLiteral(".local/share/private-multisig-ui"));
}

QString PrivateMultisigUiPlugin::runRootPath() const
{
    return QDir(workspacePath()).filePath(QStringLiteral("latest"));
}

QString PrivateMultisigUiPlugin::absolutePath(const QString& relative) const
{
    return QDir(runRootPath()).filePath(relative);
}

QString PrivateMultisigUiPlugin::jsonFile(const QString& relative) const
{
    return absolutePath(relative);
}

QString PrivateMultisigUiPlugin::memberKeyPath(const QString& memberId) const
{
    const QString key = memberId.trimmed().replace(QStringLiteral("member-"), QString());
    return jsonFile(QStringLiteral("%1.json").arg(key));
}

QString PrivateMultisigUiPlugin::approvalKeyPath(const QString& memberId) const
{
    const QString key = memberId.trimmed().replace(QStringLiteral("member-"), QString());
    return jsonFile(QStringLiteral("approval-%1.json").arg(key));
}

void PrivateMultisigUiPlugin::removeArtifacts(const QStringList& relativePaths) const
{
    for (const QString& relativePath : relativePaths) {
        const QString path = absolutePath(relativePath);
        QFileInfo info(path);
        if (!info.exists()) {
            continue;
        }
        if (info.isDir()) {
            QDir(path).removeRecursively();
            continue;
        }
        QFile::remove(path);
    }
}

QString PrivateMultisigUiPlugin::readTextFile(const QString& path) const
{
    QFile file(path);
    if (!file.open(QIODevice::ReadOnly | QIODevice::Text)) {
        return QString();
    }
    return QString::fromUtf8(file.readAll());
}

bool PrivateMultisigUiPlugin::writeTextFile(const QString& path, const QString& text, QString* error) const
{
    QFileInfo info(path);
    QDir().mkpath(info.absolutePath());
    QFile file(path);
    if (!file.open(QIODevice::WriteOnly | QIODevice::Text | QIODevice::Truncate)) {
        if (error) {
            *error = file.errorString();
        }
        return false;
    }
    file.write(text.toUtf8());
    return true;
}

bool PrivateMultisigUiPlugin::ensureWorkspace(QString* error) const
{
    QDir dir(runRootPath());
    if (!dir.exists() && !dir.mkpath(QStringLiteral("."))) {
        if (error) {
            *error = QStringLiteral("failed to create workspace %1").arg(runRootPath());
        }
        return false;
    }
    QString markerError;
    const bool markerOk = writeTextFile(
        QDir(runRootPath()).filePath(QStringLiteral("backend-marker.txt")),
        QStringLiteral("private_multisig_ui backend workspace\n%1\n").arg(QDateTime::currentDateTimeUtc().toString(Qt::ISODate)),
        &markerError);
    if (!markerOk && error) {
        *error = markerError;
    }
    return markerOk;
}

QProcessEnvironment PrivateMultisigUiPlugin::sanitizedScriptEnvironment(
    const QVariantMap& environmentOverrides) const
{
    const QProcessEnvironment source = QProcessEnvironment::systemEnvironment();
    QProcessEnvironment env;

    const QStringList passthroughKeys = {
        QStringLiteral("HOME"),
        QStringLiteral("USER"),
        QStringLiteral("USERNAME"),
        QStringLiteral("LOGNAME"),
        QStringLiteral("SHELL"),
        QStringLiteral("LANG"),
        QStringLiteral("LC_ALL"),
        QStringLiteral("LC_CTYPE"),
        QStringLiteral("TERM"),
        QStringLiteral("TMPDIR"),
        QStringLiteral("XDG_RUNTIME_DIR"),
        QStringLiteral("NIX_PROFILES"),
        QStringLiteral("NIX_SSL_CERT_FILE"),
        QStringLiteral("CARGO_HOME"),
        QStringLiteral("RUSTUP_HOME"),
    };

    for (const QString& key : passthroughKeys) {
        if (source.contains(key)) {
            env.insert(key, source.value(key));
        }
    }

    QStringList cleanPathEntries;
    const QString rawPath = source.value(QStringLiteral("PATH"));
    for (const QString& entry : rawPath.split(QLatin1Char(':'), Qt::SkipEmptyParts)) {
        if (entry.contains(QStringLiteral(".mount_Logos"))) {
            continue;
        }
        cleanPathEntries.append(entry);
    }
    if (cleanPathEntries.isEmpty()) {
        cleanPathEntries = {
            QStringLiteral("/home/agate/.nix-profile/bin"),
            QStringLiteral("/home/agate/.cargo/bin"),
            QStringLiteral("/home/agate/.local/bin"),
            QStringLiteral("/usr/local/sbin"),
            QStringLiteral("/usr/local/bin"),
            QStringLiteral("/usr/sbin"),
            QStringLiteral("/usr/bin"),
            QStringLiteral("/sbin"),
            QStringLiteral("/bin"),
        };
    }
    env.insert(QStringLiteral("PATH"), cleanPathEntries.join(QLatin1Char(':')));

    env.insert(QStringLiteral("PRIVATE_MULTISIG_CLI"), resolveCliPath());
    env.insert(QStringLiteral("PRIVATE_MULTISIG_RUNNER"), resolveRunnerPath());
    env.insert(QStringLiteral("TARGET_PROGRAM_BINARY"), resolveTargetProgramBinaryPath());
    for (auto it = environmentOverrides.constBegin(); it != environmentOverrides.constEnd(); ++it) {
        env.insert(it.key(), it.value().toString());
    }
    return env;
}

PrivateMultisigUiPlugin::CommandResult PrivateMultisigUiPlugin::runProcess(
    const QString& program,
    const QStringList& args,
    const QString& workingDirectory,
    const QProcessEnvironment& environment,
    int timeoutMs) const
{
    CommandResult result;
    QProcess process;
    process.setProgram(program);
    process.setArguments(args);
    process.setWorkingDirectory(workingDirectory);
    process.setProcessEnvironment(environment);
    process.setProcessChannelMode(QProcess::SeparateChannels);
    process.start();
    if (!process.waitForStarted(5000)) {
        result.error = process.errorString();
        return result;
    }
    if (!process.waitForFinished(timeoutMs)) {
        result.timedOut = true;
        process.kill();
        process.waitForFinished(3000);
    }
    result.exitCode = process.exitCode();
    result.stdoutText = QString::fromUtf8(process.readAllStandardOutput());
    result.stderrText = QString::fromUtf8(process.readAllStandardError());
    if (process.error() != QProcess::UnknownError) {
        result.error = process.errorString();
    }
    return result;
}

PrivateMultisigUiPlugin::CommandResult PrivateMultisigUiPlugin::runCli(const QStringList& args,
                                                                       int timeoutMs) const
{
    return runProcess(resolveCliPath(),
                      args,
                      runRootPath(),
                      QProcessEnvironment::systemEnvironment(),
                      timeoutMs);
}

PrivateMultisigUiPlugin::CommandResult PrivateMultisigUiPlugin::runRepoScript(
    const QString& scriptName,
    const QStringList& scriptArgs,
    const QVariantMap& environmentOverrides,
    int timeoutMs) const
{
    const QProcessEnvironment env = sanitizedScriptEnvironment(environmentOverrides);

    const QString repoRoot = resolveRepoRoot();
    const QString scriptPath = QDir(repoRoot).absoluteFilePath(QStringLiteral("scripts/%1").arg(scriptName));
    QStringList args{QStringLiteral("-lc"),
                     QStringLiteral("cd \"%1\" && bash \"%2\" %3")
                         .arg(repoRoot,
                              scriptPath,
                              scriptArgs.join(QLatin1Char(' ')))};
    return runProcess(QStringLiteral("bash"), args, repoRoot, env, timeoutMs);
}

QString PrivateMultisigUiPlugin::makeJson(const QVariantMap& map) const
{
    return QString::fromUtf8(QJsonDocument::fromVariant(map).toJson(QJsonDocument::Indented));
}

QVariantMap PrivateMultisigUiPlugin::commandMap(const QString& step, const CommandResult& result) const
{
    QVariantMap map;
    map.insert(QStringLiteral("step"), step);
    map.insert(QStringLiteral("exit_code"), result.exitCode);
    map.insert(QStringLiteral("stdout"), result.stdoutText.trimmed());
    map.insert(QStringLiteral("stderr"), result.stderrText.trimmed());
    map.insert(QStringLiteral("error"), result.error);
    map.insert(QStringLiteral("timed_out"), result.timedOut);
    map.insert(QStringLiteral("ok"), result.exitCode == 0 && !result.timedOut);
    return map;
}

QVariantMap PrivateMultisigUiPlugin::artifactsMap() const
{
    const struct ArtifactSpec {
        const char* key;
        const char* relativePath;
    } artifacts[] = {
        {"alice", "alice.json"},
        {"bob", "bob.json"},
        {"carol", "carol.json"},
        {"config", "config.json"},
        {"proposal", "proposal.json"},
        {"approval_alice", "approval-alice.json"},
        {"approval_bob", "approval-bob.json"},
        {"approval_carol", "approval-carol.json"},
        {"aggregate", "aggregate.json"},
        {"duplicate_aggregate", "duplicate-aggregate.json"},
        {"prove_journal", "proof/journal.json"},
        {"prove_receipt", "proof/receipt.json"},
        {"prove_summary", "proof/summary.json"},
        {"localnet_evidence", "onchain/localnet/localnet-evidence.json"},
        {"testnet_evidence", "onchain/testnet/testnet-evidence.json"},
    };

    QVariantMap map;
    for (const ArtifactSpec& artifact : artifacts) {
        const QString path = absolutePath(QString::fromUtf8(artifact.relativePath));
        if (QFileInfo::exists(path)) {
            map.insert(QString::fromUtf8(artifact.key), readTextFile(path));
        }
    }
    return map;
}
