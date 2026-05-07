import QtQuick 2.15
import QtQuick.Layouts 1.15

Item {
    id: emailList
    property var backend: null
    property var appState: null
    property var sendRequestFunc: null

    function setBackend(b) { emailList.backend = b }
    function setAppState(s) { emailList.appState = s }
    function setSendRequest(fn) { emailList.sendRequestFunc = fn; loadEmails() }

    property var emails: []

    function loadEmails() {
        if (sendRequestFunc && appState && appState.activeAccountId) {
            var folder = appState.activeFolder || "INBOX"
            sendRequestFunc("get_emails", {"account_id": appState.activeAccountId, "folder": folder}, function(resp) {
                var data = resp.data ? resp.data : resp
                if (data.emails) emails = data.emails
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
                    MouseArea { id: backMouse; anchors.fill: parent; onClicked: appState.currentView = "folder_list" }
                }

                Text {
                    text: appState.activeFolder || "INBOX"
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
            model: emails

            delegate: Rectangle {
                width: listView.width
                height: 126
                property bool unread: !modelData.read
                color: emailMouse.pressed ? "#e8e4dc" : (unread ? "#ffffff" : "#faf6f0")

                Rectangle {
                    anchors.bottom: parent.bottom; anchors.left: parent.left; anchors.right: parent.right
                    height: 2; color: "#e0dbd2"
                }

                Rectangle {
                    visible: unread
                    anchors.left: parent.left; anchors.top: parent.top; anchors.bottom: parent.bottom
                    width: 6; color: "#2c2c2c"
                }

                ColumnLayout {
                    anchors.fill: parent
                    anchors.margins: 21
                    anchors.leftMargin: unread ? 30 : 21
                    spacing: 3

                    RowLayout {
                        Layout.fillWidth: true

                        Text {
                            text: modelData.subject || "(no subject)"
                            font.pixelSize: 33; font.bold: unread; color: "#2c2c2c"
                            Layout.fillWidth: true; elide: Text.ElideRight
                        }

                        Text {
                            text: modelData.date || ""
                            font.pixelSize: 24; color: "#a09890"
                        }
                    }

                    RowLayout {
                        Layout.fillWidth: true

                        Text {
                            text: modelData.from || ""
                            font.pixelSize: 27; color: "#7a7368"
                            elide: Text.ElideRight; Layout.fillWidth: true
                        }

                        Text {
                            text: modelData.has_attachments ? "\ud83d\udcce" : ""
                            font.pixelSize: 27
                        }
                    }
                }

                MouseArea {
                    id: emailMouse; anchors.fill: parent
                    onClicked: {
                        appState.activeEmailId = modelData.id
                        appState.currentView = "email_reader"
                    }
                }
            }
        }

        Item {
            Layout.fillWidth: true
            Layout.fillHeight: emails.length === 0
            visible: emails.length === 0

            Text {
                anchors.centerIn: parent
                text: "No emails in this folder\nTap Sync on the accounts page"
                font.pixelSize: 33; color: "#a09890"
                horizontalAlignment: Text.AlignHCenter
            }
        }
    }
}
