import QtQuick 2.15
import QtQuick.Window 2.15
import QtQuick.Layouts 1.15
import net.asivery.AppLoad 1.0

Item {
    id: root

    // AppLoad required signals
    signal close
    function unloading() {
        backend.terminate()
    }

    // AppLoad backend bridge
    AppLoad {
        id: backend
        applicationID: "remailable"

        onMessageReceived: function(type, contents) {
            var msg
            try {
                msg = JSON.parse(contents)
            } catch(e) {
                return
            }

            // Response to a request
            if (msg.id !== undefined && _callbacks[msg.id]) {
                var cb = _callbacks[msg.id]
                delete _callbacks[msg.id]
                cb(msg)
                return
            }

            // Event from backend
            if (msg.event) {
                switch (msg.event) {
                case "initial_state":
                    appState.currentView = msg.data.current_view || "account_list"
                    appState.accountCount = msg.data.account_count || 0
                    appState.syncStatus = msg.data.sync_status || "idle"
                    if (msg.data.accounts) {
                        appState.accounts = msg.data.accounts
                    }
                    break
                case "accounts_changed":
                    sendRequest("get_accounts", {}, function(resp) {
                        var accountsData = resp.data ? resp.data.accounts : resp.accounts
                        if (accountsData) {
                            appState.accounts = accountsData
                            appState.accountCount = accountsData.length
                        }
                    })
                    break
                case "sync_status":
                    if (msg.data) {
                        appState.syncStatus = msg.data.status || "idle"
                    }
                    break
                }
            }
        }
    }

    // Request tracking
    property int _nextId: 1
    property var _callbacks: ({})

    function sendRequest(action, params, callback) {
        var reqId = _nextId++
        if (callback) {
            _callbacks[reqId] = callback
        }
        var payload = JSON.stringify({
            "action": action,
            "params": params || {},
            "id": reqId
        })
        backend.sendMessage(1, payload)
    }

    // App state
    QtObject {
        id: appState
        property string currentView: "account_list"
        property int accountCount: 0
        property string syncStatus: "idle"
        property var accounts: []
        property string activeAccountId: ""
        property string activeAccountName: ""
        property string activeFolder: "INBOX"
        property string activeEmailId: ""
    }

    // Background
    Rectangle {
        anchors.fill: parent
        color: "#ffffff"
    }

    // Main content
    Loader {
        id: contentLoader
        anchors.fill: parent
        source: {
            switch (appState.currentView) {
            case "account_settings": return "AccountSettings.qml"
            case "folder_list": return "FolderList.qml"
            case "email_list": return "EmailList.qml"
            case "email_reader": return "EmailReader.qml"
            case "account_list":
            default: return "AccountList.qml"
            }
        }

        onLoaded: {
            if (item && item.setBackend) {
                item.setBackend(backend)
            }
            if (item && item.setAppState) {
                item.setAppState(appState)
            }
            if (item && item.setSendRequest) {
                item.setSendRequest(sendRequest)
            }
        }
    }

    // Sync status indicator at top
    Rectangle {
        id: syncBar
        anchors.top: parent.top
        anchors.left: parent.left
        anchors.right: parent.right
        height: 40
        color: appState.syncStatus === "syncing" ? "#e0e0e0" : "#f5f5f5"
        border.color: "#cccccc"
        border.width: 1

        RowLayout {
            anchors.fill: parent
            anchors.margins: 8

            Text {
                text: "remailable"
                font.pixelSize: 20
                font.bold: true
                Layout.fillWidth: true
            }

            Text {
                text: appState.syncStatus === "syncing" ? "⟳ Syncing..." : "● " + appState.syncStatus
                font.pixelSize: 14
                color: "#666666"
            }

            // Close button
            Rectangle {
                width: 30
                height: 30
                radius: 15
                color: closeMa.pressed ? "#cccccc" : "#e0e0e0"
                border.color: "#999999"
                border.width: 1

                Text {
                    anchors.centerIn: parent
                    text: "✕"
                    font.pixelSize: 16
                    font.bold: true
                }

                MouseArea {
                    id: closeMa
                    anchors.fill: parent
                    onClicked: root.close()
                }
            }
        }
    }

    Component.onCompleted: {
        // Backend will send initial_state after SYS_NEW_FRONTEND
    }
}