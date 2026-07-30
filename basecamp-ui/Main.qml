import QtQuick 2.15
import QtQuick.Controls 2.15
import QtQuick.Layouts 1.15

Item {
    id: root
    objectName: "PrivateMultisigBasecampUi"
    width: 1180
    height: 760

    readonly property color bg0: "#0d1117"
    readonly property color bg1: "#131922"
    readonly property color bg2: "#18202a"
    readonly property color panel: "#151c24"
    readonly property color panelRaised: "#1a2330"
    readonly property color border: "#2a3441"
    readonly property color borderStrong: "#3a4b5d"
    readonly property color textPrimary: "#f3f6fb"
    readonly property color textSecondary: "#9aa9ba"
    readonly property color accent: "#5ab0ff"
    readonly property color accentStrong: "#2f89e1"
    readonly property color ok: "#3ecf8e"
    readonly property color okMuted: "#16392e"
    readonly property color warn: "#d5ad45"
    readonly property color warnMuted: "#342b16"

    property string multisigId: ""
    property int threshold: 2
    property int targetAccountCount: 1
    property string activeMemberId: "member-alice"
    property bool proposalCreated: false
    property bool aggregateReady: false
    property bool duplicateRejected: false
    property string proposalId: "1"
    property string targetProgram: "1,2,3,4,5,6,7,8"
    property string instruction: "9,10"
    property string memberRoot: ""
    property var members: []
    property var approvals: []
    property var logLines: []
    property string evidenceJson: "{}"
    readonly property var backend: logos && logos.module ? logos.module("private_multisig_ui") : null
    property bool backendReady: false
    readonly property bool backendConnected: backend !== null && backendReady
    property string backendStatus: "Connecting to backend"
    property string backendHealthJson: "{}"
    property string backendResultJson: "{}"
    property bool backendRunning: false

    function fakeHash(seed) {
        var h1 = 0x811c9dc5
        var h2 = 0x45d9f3b
        var text = String(seed)
        for (var i = 0; i < text.length; i++) {
            h1 = (h1 ^ text.charCodeAt(i)) >>> 0
            h1 = ((h1 * 16777619) >>> 0)
            h2 = (h2 + ((text.charCodeAt(i) + i + 97) * 2654435761)) >>> 0
        }
        function part(v) {
            var out = (v >>> 0).toString(16)
            while (out.length < 8)
                out = "0" + out
            return out
        }
        var hex = part(h1) + part(h2) + part((h1 ^ h2) >>> 0) + part((h1 + h2) >>> 0)
        return hex + hex
    }

    function shortHex(value) {
        var text = String(value)
        if (text.length <= 18)
            return text
        return text.slice(0, 10) + "..." + text.slice(text.length - 6)
    }

    function freshMultisigId() {
        return fakeHash("multisig:" + Date.now() + ":" + Math.random())
    }

    function assignFreshMultisigId() {
        multisigId = freshMultisigId()
        if (multisigIdField)
            multisigIdField.text = multisigId
    }

    function pushLog(line) {
        var copy = logLines.slice()
        var now = new Date().toISOString().replace("T", " ").replace("Z", "Z")
        copy.unshift(now + "  " + line)
        if (copy.length > 10)
            copy = copy.slice(0, 10)
        logLines = copy
    }

    function defaultMembers() {
        return [
            { "id": "member-alice", "key": "alice", "label": "Alice", "commitment": "", "npk": "" },
            { "id": "member-bob", "key": "bob", "label": "Bob", "commitment": "", "npk": "" },
            { "id": "member-carol", "key": "carol", "label": "Carol", "commitment": "", "npk": "" }
        ]
    }

    function selectedMember() {
        for (var i = 0; i < members.length; i++) {
            if (members[i].id === activeMemberId)
                return members[i]
        }
        return members.length > 0 ? members[0] : null
    }

    function approvalStoredFor(memberId) {
        for (var i = 0; i < approvals.length; i++) {
            if (approvals[i].member_id === memberId)
                return true
        }
        return false
    }

    function approvalLabelFor(memberId) {
        return approvalStoredFor(memberId) ? "approval stored" : "waiting"
    }

    function syncFromArtifacts(artifacts) {
        var nextMembers = defaultMembers()
        var nextApprovals = []
        var config = null
        var proposal = null
        var aggregate = null

        if (artifacts && artifacts.config)
            config = parseJsonOrNull(artifacts.config)
        if (artifacts && artifacts.proposal)
            proposal = parseJsonOrNull(artifacts.proposal)
        if (artifacts && artifacts.aggregate)
            aggregate = parseJsonOrNull(artifacts.aggregate)

        if (config) {
            if (config.threshold !== undefined)
                threshold = Number(config.threshold)
            if (config.multisig_id)
                multisigId = String(config.multisig_id)
            memberRoot = config.member_root ? String(config.member_root) : ""
        } else {
            memberRoot = ""
        }

        if (proposal) {
            proposalCreated = true
            proposalId = String(proposal.proposal_id)
            targetProgram = proposal.target_program_id ? proposal.target_program_id.join(",") : targetProgram
            instruction = proposal.target_instruction_data ? proposal.target_instruction_data.join(",") : ""
            if (proposal.target_account_count !== undefined)
                targetAccountCount = Number(proposal.target_account_count)
        } else {
            proposalCreated = false
        }

        for (var i = 0; i < nextMembers.length; i++) {
            var memberArtifact = artifacts ? artifacts[nextMembers[i].key] : null
            var memberData = memberArtifact ? parseJsonOrNull(memberArtifact) : null
            if (memberData) {
                nextMembers[i].commitment = memberData.leaf ? String(memberData.leaf) : ""
                nextMembers[i].npk = memberData.npk ? String(memberData.npk) : ""
                if (memberData.multisig_id)
                    multisigId = String(memberData.multisig_id)
            }

            var approvalArtifact = artifacts ? artifacts["approval_" + nextMembers[i].key] : null
            var approvalData = approvalArtifact ? parseJsonOrNull(approvalArtifact) : null
            if (approvalData) {
                nextApprovals.push({
                    "member_id": nextMembers[i].id,
                    "member_label": nextMembers[i].label,
                    "commitment": approvalData.member_leaf ? String(approvalData.member_leaf) : "",
                    "nullifier": approvalData.nullifier ? String(approvalData.nullifier) : "",
                    "approval_hash": ""
                })
            }
        }

        members = nextMembers
        approvals = nextApprovals
        aggregateReady = !!aggregate
        if (aggregate && aggregate.approval_count !== undefined)
            aggregateReady = Number(aggregate.approval_count) >= threshold
        if (members.length > 0 && !selectedMember())
            activeMemberId = members[0].id
    }

    function handleBackendAction(reply, successLog, failureLog) {
        backendRunning = true
        var done = function(value) {
            backendRunning = false
            var text = payloadToText(value)
            evidenceJson = text
            backendResultJson = text
            var parsed = parseJsonOrNull(text)
            if (parsed && parsed.artifacts)
                syncFromArtifacts(parsed.artifacts)
            duplicateRejected = parsed && parsed.duplicate_rejected === true
            backendStatus = parsed && parsed.ok ? "Action complete" : "Action failed"
            pushLog(parsed && parsed.ok ? successLog : failureLog)
        }
        var fail = function(error) {
            backendRunning = false
            backendStatus = "Backend action error"
            backendResultJson = JSON.stringify({"ok": false, "error": String(error)}, null, 2)
            evidenceJson = backendResultJson
            pushLog(failureLog + ": " + String(error))
        }
        watchBackend(reply, done, fail)
    }

    function generateMembers() {
        if (!updateBackendReady() || !backend || !backend.generateMembers) {
            backendStatus = "Backend unavailable"
            pushLog("Generate members unavailable")
            return
        }
        backendStatus = "Generating members"
        handleBackendAction(backend.generateMembers(multisigId), "Members generated", "Member generation failed")
    }

    function createBackendConfig() {
        if (!updateBackendReady() || !backend || !backend.createConfig) {
            backendStatus = "Backend unavailable"
            pushLog("Create config unavailable")
            return
        }
        backendStatus = "Creating config"
        handleBackendAction(backend.createConfig(multisigId, threshold), "Config created", "Config failed")
    }

    function createProposal() {
        if (!updateBackendReady() || !backend || !backend.createProposal) {
            backendStatus = "Backend unavailable"
            pushLog("Create proposal unavailable")
            return
        }
        backendStatus = "Creating proposal"
        handleBackendAction(
                    backend.createProposal(multisigId, proposalId, targetProgram, instruction, targetAccountCount),
                    "Proposal created",
                    "Proposal failed")
    }

    function approveSelectedMember() {
        var member = selectedMember()
        if (member === null) {
            pushLog("No member selected")
            return
        }
        if (!updateBackendReady() || !backend || !backend.approveMember) {
            backendStatus = "Backend unavailable"
            pushLog("Approve member unavailable")
            return
        }
        backendStatus = "Approving " + member.label
        handleBackendAction(backend.approveMember(member.id), "Approval stored for " + member.label, "Approval failed for " + member.label)
    }

    function aggregateBackendApprovals() {
        if (!updateBackendReady() || !backend || !backend.aggregateApprovals) {
            backendStatus = "Backend unavailable"
            pushLog("Aggregate unavailable")
            return
        }
        backendStatus = "Aggregating approvals"
        handleBackendAction(backend.aggregateApprovals(), "Aggregate created", "Aggregate failed")
    }

    function verifyBackendAggregate() {
        if (!updateBackendReady() || !backend || !backend.verifyAggregate) {
            backendStatus = "Backend unavailable"
            pushLog("Verify unavailable")
            return
        }
        backendStatus = "Verifying aggregate"
        handleBackendAction(backend.verifyAggregate(), "Aggregate verified", "Verify failed")
    }

    function proveBackendAggregate() {
        if (!updateBackendReady() || !backend || !backend.proveAggregate) {
            backendStatus = "Backend unavailable"
            pushLog("Prove unavailable")
            return
        }
        backendStatus = "Generating proof"
        handleBackendAction(backend.proveAggregate(), "Proof generated", "Proof generation failed")
    }

    function executeLocalnetBackend() {
        if (!updateBackendReady() || !backend || !backend.runLocalnetExecution) {
            backendStatus = "Backend unavailable"
            pushLog("Localnet execution unavailable")
            return
        }
        backendStatus = "Running localnet execution"
        handleBackendAction(backend.runLocalnetExecution(), "Localnet execution complete", "Localnet execution failed")
    }

    function executeHostedTestnetBackend() {
        if (!updateBackendReady() || !backend || !backend.runHostedTestnetExecution) {
            backendStatus = "Backend unavailable"
            pushLog("Hosted testnet execution unavailable")
            return
        }
        backendStatus = "Running hosted testnet execution"
        handleBackendAction(backend.runHostedTestnetExecution(), "Hosted testnet execution complete", "Hosted testnet execution failed")
    }

    function testDuplicateSelected() {
        var member = selectedMember()
        if (member === null) {
            pushLog("No member selected")
            return
        }
        if (!updateBackendReady() || !backend || !backend.testDuplicateAggregate) {
            backendStatus = "Backend unavailable"
            pushLog("Duplicate test unavailable")
            return
        }
        backendStatus = "Testing duplicate rejection"
        handleBackendAction(backend.testDuplicateAggregate(member.id), "Duplicate approval rejected", "Duplicate test failed")
    }

    function resetWorkflow() {
        members = defaultMembers()
        approvals = []
        proposalCreated = false
        aggregateReady = false
        duplicateRejected = false
        memberRoot = ""
        evidenceJson = "{}"
        assignFreshMultisigId()
        pushLog("Reset local view")
    }

    function parseJsonOrNull(value) {
        try {
            return JSON.parse(String(value))
        } catch (err) {
            return null
        }
    }

    function payloadToText(value) {
        if (value === undefined || value === null)
            return ""
        if (typeof value === "string")
            return value
        try {
            return JSON.stringify(value, null, 2)
        } catch (err) {
            return String(value)
        }
    }

    function updateBackendReady() {
        var isReady = false
        if (backend) {
            if (logos && logos.isViewModuleReady)
                isReady = logos.isViewModuleReady("private_multisig_ui")
            else
                isReady = true
        }
        backendReady = isReady
        if (!backendReady && !backendRunning)
            backendStatus = backend ? "Connecting to backend" : "Backend unavailable"
        return backendReady
    }

    function watchBackend(reply, onSuccess, onFailure) {
        if (logos && logos.watch) {
            logos.watch(reply, function(value) {
                onSuccess(value)
            }, function(error) {
                onFailure(error)
            })
            return
        }
        onSuccess(reply)
    }

    function refreshBackendHealth() {
        if (!updateBackendReady() || !backend || !backend.health) {
            backendStatus = "Backend unavailable"
            backendHealthJson = JSON.stringify({"ok": false, "error": "backend replica unavailable"}, null, 2)
            evidenceJson = backendHealthJson
            pushLog("Backend health unavailable")
            return
        }
        backendStatus = "Checking backend"
        var reply
        try {
            reply = backend.health()
        } catch (err) {
            backendStatus = "Backend health error"
            backendHealthJson = JSON.stringify({"ok": false, "error": String(err)}, null, 2)
            evidenceJson = backendHealthJson
            pushLog("Backend health error: " + String(err))
            return
        }
        watchBackend(reply, function(value) {
            backendHealthJson = payloadToText(value)
            evidenceJson = backendHealthJson
            var parsed = parseJsonOrNull(backendHealthJson)
            backendStatus = parsed && parsed.ok ? "Backend ready" : "Backend health failed"
            pushLog(parsed && parsed.ok ? "Backend health OK" : "Backend health failed")
        }, function(error) {
            backendStatus = "Backend health error"
            backendHealthJson = JSON.stringify({"ok": false, "error": String(error)}, null, 2)
            evidenceJson = backendHealthJson
            pushLog("Backend health error: " + String(error))
        })
    }

    function refreshWorkspaceState() {
        if (!updateBackendReady() || !backend || !backend.loadWorkspaceState) {
            backendStatus = "Backend unavailable"
            pushLog("Workspace refresh unavailable")
            return
        }
        backendStatus = "Loading workspace"
        var reply
        try {
            reply = backend.loadWorkspaceState()
        } catch (err) {
            backendStatus = "Workspace refresh error"
            evidenceJson = JSON.stringify({"ok": false, "error": String(err)}, null, 2)
            pushLog("Workspace refresh error: " + String(err))
            return
        }
        watchBackend(reply, function(value) {
            var text = payloadToText(value)
            evidenceJson = text
            backendResultJson = text
            var parsed = parseJsonOrNull(text)
            if (parsed && parsed.artifacts)
                syncFromArtifacts(parsed.artifacts)
            backendStatus = parsed && parsed.ok ? "Workspace loaded" : "Workspace load failed"
            pushLog(parsed && parsed.ok ? "Workspace loaded" : "Workspace load failed")
        }, function(error) {
            backendStatus = "Workspace refresh error"
            evidenceJson = JSON.stringify({"ok": false, "error": String(error)}, null, 2)
            pushLog("Workspace refresh error: " + String(error))
        })
    }

    function runBackendFlow() {
        if (!updateBackendReady() || !backend || !backend.runDemoFlow) {
            backendStatus = "Backend unavailable"
            backendResultJson = JSON.stringify({"ok": false, "error": "backend replica unavailable"}, null, 2)
            evidenceJson = backendResultJson
            pushLog("Backend flow unavailable")
            return
        }
        backendRunning = true
        backendStatus = "Running"
        backendResultJson = "{}"
        pushLog("Backend CLI flow started")
        var reply
        try {
            reply = backend.runDemoFlow(threshold, proposalId, targetProgram, instruction, 1)
        } catch (err) {
            backendRunning = false
            backendStatus = "Backend flow error"
            backendResultJson = JSON.stringify({"ok": false, "error": String(err)}, null, 2)
            evidenceJson = backendResultJson
            pushLog("Backend flow error: " + String(err))
            return
        }
        watchBackend(reply, function(value) {
            backendResultJson = payloadToText(value)
            evidenceJson = backendResultJson
            var parsed = parseJsonOrNull(backendResultJson)
            backendStatus = parsed && parsed.ok ? "Demo flow verified" : "Demo flow failed"
            backendRunning = false
            pushLog(parsed && parsed.ok ? "Backend CLI flow verified" : "Backend CLI flow failed")
        }, function(error) {
            backendRunning = false
            backendStatus = "Backend flow error"
            backendResultJson = JSON.stringify({"ok": false, "error": String(error)}, null, 2)
            evidenceJson = backendResultJson
            pushLog("Backend flow error: " + String(error))
        })
    }

    function resetBackendWorkspace() {
        if (!updateBackendReady() || !backend || !backend.resetWorkspace) {
            backendStatus = "Backend unavailable"
            pushLog("Backend reset unavailable")
            return
        }
        backendStatus = "Resetting backend workspace"
        var reply
        try {
            reply = backend.resetWorkspace()
        } catch (err) {
            backendStatus = "Backend reset error"
            backendResultJson = JSON.stringify({"ok": false, "error": String(err)}, null, 2)
            evidenceJson = backendResultJson
            pushLog("Backend reset error: " + String(err))
            return
        }
        watchBackend(reply, function(value) {
            backendResultJson = payloadToText(value)
            evidenceJson = backendResultJson
            var parsed = parseJsonOrNull(backendResultJson)
            if (parsed && parsed.ok)
                resetWorkflow()
            backendStatus = parsed && parsed.ok ? "Workspace reset" : "Backend reset failed"
            pushLog(parsed && parsed.ok ? "Backend workspace reset" : "Backend reset failed")
        }, function(error) {
            backendStatus = "Backend reset error"
            backendResultJson = JSON.stringify({"ok": false, "error": String(error)}, null, 2)
            evidenceJson = backendResultJson
            pushLog("Backend reset error: " + String(error))
        })
    }

    function evidenceObject() {
        var nullifiers = []
        var approvalHashes = []
        for (var i = 0; i < approvals.length; i++) {
            nullifiers.push(approvals[i].nullifier)
            approvalHashes.push(approvals[i].approval_hash)
        }
        return {
            "module": "private_multisig_ui",
            "threshold": threshold,
            "member_count": members.length,
            "member_root": memberRoot,
            "proposal": {
                "id": proposalId,
                "target_program": targetProgram,
                "instruction": instruction,
                "target_account_count": targetAccountCount,
                "proposal_hash": fakeHash("proposal:" + proposalId + ":" + targetProgram + ":" + instruction + ":" + targetAccountCount)
            },
            "approvals": approvals,
            "aggregate": {
                "ready": aggregateReady,
                "approval_count": approvals.length,
                "threshold": threshold,
                "nullifiers": nullifiers,
                "aggregate_hash": fakeHash("aggregate:" + proposalId + ":" + approvalHashes.join(":"))
            },
            "policy": {
                "double_vote_rejected": duplicateRejected,
                "below_threshold_blocked": approvals.length < threshold
            }
        }
    }

    function updateEvidence() {
        evidenceJson = JSON.stringify(evidenceObject(), null, 2)
    }

    Component.onCompleted: {
        members = defaultMembers()
        assignFreshMultisigId()
        if (updateBackendReady()) {
            refreshBackendHealth()
            refreshWorkspaceState()
        }
    }

    Connections {
        target: logos
        function onViewModuleReadyChanged(moduleName, isReady) {
            if (moduleName !== "private_multisig_ui")
                return
            backendReady = isReady && backend !== null
            backendStatus = backendReady ? "Backend connected" : "Connecting to backend"
            pushLog(backendReady ? "Backend connected" : "Backend disconnected")
            if (backendReady)
                refreshBackendHealth()
        }
    }

    Timer {
        id: backendReadyTimer
        interval: 500
        repeat: true
        running: !backendReady
        onTriggered: {
            if (updateBackendReady()) {
                backendStatus = "Backend connected"
                refreshBackendHealth()
                refreshWorkspaceState()
            }
        }
    }

    Rectangle {
        anchors.fill: parent
        gradient: Gradient {
            GradientStop { position: 0.0; color: bg0 }
            GradientStop { position: 1.0; color: bg1 }
        }
    }

    ColumnLayout {
        anchors.fill: parent
        anchors.margins: 24
        spacing: 18

        RowLayout {
            Layout.fillWidth: true
            spacing: 16

            ColumnLayout {
                Layout.fillWidth: true
                spacing: 6

                Text {
                    text: "Private Multisig"
                    color: textPrimary
                    font.pixelSize: 30
                    font.bold: true
                }

                Text {
                    text: "2-of-3 shielded approvals with proposal nullifiers"
                    color: textSecondary
                    font.pixelSize: 14
                }
            }

            Rectangle {
                width: 186
                height: 44
                radius: 10
                color: aggregateReady ? okMuted : warnMuted
                border.color: aggregateReady ? ok : warn
                border.width: 1

                Text {
                    anchors.centerIn: parent
                    text: aggregateReady ? "Ready to execute" : approvals.length + "/" + threshold + " approvals"
                    color: aggregateReady ? "#b9f7d7" : "#f2d68b"
                    font.pixelSize: 14
                    font.bold: true
                }
            }
        }

        Rectangle {
            Layout.fillWidth: true
            height: 84
            radius: 12
            color: panel
            border.color: border
            border.width: 1

            RowLayout {
                anchors.fill: parent
                anchors.margins: 14
                spacing: 12

                ColumnLayout {
                    Layout.fillWidth: true
                    spacing: 4

                    Text {
                        text: "Backend"
                        color: textSecondary
                        font.pixelSize: 12
                    }

                    Text {
                        Layout.fillWidth: true
                        text: backendConnected ? backendStatus + "  " + (backend.cliPath ? shortHex(backend.cliPath) : "") : backendStatus
                        color: backendConnected ? "#d5dde6" : "#f1d176"
                        font.pixelSize: 13
                        elide: Text.ElideMiddle
                    }
                }

                ActionButton {
                    text: "Health"
                    enabled: backendConnected
                    onClicked: refreshBackendHealth()
                }

                ActionButton {
                    text: "Load State"
                    enabled: backendConnected && !backendRunning
                    onClicked: refreshWorkspaceState()
                }

                ActionButton {
                    text: backendRunning ? "Running" : "Run Backend Flow"
                    tone: "accent"
                    enabled: !backendRunning && backendConnected
                    onClicked: runBackendFlow()
                }

                ActionButton {
                    text: "Reset Backend"
                    tone: "ghost"
                    enabled: backendConnected && !backendRunning
                    onClicked: resetBackendWorkspace()
                }
            }
        }

        RowLayout {
            Layout.fillWidth: true
            Layout.fillHeight: true
            spacing: 16

            Rectangle {
                Layout.preferredWidth: 300
                Layout.fillHeight: true
                radius: 12
                color: panel
                border.color: border
                border.width: 1

                ColumnLayout {
                    anchors.fill: parent
                    anchors.margins: 16
                    spacing: 14

                    Text {
                        text: "Members"
                        color: textPrimary
                        font.pixelSize: 17
                        font.bold: true
                    }

                    RowLayout {
                        Layout.fillWidth: true
                        spacing: 10

                        Text {
                            text: "Threshold"
                            color: textSecondary
                            font.pixelSize: 13
                        }

                        SpinBox {
                            id: thresholdBox
                            from: 1
                            to: Math.max(1, members.length)
                            value: root.threshold
                            editable: false
                            onValueModified: {
                                root.threshold = value
                                root.aggregateReady = root.approvals.length >= root.threshold
                            }
                        }
                    }

                    Repeater {
                        model: members

                        Rectangle {
                            Layout.fillWidth: true
                            height: 76
                            radius: 10
                            color: activeMemberId === modelData.id ? panelRaised : bg1
                            border.color: activeMemberId === modelData.id ? accent : border
                            border.width: 1

                            MouseArea {
                                anchors.fill: parent
                                onClicked: activeMemberId = modelData.id
                            }

                            Column {
                                anchors.fill: parent
                                anchors.margins: 10
                                spacing: 5

                                Text {
                                    text: modelData.label
                                    color: textPrimary
                                    font.pixelSize: 14
                                    font.bold: true
                                }

                                Text {
                                    text: shortHex(modelData.commitment)
                                    color: textSecondary
                                    font.pixelSize: 12
                                    elide: Text.ElideMiddle
                                    width: parent.width
                                }

                                Text {
                                    text: approvalLabelFor(modelData.id)
                                    color: approvalStoredFor(modelData.id) ? "#87e0ad" : "#6f7a86"
                                    font.pixelSize: 12
                                }
                            }
                        }
                    }

                    RowLayout {
                        Layout.fillWidth: true
                        spacing: 8

                        ActionButton {
                            Layout.fillWidth: true
                            text: "Generate"
                            tone: "accent"
                            onClicked: generateMembers()
                        }

                        ActionButton {
                            Layout.fillWidth: true
                            text: "Approve"
                            onClicked: approveSelectedMember()
                        }
                    }

                    ActionButton {
                        Layout.fillWidth: true
                        text: "Check Duplicate"
                        tone: "warn"
                        onClicked: testDuplicateSelected()
                    }
                }
            }

            ColumnLayout {
                Layout.fillWidth: true
                Layout.fillHeight: true
                spacing: 16

                Rectangle {
                    Layout.fillWidth: true
                    height: 286
                    radius: 12
                    color: panel
                    border.color: border
                    border.width: 1

                    GridLayout {
                        anchors.fill: parent
                        anchors.margins: 16
                        columns: 2
                        columnSpacing: 14
                        rowSpacing: 12

                        Text {
                            Layout.columnSpan: 2
                            text: "Proposal"
                            color: textPrimary
                            font.pixelSize: 17
                            font.bold: true
                        }

                        Text {
                            text: "Multisig ID"
                            color: textSecondary
                            font.pixelSize: 13
                        }

                        TextField {
                            id: multisigIdField
                            Layout.fillWidth: true
                            text: multisigId
                            selectByMouse: true
                            onTextEdited: multisigId = text
                            color: textPrimary
                            placeholderTextColor: textSecondary
                            background: Rectangle {
                                radius: 8
                                color: bg2
                                border.color: multisigIdField.activeFocus ? accent : borderStrong
                                border.width: 1
                            }
                        }

                        Text {
                            text: "Proposal ID"
                            color: textSecondary
                            font.pixelSize: 13
                        }

                        TextField {
                            id: proposalIdField
                            Layout.fillWidth: true
                            text: proposalId
                            selectByMouse: true
                            onTextEdited: {
                                proposalId = text
                            }
                            color: textPrimary
                            placeholderTextColor: textSecondary
                            background: Rectangle {
                                radius: 8
                                color: bg2
                                border.color: proposalIdField.activeFocus ? accent : borderStrong
                                border.width: 1
                            }
                        }

                        Text {
                            text: "Target program"
                            color: textSecondary
                            font.pixelSize: 13
                        }

                        TextField {
                            id: targetProgramField
                            Layout.fillWidth: true
                            text: targetProgram
                            selectByMouse: true
                            onTextEdited: {
                                targetProgram = text
                            }
                            color: textPrimary
                            placeholderTextColor: textSecondary
                            background: Rectangle {
                                radius: 8
                                color: bg2
                                border.color: targetProgramField.activeFocus ? accent : borderStrong
                                border.width: 1
                            }
                        }

                        Text {
                            text: "Instruction"
                            color: textSecondary
                            font.pixelSize: 13
                        }

                        TextField {
                            id: instructionField
                            Layout.fillWidth: true
                            text: instruction
                            selectByMouse: true
                            onTextEdited: {
                                instruction = text
                            }
                            color: textPrimary
                            placeholderTextColor: textSecondary
                            background: Rectangle {
                                radius: 8
                                color: bg2
                                border.color: instructionField.activeFocus ? accent : borderStrong
                                border.width: 1
                            }
                        }

                        Text {
                            text: "Target account count"
                            color: textSecondary
                            font.pixelSize: 13
                        }

                        SpinBox {
                            Layout.fillWidth: true
                            from: 0
                            to: 8
                            value: targetAccountCount
                            editable: false
                            onValueModified: targetAccountCount = value
                        }

                        RowLayout {
                            Layout.columnSpan: 2
                            Layout.fillWidth: true
                            spacing: 8

                            ActionButton {
                                Layout.fillWidth: true
                                text: "Create Config"
                                onClicked: createBackendConfig()
                            }

                            ActionButton {
                                Layout.fillWidth: true
                                text: "Create Proposal"
                                tone: "accent"
                                onClicked: createProposal()
                            }

                            ActionButton {
                                Layout.fillWidth: true
                                text: "Aggregate"
                                onClicked: aggregateBackendApprovals()
                            }

                            ActionButton {
                                Layout.fillWidth: true
                                text: "Verify"
                                onClicked: verifyBackendAggregate()
                            }
                        }

                        RowLayout {
                            Layout.columnSpan: 2
                            Layout.fillWidth: true
                            spacing: 8

                            ActionButton {
                                Layout.fillWidth: true
                                text: "Prove"
                                tone: "accent"
                                enabled: backendConnected && aggregateReady
                                onClicked: proveBackendAggregate()
                            }

                            ActionButton {
                                Layout.fillWidth: true
                                text: "Execute Localnet"
                                enabled: backendConnected && aggregateReady
                                onClicked: executeLocalnetBackend()
                            }

                            ActionButton {
                                Layout.fillWidth: true
                                text: "Execute Testnet"
                                tone: "ok"
                                enabled: backendConnected && aggregateReady
                                onClicked: executeHostedTestnetBackend()
                            }
                        }

                        RowLayout {
                            Layout.columnSpan: 2
                            Layout.fillWidth: true
                            spacing: 8

                            Item {
                                Layout.fillWidth: true
                            }

                            ActionButton {
                                text: "Load"
                                onClicked: refreshWorkspaceState()
                            }

                            ActionButton {
                                tone: "ghost"
                                text: "Reset View"
                                onClicked: resetWorkflow()
                            }
                        }
                    }
                }

                RowLayout {
                    Layout.fillWidth: true
                    Layout.preferredHeight: 118
                    spacing: 12

                    StatusCard {
                        Layout.fillWidth: true
                        title: "Member root"
                        value: memberRoot === "" ? "not created" : shortHex(memberRoot)
                        tone: memberRoot !== "" ? "ok" : "muted"
                    }

                    StatusCard {
                        Layout.fillWidth: true
                        title: "Nullifiers"
                        value: approvals.length + " unique"
                        tone: duplicateRejected ? "warn" : "ok"
                    }

                    StatusCard {
                        Layout.fillWidth: true
                        title: "Execution gate"
                        value: aggregateReady ? "threshold met" : (proposalCreated ? "awaiting approvals" : "no proposal")
                        tone: aggregateReady ? "ok" : "warn"
                    }
                }

                Rectangle {
                    Layout.fillWidth: true
                    Layout.fillHeight: true
                    radius: 12
                    color: panel
                    border.color: border
                    border.width: 1

                    RowLayout {
                        anchors.fill: parent
                        anchors.margins: 16
                        spacing: 14

                        ColumnLayout {
                            Layout.fillWidth: true
                            Layout.fillHeight: true
                            spacing: 8

                            Text {
                                text: "Evidence"
                                color: textPrimary
                                font.pixelSize: 17
                                font.bold: true
                            }

                            ScrollView {
                                id: evidenceScroll
                                Layout.fillWidth: true
                                Layout.fillHeight: true
                                clip: true
                                ScrollBar.horizontal.policy: ScrollBar.AsNeeded
                                ScrollBar.vertical.policy: ScrollBar.AsNeeded

                                background: Rectangle {
                                    color: bg0
                                    border.color: border
                                    radius: 8
                                }

                                TextEdit {
                                    id: evidenceView
                                    readOnly: true
                                    wrapMode: TextEdit.NoWrap
                                    selectByMouse: true
                                    textFormat: TextEdit.PlainText
                                    text: evidenceJson
                                    color: "#d5dde6"
                                    selectedTextColor: "#101214"
                                    selectionColor: "#9bd1ff"
                                    font.family: "monospace"
                                    font.pixelSize: 12
                                    width: Math.max(evidenceScroll.availableWidth, contentWidth)
                                    height: Math.max(evidenceScroll.availableHeight, contentHeight)
                                }
                            }
                        }

                        ColumnLayout {
                            Layout.preferredWidth: 300
                            Layout.fillHeight: true
                            spacing: 8

                            Text {
                                text: "Activity"
                                color: textPrimary
                                font.pixelSize: 17
                                font.bold: true
                            }

                            ListView {
                                Layout.fillWidth: true
                                Layout.fillHeight: true
                                clip: true
                                model: logLines

                                delegate: Text {
                                    width: ListView.view.width
                                    text: modelData
                                    color: "#9ba7b4"
                                    font.pixelSize: 12
                                    wrapMode: Text.WordWrap
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    component StatusCard: Rectangle {
        property string title: ""
        property string value: ""
        property string tone: "muted"

        radius: 12
        color: panel
        border.color: tone === "ok" ? "#315f49" : tone === "warn" ? "#67582a" : border
        border.width: 1

        Column {
            anchors.fill: parent
            anchors.margins: 14
            spacing: 8

            Text {
                text: title
                color: textSecondary
                font.pixelSize: 12
            }

            Text {
                width: parent.width
                text: value
                color: tone === "ok" ? "#9be5b8" : tone === "warn" ? "#f1d176" : "#d5dde6"
                font.pixelSize: 16
                font.bold: true
                elide: Text.ElideMiddle
            }
        }
    }

    component ActionButton: Button {
        id: control
        property string tone: "default"

        implicitHeight: 40
        leftPadding: 14
        rightPadding: 14

        contentItem: Text {
            text: control.text
            color: !control.enabled ? "#617082"
                  : control.tone === "accent" ? "#eff7ff"
                  : control.tone === "ok" ? "#e9fff4"
                  : control.tone === "warn" ? "#fff2c7"
                  : root.textPrimary
            horizontalAlignment: Text.AlignHCenter
            verticalAlignment: Text.AlignVCenter
            elide: Text.ElideRight
            font.pixelSize: 13
            font.bold: true
        }

        background: Rectangle {
            radius: 8
            border.width: 1
            border.color: !control.enabled ? "#2d3742"
                         : control.down ? root.accentStrong
                         : control.tone === "accent" ? root.accentStrong
                         : control.tone === "ok" ? "#238a57"
                         : control.tone === "warn" ? "#8f6e1c"
                         : control.hovered ? root.borderStrong
                         : root.border
            color: !control.enabled ? "#151b22"
                 : control.down ? "#223041"
                 : control.tone === "accent" ? "#24558b"
                 : control.tone === "ok" ? "#163a2c"
                 : control.tone === "warn" ? "#3b3116"
                 : control.tone === "ghost" ? "#10151c"
                 : control.hovered ? "#202938"
                 : root.bg2
        }
    }
}
