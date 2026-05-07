import QtQuick 2.15
import QtQuick.Layouts 1.15

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
        anchors.topMargin: 84
        spacing: 0

        Rectangle {
            Layout.fillWidth: true
            height: 126
            color: "#faf6f0"

            RowLayout {
                anchors.fill: parent
                anchors.margins: 30
                spacing: 18

                Text {
                    text: "Email Accounts"
                    font.pixelSize: 42
                    font.bold: true
                    color: "#2c2c2c"
                    Layout.fillWidth: true
                }

                Rectangle {
                    width: 150; height: 72
                    color: syncMouse.pressed ? "#e8e4dc" : "#ffffff"
                    border.color: "#777777"; border.width: 3
                    Text {
                        anchors.centerIn: parent
                        text: appState.syncStatus === "syncing" ? "..." : "Sync"
                        font.pixelSize: 30; font.bold: true; color: "#2c2c2c"
                    }
                    MouseArea {
                        id: syncMouse; anchors.fill: parent
                        onClicked: { if (sendRequestFunc) sendRequestFunc("sync", {}, function(resp) {}) }
                    }
                }

                Rectangle {
                    width: 72; height: 72
                    color: addMouse.pressed ? "#e8e4dc" : "#ffffff"
                    border.color: "#777777"; border.width: 3
                    Text {
                        anchors.centerIn: parent
                        text: "+"; font.pixelSize: 48; font.bold: true; color: "#2c2c2c"
                    }
                    MouseArea {
                        id: addMouse; anchors.fill: parent
                        onClicked: appState.currentView = "account_settings"
                    }
                }
            }
        }

        ListView {
            id: listView
            Layout.fillWidth: true
            Layout.fillHeight: true
            clip: true
            model: appState ? appState.accounts : []

            delegate: Rectangle {
                width: listView.width
                height: 126
                color: "#ffffff"
                property color borderC: "#e0dbd2"
                Rectangle {
                    anchors.top: parent.top; anchors.left: parent.left; anchors.right: parent.right
                    height: 2; color: "#e0dbd2"
                }

                RowLayout {
                    anchors.fill: parent
                    anchors.margins: 24
                    spacing: 18

                    ColumnLayout {
                        Layout.fillWidth: true
                        spacing: 6

                        Text {
                            text: modelData.display_name || modelData.email || "Unknown"
                            font.pixelSize: 33; font.bold: true; color: "#2c2c2c"
                        }
                        Text {
                            text: modelData.email || modelData.imap_host || ""
                            font.pixelSize: 27; color: "#7a7368"
                        }
                    }

                    Rectangle {
                        width: 150; height: 66
                        color: folderMouse.pressed ? "#e8e4dc" : "#ffffff"
                        border.color: "#777777"; border.width: 3
                        Text {
                            anchors.centerIn: parent
                            text: "Folders"; font.pixelSize: 27; font.bold: true; color: "#2c2c2c"
                        }
                        MouseArea {
                            id: folderMouse; anchors.fill: parent
                            onClicked: {
                                appState.activeAccountId = modelData.id
                                appState.activeAccountName = modelData.display_name
                                appState.currentView = "folder_list"
                            }
                        }
                    }

                    Rectangle {
                        width: 135; height: 66
                        color: delMouse.pressed ? "#e8d0d0" : "#ffffff"
                        border.color: "#cc7777"; border.width: 3
                        Text {
                            anchors.centerIn: parent
                            text: "Remove"; font.pixelSize: 27; font.bold: true; color: "#a03030"
                        }
                        MouseArea {
                            id: delMouse; anchors.fill: parent
                            onClicked: {
                                if (sendRequestFunc) sendRequestFunc("remove_account", {"id": modelData.id}, function(resp) {})
                            }
                        }
                    }
                }
            }
        }
    }
}
