import QtQuick 2.15
import QtQuick.Layouts 1.15

Item {
    id: settings
    property var backend: null
    property var appState: null
    property var sendRequestFunc: null

    function setBackend(b) { settings.backend = b }
    function setAppState(s) { settings.appState = s }
    function setSendRequest(fn) { settings.sendRequestFunc = fn }

    property string displayName: ""
    property string emailAddr: ""
    property string imapHost: ""
    property string imapPort: "993"
    property string smtpHost: ""
    property string smtpPort: "587"
    property string username: ""
    property string password: ""

    ColumnLayout {
        anchors.fill: parent
        anchors.topMargin: 84
        anchors.margins: 30
        spacing: 18

        RowLayout {
            Layout.fillWidth: true
            spacing: 18

            Rectangle {
                width: 144; height: 72
                color: backMouse.pressed ? "#e8e4dc" : "#ffffff"
                border.color: "#777777"; border.width: 3
                Text { anchors.centerIn: parent; text: "\u2190 Back"; font.pixelSize: 30; font.bold: true; color: "#2c2c2c" }
                MouseArea { id: backMouse; anchors.fill: parent; onClicked: appState.currentView = "account_list" }
            }

            Text {
                text: "Add Account"
                font.pixelSize: 42; font.bold: true; color: "#2c2c2c"
                Layout.fillWidth: true
            }
        }

        Flickable {
            Layout.fillWidth: true
            Layout.fillHeight: true
            clip: true
            contentWidth: parent.width
            contentHeight: formColumn.height + 36

            ColumnLayout {
                id: formColumn
                width: parent.width
                spacing: 15

                Repeater {
                    model: [
                        { label: "Display Name", field: "displayName", placeholder: "e.g. John Doe" },
                        { label: "Email", field: "emailAddr", placeholder: "john@example.com" },
                        { label: "IMAP Host", field: "imapHost", placeholder: "imap.example.com" },
                        { label: "IMAP Port", field: "imapPort", placeholder: "993" },
                        { label: "SMTP Host", field: "smtpHost", placeholder: "smtp.example.com" },
                        { label: "SMTP Port", field: "smtpPort", placeholder: "587" },
                        { label: "Username", field: "username", placeholder: "john@example.com" },
                        { label: "Password", field: "password", placeholder: "\u00b7\u00b7\u00b7\u00b7\u00b7\u00b7\u00b7\u00b7" }
                    ]

                    delegate: ColumnLayout {
                        width: parent.width
                        spacing: 6

                        Text {
                            text: modelData.label
                            font.pixelSize: 30; font.bold: true; color: "#2c2c2c"
                        }

                        Rectangle {
                            Layout.fillWidth: true
                            height: 75
                            color: "#ffffff"
                            border.color: "#bbb5aa"; border.width: 3

                            TextInput {
                                anchors.fill: parent
                                anchors.margins: 15
                                font.pixelSize: 30; color: "#2c2c2c"
                                text: {
                                    switch (modelData.field) {
                                    case "displayName": return displayName
                                    case "emailAddr":   return emailAddr
                                    case "imapHost":    return imapHost
                                    case "imapPort":    return imapPort
                                    case "smtpHost":    return smtpHost
                                    case "smtpPort":    return smtpPort
                                    case "username":    return username
                                    case "password":    return password
                                    }
                                }
                                onTextChanged: {
                                    switch (modelData.field) {
                                    case "displayName": displayName = text; break
                                    case "emailAddr":   emailAddr = text; break
                                    case "imapHost":    imapHost = text; break
                                    case "imapPort":    imapPort = text; break
                                    case "smtpHost":    smtpHost = text; break
                                    case "smtpPort":    smtpPort = text; break
                                    case "username":    username = text; break
                                    case "password":    password = text; break
                                    }
                                }
                                echoMode: modelData.field === "password" ? TextInput.Password : TextInput.Normal
                            }
                        }
                    }
                }

                Item { Layout.preferredHeight: 12 }

                Rectangle {
                    Layout.fillWidth: true
                    height: 84
                    color: saveMouse.pressed ? "#333333" : "#2c2c2c"
                    border.color: "#000000"; border.width: 3

                    Text {
                        anchors.centerIn: parent
                        text: "Save Account"
                        font.pixelSize: 33; font.bold: true; color: "#ffffff"
                    }

                    MouseArea {
                        id: saveMouse; anchors.fill: parent
                        onClicked: {
                            sendRequestFunc("add_account", {
                                "display_name": displayName,
                                "email": emailAddr,
                                "imap_host": imapHost,
                                "imap_port": parseInt(imapPort) || 993,
                                "smtp_host": smtpHost,
                                "smtp_port": parseInt(smtpPort) || 587,
                                "username": username,
                                "password": password
                            }, function(resp) {
                                if (resp.data && resp.data.success) appState.currentView = "account_list"
                            })
                        }
                    }
                }
            }
        }
    }
}
