import QtQuick 2.15
import QtQuick.Layouts 1.15

// Account list — the default landing view
Item {
    id: accountList
    property var backend: null
    property var appState: null

    function setBackend(b) { accountList.backend = b }
    function setAppState(s) { accountList.appState = s }

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
                color: ma.pressed ? "#f0f0f0" : "#ffffff"
                border.color: "#e0e0e0"
                border.width: 1

                RowLayout {
                    anchors.fill: parent
                    anchors.margins: 12

                    ColumnLayout {
                        Layout.fillWidth: true
                        spacing: 4

                        Text {
                            text: modelData.display_name || modelData.email || "Unknown"
                            font.pixelSize: 20
                            font.bold: true
                        }

                        Text {
                            text: modelData.imap_host || ""
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
                                var payload = JSON.stringify({
                                    "action": "remove_account",
                                    "params": {"id": modelData.id},
                                    "id": 0
                                })
                                backend.sendMessage(1, payload)
                            }
                        }
                    }
                }

                MouseArea {
                    id: ma
                    anchors.fill: parent
                    onClicked: {
                        appState.activeAccountId = modelData.id
                    }
                }
            }
        }
    }
}