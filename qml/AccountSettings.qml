import QtQuick 2.17
import QtQuick.Layouts 1.17
import io.remailable.Remailable 1.0

Item {
    id: accountSettings

    // Back button + header
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
                text: "Add Account"
                font.pixelSize: 28
                font.bold: true
                Layout.fillWidth: true
            }
        }
    }

    // Form fields
    ColumnLayout {
        id: form
        anchors.top: header.bottom
        anchors.bottom: parent.bottom
        anchors.left: parent.left
        anchors.right: parent.right
        anchors.margins: 24
        spacing: 20

        // Display Name
        ColumnLayout {
            spacing: 4
            Text { text: "Account Name"; font.pixelSize: 16; color: "#666666" }
            Rectangle {
                height: 48; Layout.fillWidth: true
                color: "#ffffff"; border.color: "#cccccc"; border.width: 1; radius: 4
                TextInput {
                    id: displayNameField
                    anchors.fill: parent; anchors.margins: 8
                    font.pixelSize: 20
                    clip: true
                }
            }
        }

        // IMAP Host
        ColumnLayout {
            spacing: 4
            Text { text: "IMAP Server"; font.pixelSize: 16; color: "#666666" }
            Rectangle {
                height: 48; Layout.fillWidth: true
                color: "#ffffff"; border.color: "#cccccc"; border.width: 1; radius: 4
                TextInput {
                    id: imapHostField
                    anchors.fill: parent; anchors.margins: 8
                    font.pixelSize: 20; clip: true
                    text: "imap.gmail.com"
                }
            }
        }

        // IMAP Port
        ColumnLayout {
            spacing: 4
            Text { text: "IMAP Port"; font.pixelSize: 16; color: "#666666" }
            Rectangle {
                height: 48; Layout.fillWidth: true
                color: "#ffffff"; border.color: "#cccccc"; border.width: 1; radius: 4
                TextInput {
                    id: imapPortField
                    anchors.fill: parent; anchors.margins: 8
                    font.pixelSize: 20; clip: true
                    text: "993"
                    validator: IntValidator { bottom: 1; top: 65535 }
                }
            }
        }

        // Username (email address)
        ColumnLayout {
            spacing: 4
            Text { text: "Email Address (Username)"; font.pixelSize: 16; color: "#666666" }
            Rectangle {
                height: 48; Layout.fillWidth: true
                color: "#ffffff"; border.color: "#cccccc"; border.width: 1; radius: 4
                TextInput {
                    id: usernameField
                    anchors.fill: parent; anchors.margins: 8
                    font.pixelSize: 20; clip: true
                }
            }
        }

        // Password
        ColumnLayout {
            spacing: 4
            Text { text: "Password"; font.pixelSize: 16; color: "#666666" }
            Rectangle {
                height: 48; Layout.fillWidth: true
                color: "#ffffff"; border.color: "#cccccc"; border.width: 1; radius: 4
                TextInput {
                    id: passwordField
                    anchors.fill: parent; anchors.margins: 8
                    font.pixelSize: 20; clip: true
                    echoMode: TextInput.Password
                }
            }
        }

        // SMTP Host
        ColumnLayout {
            spacing: 4
            Text { text: "SMTP Server"; font.pixelSize: 16; color: "#666666" }
            Rectangle {
                height: 48; Layout.fillWidth: true
                color: "#ffffff"; border.color: "#cccccc"; border.width: 1; radius: 4
                TextInput {
                    id: smtpHostField
                    anchors.fill: parent; anchors.margins: 8
                    font.pixelSize: 20; clip: true
                    text: "smtp.gmail.com"
                }
            }
        }

        // SMTP Port
        ColumnLayout {
            spacing: 4
            Text { text: "SMTP Port"; font.pixelSize: 16; color: "#666666" }
            Rectangle {
                height: 48; Layout.fillWidth: true
                color: "#ffffff"; border.color: "#cccccc"; border.width: 1; radius: 4
                TextInput {
                    id: smtpPortField
                    anchors.fill: parent; anchors.margins: 8
                    font.pixelSize: 20; clip: true
                    text: "465"
                    validator: IntValidator { bottom: 1; top: 65535 }
                }
            }
        }

        // Validation status
        Text {
            id: validationText
            font.pixelSize: 16
            color: {
                switch (appModel.validation_status) {
                    case "success": return "#333333"
                    case "error": return "#cc0000"
                    case "validating": return "#666666"
                    default: return "transparent"
                }
            }
            text: {
                switch (appModel.validation_status) {
                    case "success": return "\u2713 Connection validated"
                    case "error": return "\u2717 " + appModel.validation_error
                    case "validating": return "\u25C6 Validating..."
                    default: return ""
                }
            }
        }

        // Buttons: Validate + Save
        RowLayout {
            Layout.fillWidth: true
            spacing: 20

            Rectangle {
                height: 56; Layout.fillWidth: true
                color: validateMouse.pressed ? "#cccccc" : "#e0e0e0"
                border.color: "#999999"; border.width: 1; radius: 4

                Text {
                    anchors.centerIn: parent
                    text: "Validate Connection"
                    font.pixelSize: 18; font.bold: true
                }

                MouseArea {
                    id: validateMouse
                    anchors.fill: parent
                    onClicked: {
                        appModel.validation_status = "validating";
                        var result = accountListModel.validate_connection(
                            imapHostField.text,
                            parseInt(imapPortField.text) || 993,
                            usernameField.text,
                            passwordField.text,
                            smtpHostField.text,
                            parseInt(smtpPortField.text) || 465
                        );
                        appModel.validation_status = result ? "success" : "error";
                        if (!result) {
                            appModel.validation_error = "Connection failed. Check your settings.";
                        } else {
                            appModel.validation_error = "";
                        }
                    }
                }
            }

            Rectangle {
                height: 56; Layout.fillWidth: true
                color: saveMouse.pressed ? "#cccccc" : "#e0e0e0"
                border.color: "#999999"; border.width: 1; radius: 4

                Text {
                    anchors.centerIn: parent
                    text: "Save Account"
                    font.pixelSize: 18; font.bold: true
                }

                MouseArea {
                    id: saveMouse
                    anchors.fill: parent
                    onClicked: {
                        var saved = accountListModel.add_account(
                            displayNameField.text || usernameField.text,
                            imapHostField.text,
                            parseInt(imapPortField.text) || 993,
                            usernameField.text,
                            passwordField.text,
                            smtpHostField.text,
                            parseInt(smtpPortField.text) || 465
                        );
                        if (saved) {
                            appModel.current_view = "account_list";
                            accountListModel.refresh_accounts();
                        }
                    }
                }
            }
        }
    }
}