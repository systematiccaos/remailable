import QtQuick 2.15
import QtQuick.Layouts 1.15

// Account list — the default landing view
Item {
    id: accountList
    property var backend: null
    property var appState: null
    property var sendRequestFunc: null

    function setBackend(b) { accountList.backend = b }
    function setAppState(s) { accountList.appState = s }
    function setSendRequest(fn) { accountList.sendRequestFunc = fn }

    ColumnLayout {
        anchors.fill: parent
        anchors.topMargin: 48  // below sync bar
        spacing: 0

        // Header
        Rectangle {
            Layout.fillWidth: true
            height: 80
            color: "#ffffff"

            RowLayout {
                anchors.fill: parent
                anchors.margins: 16

                Text {
                    text: "Email Accounts"
                    font.pixelSize: 28
                    font.bold: true
                    Layout.fillWidth: true
                }

                // Sync button
                Rectangle {
                    height: 44
                    width: 80
                    color: syncMouse.pressed ? "#cccccc" : "#e0e0e0"
                    border.color: "#999999"
                    border.width: 1
                    radius: 4

                    Text {
                        anchors.centerIn: parent
                        text: appState.syncStatus === "syncing" ? "..." : "Sync"
                        font.pixelSize: 16
                        font.bold: true
                    }

                    MouseArea {
                        id: syncMouse
                        anchors.fill: parent
                        onClicked: {
                            if (sendRequestFunc) {
                                sendRequestFunc("sync", {}, function(resp) {})
                            }
                        }
                    }
                }

                // Add account button
                Rectangle {
                    height: 44
                    width: 44
                    color: addMouse.pressed ? "#cccccc" : "#e0e0e0"
                    border.color: "#999999"
                    border.width: 1
                    radius: 4

                    Text {
                        anchors.centerIn: parent
                        text: "+"
                        font.pixelSize: 28
                        font.bold: true
                    }

                    MouseArea {
                        id: addMouse
                        anchors.fill: parent
                        onClicked: appState.currentView = "account_settings"
                    }
                }
            }
        }

        // Account list
        ListView {
            id: listView
            Layout.fillWidth: true
            Layout.fillHeight: true
            clip: true
            model: appState ? appState.accounts : []

            delegate: Rectangle {
                width: listView.width
                height: 80
                color: "#ffffff"
                border.color: "#e0e0e0"
                border.width: 1

                RowLayout {
                    anchors.fill: parent
                    anchors.margins: 12

                    // Account info — tappable to open folders
                    ColumnLayout {
                        Layout.fillWidth: true
                        spacing: 4

                        Text {
                            text: modelData.display_name || modelData.email || "Unknown"
                            font.pixelSize: 20
                            font.bold: true
                        }

                        Text {
                            text: modelData.email || modelData.imap_host || ""
                            font.pixelSize: 14
                            color: "#666666"
                        }
                    }

                    // Folders button
                    Rectangle {
                        height: 36
                        width: 90
                        color: folderMouse.pressed ? "#cccccc" : "#e0e0e0"
                        border.color: "#999999"
                        border.width: 1
                        radius: 4

                        Text {
                            anchors.centerIn: parent
                            text: "Folders"
                            font.pixelSize: 14
                        }

                        MouseArea {
                            id: folderMouse
                            anchors.fill: parent
                            onClicked: {
                                appState.activeAccountId = modelData.id
                                appState.activeAccountName = modelData.display_name
                                appState.currentView = "folder_list"
                            }
                        }
                    }

                    // Remove button
                    Rectangle {
                        height: 36
                        width: 70
                        color: delMouse.pressed ? "#ff9999" : "#ffe0e0"
                        border.color: "#cc6666"
                        border.width: 1
                        radius: 4

                        Text {
                            anchors.centerIn: parent
                            text: "Remove"
                            font.pixelSize: 14
                            color: "#cc3333"
                        }

                        MouseArea {
                            id: delMouse
                            anchors.fill: parent
                            onClicked: {
                                if (sendRequestFunc) {
                                    sendRequestFunc("remove_account", {"id": modelData.id}, function(resp) {})
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}