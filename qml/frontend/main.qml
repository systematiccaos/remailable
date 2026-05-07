import QtQuick 2.15
import QtQuick.Window 2.15
import QtQuick.Layouts 1.15
import net.asivery.AppLoad 1.0

Item {
    id: root
    signal close
    function unloading() { backend.terminate() }

    AppLoad {
        id: backend
        applicationID: "remailable"

        onMessageReceived: function(type, contents) {
            var msg
            try { msg = JSON.parse(contents) } catch(e) { return }

            if (msg.id !== undefined && _callbacks[msg.id]) {
                var cb = _callbacks[msg.id]
                delete _callbacks[msg.id]
                cb(msg)
                return
            }

            if (msg.event) {
                switch (msg.event) {
                case "initial_state":
                    appState.currentView = msg.data.current_view || "account_list"
                    appState.accountCount = msg.data.account_count || 0
                    appState.syncStatus = msg.data.sync_status || "idle"
                    if (msg.data.accounts) appState.accounts = msg.data.accounts
                    break
                case "accounts_changed":
                    sendRequest("get_accounts", {}, function(resp) {
                        var d = resp.data ? resp.data.accounts : resp.accounts
                        if (d) { appState.accounts = d; appState.accountCount = d.length }
                    })
                    break
                case "sync_status":
                    if (msg.data) appState.syncStatus = msg.data.status || "idle"
                    break
                }
            }
        }
    }

    property int _nextId: 1
    property var _callbacks: ({})

    function sendRequest(action, params, callback) {
        var reqId = _nextId++
        if (callback) _callbacks[reqId] = callback
        backend.sendMessage(1, JSON.stringify({ action: action, params: params || {}, id: reqId }))
    }

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

    Rectangle { anchors.fill: parent; color: "#faf6f0" }

    Loader {
        id: contentLoader
        anchors.fill: parent
        source: {
            switch (appState.currentView) {
            case "account_settings": return "AccountSettings.qml"
            case "folder_list":      return "FolderList.qml"
            case "email_list":       return "EmailList.qml"
            case "email_reader":     return "EmailReader.qml"
            default:                 return "AccountList.qml"
            }
        }
        onLoaded: {
            if (item && item.setBackend)     item.setBackend(backend)
            if (item && item.setAppState)     item.setAppState(appState)
            if (item && item.setSendRequest)  item.setSendRequest(sendRequest)
        }
    }

    Rectangle {
        anchors.top: parent.top
        anchors.left: parent.left
        anchors.right: parent.right
        height: 72
        color: "#ffffff"
        border.color: "#d4cec4"
        border.width: 2

        RowLayout {
            anchors.fill: parent
            anchors.leftMargin: 24
            anchors.rightMargin: 12

            Text {
                text: "remailable"
                font.pixelSize: 33
                font.bold: true
                color: "#2c2c2c"
                Layout.fillWidth: true
            }

            Text {
                text: appState.syncStatus === "syncing" ? "syncing..." : "idle"
                font.pixelSize: 27
                color: "#7a7368"
            }

            Rectangle {
                width: 54; height: 54
                color: closeMa.pressed ? "#e8e4dc" : "#ffffff"
                border.color: "#999999"; border.width: 3
                Text { anchors.centerIn: parent; text: "✕"; font.pixelSize: 30; font.bold: true; color: "#2c2c2c" }
                MouseArea { id: closeMa; anchors.fill: parent; onClicked: root.close() }
            }
        }
    }

    Component.onCompleted: {}
}
