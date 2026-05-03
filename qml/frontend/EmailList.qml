import QtQuick 2.15
import QtQuick.Layouts 1.15

Item {
    id: emailList
    property var backend: null
    property var appState: null
    property var sendRequestFunc: null

    function setBackend(b) { emailList.backend = b }
    function setAppState(s) { emailList.appState = s }
    function setSendRequest(fn) {
        emailList.sendRequestFunc = fn
        loadEmails()
    }

    property var emails: []

    function loadEmails() {
        if (sendRequestFunc && appState && appState.activeAccountId) {
            var folder = appState.activeFolder || "INBOX"
            sendRequestFunc("get_emails", {"account_id": appState.activeAccountId, "folder": folder}, function(resp) {
                var data = resp.data ? resp.data : resp
                if (data.emails) {
                    emails = data.emails
                }
            })
        }
    }

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
                    Text { anchors.centerIn: parent; text: "\u2190 Back"; font.pixelSize: 16 }
                    MouseArea { id: backMouse; anchors.fill: parent; onClicked: appState.currentView = "folder_list" }
                }

                Text {
                    text: appState.activeFolder || "INBOX"
                    font.pixelSize: 28
                    font.bold: true
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
                height: 80
                color: emailMouse.pressed ? "#e0e0e0" : (modelData.read ? "#ffffff" : "#f0f4ff")
                border.color: "#eeeeee"
                border.width: 1

                ColumnLayout {
                    anchors.fill: parent
                    anchors.margins: 12
                    spacing: 2

                    RowLayout {
                        Layout.fillWidth: true

                        Text {
                            text: modelData.subject || "(no subject)"
                            font.pixelSize: 18
                            font.bold: !modelData.read
                            Layout.fillWidth: true
                            elide: Text.ElideRight
                        }

                        Text {
                            text: modelData.date || ""
                            font.pixelSize: 12
                            color: "#999999"
                        }
                    }

                    RowLayout {
                        Layout.fillWidth: true

                        Text {
                            text: modelData.from || ""
                            font.pixelSize: 14
                            color: "#666666"
                            elide: Text.ElideRight
                            Layout.fillWidth: true
                        }

                        Text {
                            text: modelData.has_attachments ? "\uD83D\uDCCE" : ""
                            font.pixelSize: 14
                        }
                    }
                }

                MouseArea {
                    id: emailMouse
                    anchors.fill: parent
                    onClicked: {
                        appState.activeEmailId = modelData.id
                        appState.currentView = "email_reader"
                    }
                }
            }
        }

        // Empty state
        Item {
            Layout.fillWidth: true
            Layout.fillHeight: emails.length === 0 ? parent.height * 0.5 : 0
            visible: emails.length === 0

            Text {
                anchors.centerIn: parent
                text: "No emails in this folder\nTap Sync on the accounts page"
                font.pixelSize: 18
                color: "#999999"
                horizontalAlignment: Text.AlignHCenter
            }
        }
    }
}