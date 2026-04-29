import QtQuick 2.17
import QtQuick.Layouts 1.17
import io.remailable.Remailable 1.0

Rectangle {
    id: syncIndicator
    height: 40
    color: "#f0f0f0"
    border.color: "#cccccc"
    border.width: 1

    RowLayout {
        anchors.fill: parent
        anchors.margins: 8
        spacing: 12

        // Status icon/text
        Text {
            id: statusText
            Layout.fillWidth: true
            font.pixelSize: 18
            font.bold: true
            text: {
                switch (appModel.sync_status_text) {
                    case "syncing": return "\u25C6 Syncing..."
                    case "synced": return "\u2713 Synced"
                    case "offline": return "\u25CB Offline"
                    case "error": return "\u2717 Error"
                    default: return "\u25CB Idle"
                }
            }
            color: {
                switch (appModel.sync_status_text) {
                    case "syncing": return "#666666"
                    case "synced": return "#333333"
                    case "offline": return "#999999"
                    case "error": return "#cc0000"
                    default: return "#666666"
                }
            }
        }

        // Sync now button
        Rectangle {
            height: 28
            width: 90
            color: syncMouse.pressed ? "#cccccc" : "#e0e0e0"
            border.color: "#999999"
            border.width: 1
            radius: 4

            Text {
                anchors.centerIn: parent
                text: "Sync Now"
                font.pixelSize: 14
            }

            MouseArea {
                id: syncMouse
                anchors.fill: parent
                onClicked: accountListModel.sync_all()
            }
        }
    }
}