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

    // Form fields
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
        anchors.topMargin: 48
        anchors.margins: 16
        spacing: 12

        // Back button + title
        RowLayout {
            Layout.fillWidth: true

            Rectangle {
                width: 80
                height: 40
                color: backMouse.pressed ? "#cccccc" : "#e0e0e0"
                border.color: "#999999"
                border.width: 1
                radius: 4

                Text {
                    anchors.centerIn: parent
                    text: "← Back"
                    font.pixelSize: 16
                }

                MouseArea {
                    id: backMouse
                    anchors.fill: parent
                    onClicked: appState.currentView = "account_list"
                }
            }

            Text {
                text: "Add Account"
                font.pixelSize: 28
                font.bold: true
                Layout.fillWidth: true
            }
        }

        // Form
        ColumnLayout {
            Layout.fillWidth: true
            Layout.fillHeight: true
            spacing: 8

            Repeater {
                model: [
                    { label: "Display Name", field: "displayName", placeholder: "John Doe" },
                    { label: "Email", field: "emailAddr", placeholder: "john@example.com" },
                    { label: "IMAP Host", field: "imapHost", placeholder: "imap.example.com" },
                    { label: "IMAP Port", field: "imapPort", placeholder: "993" },
                    { label: "SMTP Host", field: "smtpHost", placeholder: "smtp.example.com" },
                    { label: "SMTP Port", field: "smtpPort", placeholder: "587" },
                    { label: "Username", field: "username", placeholder: "john@example.com" },
                    { label: "Password", field: "password", placeholder: "••••••••" }
                ]

                delegate: ColumnLayout {
                    width: parent.width
                    spacing: 2

                    Text {
                        text: modelData.label
                        font.pixelSize: 16
                        font.bold: true
                    }

                    Rectangle {
                        Layout.fillWidth: true
                        height: 44
                        color: "#ffffff"
                        border.color: "#cccccc"
                        border.width: 1
                        radius: 4

                        TextInput {
                            anchors.fill: parent
                            anchors.margins: 8
                            font.pixelSize: 18
                            text: {
                                switch (modelData.field) {
                                case "displayName": return displayName
                                case "emailAddr": return emailAddr
                                case "imapHost": return imapHost
                                case "imapPort": return imapPort
                                case "smtpHost": return smtpHost
                                case "smtpPort": return smtpPort
                                case "username": return username
                                case "password": return password
                                }
                            }
                            onTextChanged: {
                                switch (modelData.field) {
                                case "displayName": displayName = text; break
                                case "emailAddr": emailAddr = text; break
                                case "imapHost": imapHost = text; break
                                case "imapPort": imapPort = text; break
                                case "smtpHost": smtpHost = text; break
                                case "smtpPort": smtpPort = text; break
                                case "username": username = text; break
                                case "password": password = text; break
                                }
                            }
                            echoMode: modelData.field === "password" ? TextInput.Password : TextInput.Normal
                        }
                    }
                }
            }
        }

        // Save button
        Rectangle {
            Layout.fillWidth: true
            height: 56
            color: saveMouse.pressed ? "#aaaaaa" : "#333333"
            radius: 4

            Text {
                anchors.centerIn: parent
                text: "Save Account"
                font.pixelSize: 20
                font.bold: true
                color: "#ffffff"
            }

            MouseArea {
                id: saveMouse
                anchors.fill: parent
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
                        if (resp.data && resp.data.success) {
                            appState.currentView = "account_list"
                        }
                    })
                }
            }
        }
    }
}