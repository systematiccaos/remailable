import QtQuick 2.17
import QtQuick.Layouts 1.17
import io.remailable.Remailable 1.0

Item {
    id: accountList

    // Header
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

            Text {
                text: "Email Accounts"
                font.pixelSize: 28
                font.bold: true
                Layout.fillWidth: true
            }

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
                    onClicked: appModel.current_view = "account_settings"
                }
            }
        }
    }

    // Account list
    ListView {
        id: listView
        anchors.top: header.bottom
        anchors.bottom: parent.bottom
        anchors.left: parent.left
        anchors.right: parent.right
        anchors.margins: 8
        clip: true

        model: accountListModel.account_count

        delegate: Rectangle {
            width: listView.width
            height: 80
            color: ma.pressed ? "#f0f0f0" : "#ffffff"
            border.bottom: "grey"

            RowLayout {
                anchors.fill: parent
                anchors.margins: 12

                ColumnLayout {
                    Layout.fillWidth: true
                    spacing: 4

                    Text {
                        text: accountListModel.get_account_display_name(index)
                        font.pixelSize: 20
                        font.bold: true
                    }

                    Text {
                        text: accountListModel.get_account_imap_host(index)
                        font.pixelSize: 14
                        color: "#666666"
                    }
                }

                // Delete button
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
                            var accountId = accountListModel.get_account_id(index);
                            accountListModel.remove_account(accountId);
                            accountListModel.refresh_accounts();
                        }
                    }
                }
            }

            MouseArea {
                id: ma
                anchors.fill: parent
                onClicked: {
                    var accountId = accountListModel.get_account_id(index);
                    appModel.active_account_id = accountId;
                }
            }
        }
    }
}