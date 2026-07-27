#pragma once

#include <QProcessEnvironment>
#include <QString>
#include <QVariantList>

#include "LogosViewPluginBase.h"
#include "module_config.h"
#include "private_multisig_ui_interface.h"
#include "rep_private_multisig_ui_source.h"

class LogosAPI;

class PrivateMultisigUiPlugin : public PrivateMultisigUiSimpleSource,
                                public PrivateMultisigUiInterface,
                                public PrivateMultisigUiViewPluginBase
{
    Q_OBJECT
    Q_PLUGIN_METADATA(IID PrivateMultisigUiInterface_iid FILE "metadata.json")
    Q_INTERFACES(PrivateMultisigUiInterface)

public:
    explicit PrivateMultisigUiPlugin(QObject* parent = nullptr);
    ~PrivateMultisigUiPlugin() override;

    QString name() const override { return MODULE_NAME; }
    QString version() const override { return MODULE_VERSION; }

    Q_INVOKABLE void initLogos(LogosAPI* api);

    QString health() override;
    QString loadWorkspaceState() override;
    QString generateMembers(QString multisigId) override;
    QString createConfig(QString multisigId, int threshold) override;
    QString createProposal(QString multisigId,
                           QString proposalId,
                           QString targetProgramId,
                           QString instructionWords,
                           int targetAccountCount) override;
    QString approveMember(QString memberId) override;
    QString aggregateApprovals() override;
    QString verifyAggregate() override;
    QString proveAggregate() override;
    QString runLocalnetExecution() override;
    QString runHostedTestnetExecution() override;
    QString testDuplicateAggregate(QString memberId) override;
    QString runDemoFlow(int threshold,
                        QString proposalId,
                        QString targetProgramId,
                        QString instructionWords,
                        int targetAccountCount) override;
    QString resetWorkspace() override;

signals:
    void eventResponse(const QString& eventName, const QVariantList& args);

private:
    struct CommandResult {
        int exitCode = -1;
        QString stdoutText;
        QString stderrText;
        QString error;
        bool timedOut = false;
    };

    LogosAPI* m_logosAPI = nullptr;

    QString resolveCliPath() const;
    QString resolveRunnerPath() const;
    QString resolveRepoRoot() const;
    QString resolveTargetProgramBinaryPath() const;
    QString defaultWorkspacePath() const;
    QString runRootPath() const;
    QString absolutePath(const QString& relative) const;
    QString jsonFile(const QString& relative) const;
    QString memberKeyPath(const QString& memberId) const;
    QString approvalKeyPath(const QString& memberId) const;
    void removeArtifacts(const QStringList& relativePaths) const;
    QString readTextFile(const QString& path) const;
    bool writeTextFile(const QString& path, const QString& text, QString* error) const;
    bool ensureWorkspace(QString* error) const;
    QProcessEnvironment sanitizedScriptEnvironment(const QVariantMap& environmentOverrides) const;
    CommandResult runProcess(const QString& program,
                             const QStringList& args,
                             const QString& workingDirectory,
                             const QProcessEnvironment& environment,
                             int timeoutMs) const;
    CommandResult runCli(const QStringList& args, int timeoutMs = 30000) const;
    CommandResult runRepoScript(const QString& scriptName,
                                const QStringList& scriptArgs,
                                const QVariantMap& environmentOverrides,
                                int timeoutMs) const;
    QString makeJson(const QVariantMap& map) const;
    QVariantMap commandMap(const QString& step, const CommandResult& result) const;
    QVariantMap artifactsMap() const;
};
