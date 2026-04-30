import QtQuick 2.15
import QtQuick.Layouts 1.15

Item {
    id: emailList
    property var backend: null
    property var appState: null

    function setBackend(b) { emailList.backend = b }
    function setAppState(s) { emailList.appState = s }

    ColumnLayout {
        anchors.fill: parent
        anchors.topMargin: 48
        spacing: 0

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
                    MouseArea { id: backMouse; anchors.fill: parent; onClicked: appState.currentView = "folder_list" }
                }

                Text {
                    text: "INBOX"
                    font.pixelSize: 28
                    font.bold: true
                    Layout.fillWidth: true
                }
            }
        }

        // Placeholder — emails will be loaded from backend
        Item {
            Layout.fillWidth: true
            Layout.fillHeight: true

            Text {
                anchors.centerIn: parent
                text: "No emails yet\nSync will populate this view"
                font.pixelSize: 18
                color: "#999999"
                horizontalAlignment: Text.AlignHCenter
            }
        }
    }
}