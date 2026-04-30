import QtQuick 2.17
import QtQuick.Layouts 1.17
import io.remailable.Remailable 1.0

Item {
    id: folderList

    // Header: back button + "Folders" title + account name
    Rectangle {
        id: header
        anchors.top: parent.top
        anchors.left: parent.left
        anchors.right: parent.right
        height: 80
        color: "#ffffff"
        border.bottom: "grey"

        RowLayout {
            anchors.fill: parent
            anchors.margins: 16

            Rectangle {
                height: 44
                width: 80
                color: backMouse.pressed ? "#cccccc" : "#e0e0e0"
                border.color: "#999999"
                border.width: 1
                radius: 4

                Text {
                    anchors.centerIn: parent
                    text: "\u2190 Back"
                    font.pixelSize: 16
                }

                MouseArea {
                    id: backMouse
                    anchors.fill: parent
                    onClicked: appModel.current_view = "account_list"
                }
            }

            Text {
                text: "Folders"
                font.pixelSize: 28
                font.bold: true
                Layout.fillWidth: true
            }

            Text {
                text: appModel.active_account_name
                font.pixelSize: 18
                color: "#666666"
            }
        }
    }

    // Folder list
    ListView {
        id: listView
        anchors.top: header.bottom
        anchors.bottom: parent.bottom
        anchors.left: parent.left
        anchors.right: parent.right
        anchors.margins: 8
        clip: true

        model: folderListModel.folder_count

        delegate: Rectangle {
            width: listView.width
            height: 64
            color: ma.pressed ? "#f0f0f0" : "#ffffff"
            border.bottom: "grey"

            RowLayout {
                anchors.fill: parent
                anchors.margins: 12

                Text {
                    text: folderListModel.get_folder_name(index)
                    font.pixelSize: 20
                    Layout.fillWidth: true
                }

                Text {
                    text: "\u25B6"
                    font.pixelSize: 20
                    color: "#666666"
                }
            }

            MouseArea {
                id: ma
                anchors.fill: parent
                onClicked: {
                    appModel.selected_folder = folderListModel.get_folder_name(index)
                    appModel.current_view = "email_list"
                }
            }
        }
    }

    Component.onCompleted: folderListModel.refresh_folders(appModel.active_account_id)
}