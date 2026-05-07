import QtQuick 2.15
import QtQuick.Layouts 1.15

Item {
    id: folderList
    property var backend: null
    property var appState: null
    property var sendRequestFunc: null

    function setBackend(b) { folderList.backend = b }
    function setAppState(s) { folderList.appState = s }
    function setSendRequest(fn) { folderList.sendRequestFunc = fn; loadFolders() }

    property var folders: []

    function loadFolders() {
        if (sendRequestFunc && appState && appState.activeAccountId) {
            sendRequestFunc("get_folders", {"account_id": appState.activeAccountId}, function(resp) {
                var data = resp.data ? resp.data : resp
                if (data.folders) folders = data.folders
            })
        }
    }

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

                Rectangle {
                    width: 144; height: 72
                    color: backMouse.pressed ? "#e8e4dc" : "#ffffff"
                    border.color: "#777777"; border.width: 3
                    Text { anchors.centerIn: parent; text: "\u2190 Back"; font.pixelSize: 30; font.bold: true; color: "#2c2c2c" }
                    MouseArea { id: backMouse; anchors.fill: parent; onClicked: appState.currentView = "account_list" }
                }

                Text {
                    text: (appState && appState.activeAccountName) || "Folders"
                    font.pixelSize: 42; font.bold: true; color: "#2c2c2c"
                    Layout.fillWidth: true
                }
            }
        }

        ListView {
            id: listView
            Layout.fillWidth: true
            Layout.fillHeight: true
            clip: true
            model: folders

            delegate: Rectangle {
                width: listView.width
                height: 108
                color: folderMouse.pressed ? "#e8e4dc" : "#ffffff"

                Rectangle {
                    anchors.bottom: parent.bottom; anchors.left: parent.left; anchors.right: parent.right
                    height: 2; color: "#e0dbd2"
                }

                Text {
                    anchors.verticalCenter: parent.verticalCenter
                    anchors.left: parent.left; anchors.leftMargin: 36
                    text: modelData.name || "Unknown"
                    font.pixelSize: 36; font.bold: true; color: "#2c2c2c"
                }

                MouseArea {
                    id: folderMouse; anchors.fill: parent
                    onClicked: {
                        appState.activeFolder = modelData.name
                        appState.currentView = "email_list"
                    }
                }
            }
        }

        Item {
            Layout.fillWidth: true
            Layout.fillHeight: folders.length === 0
            visible: folders.length === 0

            Text {
                anchors.centerIn: parent
                text: "No folders yet\nTap Sync to fetch emails"
                font.pixelSize: 33; color: "#a09890"
                horizontalAlignment: Text.AlignHCenter
            }
        }
    }
}
