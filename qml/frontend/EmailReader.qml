import QtQuick 2.15
import QtQuick.Layouts 1.15

Item {
    id: emailReader
    property var backend: null
    property var appState: null
    property var sendRequestFunc: null

    function setBackend(b) { emailReader.backend = b }
    function setAppState(s) { emailReader.appState = s }
    function setSendRequest(fn) {
        emailReader.sendRequestFunc = fn
        loadEmail()
    }

    property string emailSubject: ""
    property string emailFrom: ""
    property string emailDate: ""
    property string emailBody: ""

    function loadEmail() {
        if (sendRequestFunc && appState && appState.activeEmailId) {
            sendRequestFunc("get_email_body", {"id": appState.activeEmailId}, function(resp) {
                var data = resp.data ? resp.data : resp
                if (data.subject !== undefined) {
                    emailSubject = data.subject || "(no subject)"
                    emailFrom = data.from || ""
                    emailDate = data.date || ""
                    emailBody = data.body || ""
                }
            })
        }
    }

    ColumnLayout {
        anchors.fill: parent
        anchors.topMargin: 48
        spacing: 0

        // Header
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
                    MouseArea { id: backMouse; anchors.fill: parent; onClicked: appState.currentView = "email_list" }
                }

                Text {
                    text: emailSubject
                    font.pixelSize: 20
                    font.bold: true
                    Layout.fillWidth: true
                    elide: Text.ElideRight
                }
            }
        }

        // Email headers
        Rectangle {
            Layout.fillWidth: true
            height: 60
            color: "#f5f5f5"
            border.color: "#eeeeee"
            border.width: 1

            ColumnLayout {
                anchors.fill: parent
                anchors.margins: 12
                spacing: 2

                Text {
                    text: "From: " + emailFrom
                    font.pixelSize: 14
                    color: "#333333"
                }

                Text {
                    text: "Date: " + emailDate
                    font.pixelSize: 14
                    color: "#666666"
                }
            }
        }

        // Email body
        Flickable {
            Layout.fillWidth: true
            Layout.fillHeight: true
            clip: true

            contentWidth: parent.width
            contentHeight: bodyText.height + 24

            Text {
                id: bodyText
                width: parent.width - 24
                x: 12
                y: 12
                text: emailBody
                font.pixelSize: 16
                wrapMode: Text.WordWrap
                color: "#222222"
            }
        }
    }
}