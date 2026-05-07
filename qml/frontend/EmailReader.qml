import QtQuick 2.15
import QtQuick.Layouts 1.15

Item {
    id: emailReader
    property var backend: null
    property var appState: null
    property var sendRequestFunc: null

    function setBackend(b) { emailReader.backend = b }
    function setAppState(s) { emailReader.appState = s }
    function setSendRequest(fn) { emailReader.sendRequestFunc = fn; loadEmail() }

    property string emailSubject: ""
    property string emailFrom: ""
    property string emailDate: ""
    property string emailBody: ""
    property string emailContentType: "text/plain"

    function loadEmail() {
        if (sendRequestFunc && appState && appState.activeEmailId) {
            sendRequestFunc("get_email_body", {
                "id": appState.activeEmailId,
                "account_id": appState.activeAccountId || "",
                "folder": appState.activeFolder || "INBOX"
            }, function(resp) {
                var data = resp.data ? resp.data : resp
                if (data.subject !== undefined) {
                    emailSubject = data.subject || "(no subject)"
                    emailFrom = data.from || ""
                    emailDate = data.date || ""
                    emailBody = data.body || ""
                    emailContentType = (data.content_type || "text/plain").split(";")[0].trim()
                }
            })
        }
    }

    ColumnLayout {
        anchors.fill: parent
        anchors.topMargin: 84
        spacing: 0

        Rectangle {
            Layout.fillWidth: true
            height: 120
            color: "#faf6f0"

            RowLayout {
                anchors.fill: parent
                anchors.margins: 24
                spacing: 18

                Rectangle {
                    width: 144; height: 72
                    color: backMouse.pressed ? "#e8e4dc" : "#ffffff"
                    border.color: "#777777"; border.width: 3
                    Text { anchors.centerIn: parent; text: "\u2190 Back"; font.pixelSize: 30; font.bold: true; color: "#2c2c2c" }
                    MouseArea { id: backMouse; anchors.fill: parent; onClicked: appState.currentView = "email_list" }
                }

                Text {
                    text: emailSubject
                    font.pixelSize: 33; font.bold: true; color: "#2c2c2c"
                    Layout.fillWidth: true; elide: Text.ElideRight
                }
            }
        }

        Rectangle {
            Layout.fillWidth: true
            height: 108
            color: "#ffffff"
            border.color: "#e0dbd2"; border.width: 2

            ColumnLayout {
                anchors.fill: parent
                anchors.margins: 24
                spacing: 6

                Text {
                    text: "From: " + emailFrom
                    font.pixelSize: 30; color: "#2c2c2c"
                }
                Text {
                    text: "Date: " + emailDate
                    font.pixelSize: 27; color: "#7a7368"
                }
            }
        }

        Flickable {
            Layout.fillWidth: true
            Layout.fillHeight: true
            clip: true
            contentWidth: parent.width
            contentHeight: bodyText.height + 48

            Text {
                id: bodyText
                width: parent.width - 48
                x: 24; y: 24
                text: emailBody
                textFormat: emailContentType.indexOf("html") >= 0 ? Text.RichText : Text.PlainText
                font.pixelSize: 30
                wrapMode: Text.WordWrap
                color: "#2c2c2c"
                onLinkActivated: function(link) { Qt.openUrlExternally(link) }
            }
        }
    }
}
