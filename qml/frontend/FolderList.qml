import QtQuick 2.15
import QtQuick.Layouts 1.15

Item {
    id: folderList
    property var backend: null
    property var appState: null
    property var sendRequestFunc: null

    function setBackend(b) { folderList.backend = b }
    function setAppState(s) { folderList.appState = s }
    function setSendRequest(fn) {
        folderList.sendRequestFunc = fn
        // Fetch folders once the function is available
        loadFolders()
    }

    property var folders: []

    function loadFolders() {
        if (sendRequestFunc && appState && appState.activeAccountId) {
            sendRequestFunc("get_folders", {"account_id": appState.activeAccountId}, function(resp) {
                var data = resp.data ? resp.data : resp
                if (data.folders) {
                    folders = data.folders
                }
            })
        }
    }

    ColumnLayout {
        anchors.fill: parent
        anchors.topMargin: 48
        spacing: 0

        // Header with back button
        Rectangle {
            Layout.fillWidth: true
            height: 80
            color: "#ffffff"

            RowLayout {
                anchors.fill: parent
                anchors.margins: 16

                Rectangle {
                    width: 80
                    height: 40
                    color: backMouse.pressed ? "#cccccc" : "#e0e0e0"
                    border.color: "#999999"
                    border.width: 1
                    radius: 4

                    Text { anchors.centerIn: parent; text: "\u2190 Back"; font.pixelSize: 16 }

                    MouseArea { id: backMouse; anchors.fill: parent; onClicked: appState.currentView = "account_list" }
                }

                Text {
                    text: (appState && appState.activeAccountName) || "Folders"
                    font.pixelSize: 28
                    font.bold: true
                    Layout.fillWidth: true
                }
            }
        }

        // Folder list from backend
        ListView {
            id: listView
            Layout.fillWidth: true
            Layout.fillHeight: true
            clip: true
            model: folders

            delegate: Rectangle {
                width: listView.width
                height: 70
                color: folderMouse.pressed ? "#e0e0e0" : "#ffffff"
                border.color: "#eeeeee"
                border.width: 1

                Text {
                    anchors.verticalCenter: parent.verticalCenter
                    anchors.left: parent.left
                    anchors.leftMargin: 24
                    text: modelData.name || "Unknown"
                    font.pixelSize: 22
                }

                MouseArea {
                    id: folderMouse
                    anchors.fill: parent
                    onClicked: {
                        appState.activeFolder = modelData.name
                        appState.currentView = "email_list"
                    }
                }
            }
        }

        // Empty state
        Item {
            Layout.fillWidth: true
            Layout.fillHeight: folders.length === 0 ? parent.height * 0.5 : 0
            visible: folders.length === 0

            Text {
                anchors.centerIn: parent
                text: "No folders yet\nTap Sync to fetch emails"
                font.pixelSize: 18
                color: "#999999"
                horizontalAlignment: Text.AlignHCenter
            }
        }
    }
}