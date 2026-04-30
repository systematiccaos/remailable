import QtQuick 2.17
import QtQuick.Layouts 1.17
import io.remailable.Remailable 1.0

Item {
    id: emailList

    // Header: back button + folder name + unread count
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
                    onClicked: appModel.current_view = "folder_list"
                }
            }

            Text {
                text: appModel.selected_folder
                font.pixelSize: 28
                font.bold: true
                Layout.fillWidth: true
            }

            Text {
                id: unreadCount
                font.pixelSize: 18
                color: "#666666"
                text: {
                    var count = 0;
                    for (var i = 0; i < emailListModel.email_count; i++) {
                        if (!emailListModel.get_email_read(i)) count++;
                    }
                    return count > 0 ? count + " unread" : ""
                }
            }
        }
    }

    // Email list
    ListView {
        id: listView
        anchors.top: header.bottom
        anchors.bottom: parent.bottom
        anchors.left: parent.left
        anchors.right: parent.right
        anchors.margins: 8
        clip: true

        model: emailListModel.email_count

        delegate: Rectangle {
            width: listView.width
            height: 80
            color: ma.pressed ? "#f0f0f0" : "#ffffff"
            border.bottom: "grey"

            RowLayout {
                anchors.fill: parent
                anchors.margins: 12
                spacing: 8

                // Read/unread indicator (READ-05)
                Text {
                    text: emailListModel.get_email_read(index) ? "\u25CB" : "\u25CF"
                    font.pixelSize: 14
                    color: emailListModel.get_email_read(index) ? "#cccccc" : "#000000"
                    Layout.alignment: Qt.AlignVCenter

                    MouseArea {
                        anchors.fill: parent
                        onClicked: emailListModel.toggle_email_read(index)
                    }
                }

                // Subject + from + date column
                ColumnLayout {
                    Layout.fillWidth: true
                    spacing: 2

                    // Subject line (single line, elided)
                    Text {
                        text: emailListModel.get_email_subject(index)
                        font.pixelSize: 20
                        font.bold: !emailListModel.get_email_read(index)
                        elide: Text.ElideRight
                        Layout.fillWidth: true
                        maximumLineCount: 1
                    }

                    // From + date row
                    RowLayout {
                        Layout.fillWidth: true
                        spacing: 8

                        Text {
                            text: emailListModel.get_email_from(index)
                            font.pixelSize: 14
                            color: "#666666"
                            elide: Text.ElideRight
                            Layout.fillWidth: true
                            maximumLineCount: 1
                        }

                        Text {
                            text: emailListModel.get_email_date(index)
                            font.pixelSize: 14
                            color: "#666666"
                        }
                    }
                }

                // Attachment indicator (ATCH-01 preview)
                Text {
                    text: emailListModel.get_email_has_attachments(index) ? "\uD83D\uDCCE" : ""
                    font.pixelSize: 16
                    color: "#666666"
                    visible: emailListModel.get_email_has_attachments(index)
                    Layout.alignment: Qt.AlignVCenter
                }
            }

            MouseArea {
                id: ma
                anchors.fill: parent
                onClicked: {
                    appModel.selected_email_id = emailListModel.get_email_id(index)
                    appModel.current_view = "email_reader"
                }
            }
        }
    }

    Component.onCompleted: emailListModel.refresh_emails(appModel.active_account_id, appModel.selected_folder)
}