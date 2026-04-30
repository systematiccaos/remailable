import QtQuick 2.15
import QtQuick.Layouts 1.15

Item {
    id: folderList
    property var backend: null
    property var appState: null

    function setBackend(b) { folderList.backend = b }
    function setAppState(s) { folderList.appState = s }

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

                    Text { anchors.centerIn: parent; text: "← Back"; font.pixelSize: 16 }

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

        // Placeholder folders
        ColumnLayout {
            Layout.fillWidth: true
            Layout.fillHeight: true
            spacing: 4

            Repeater {
                model: ["INBOX", "Sent", "Drafts", "Trash"]

                delegate: Rectangle {
                    width: folderList.width
                    height: 70
                    color: folderMouse.pressed ? "#e0e0e0" : "#ffffff"
                    border.color: "#eeeeee"
                    border.width: 1

                    Text {
                        anchors.verticalCenter: parent.verticalCenter
                        anchors.left: parent.left
                        anchors.leftMargin: 24
                        text: modelData
                        font.pixelSize: 22
                    }

                    MouseArea {
                        id: folderMouse
                        anchors.fill: parent
                        onClicked: appState.currentView = "email_list"
                    }
                }
            }
        }
    }
}