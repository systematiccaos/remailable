import QtQuick 2.17
import QtQuick.Layouts 1.17
import QtQuick.Controls 2.17
import io.remailable.Remailable 1.0

Item {
    id: emailReader

    ColumnLayout {
        anchors.fill: parent
        spacing: 0

        // Header: back button + subject
        Rectangle {
            Layout.fillWidth: true
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
                        onClicked: appModel.current_view = "email_list"
                    }
                }

                Text {
                    text: emailReaderModel.email_subject
                    font.pixelSize: 24
                    font.bold: true
                    elide: Text.ElideRight
                    Layout.fillWidth: true
                    maximumLineCount: 1
                }
            }
        }

        // Email meta: from + date
        Rectangle {
            Layout.fillWidth: true
            height: 48
            color: "#ffffff"
            border.bottom: "grey"

            RowLayout {
                anchors.fill: parent
                anchors.margins: 16

                Text {
                    text: emailReaderModel.email_from
                    font.pixelSize: 18
                    Layout.fillWidth: true
                    elide: Text.ElideRight
                }

                Text {
                    text: emailReaderModel.email_date
                    font.pixelSize: 14
                    color: "#666666"
                }
            }
        }

        // Email body — dual mode: HTML via TextArea with RichText, plain text via TextArea
        // Using TextArea with textFormat: TextEdit.RichText as universal approach for e-ink
        // since QtWebView may not be available on reMarkable Paper Pro
        ScrollView {
            Layout.fillWidth: true
            Layout.fillHeight: true
            Layout.minimumHeight: 200
            clip: true

            TextArea {
                id: bodyText
                readOnly: true
                wrapMode: TextEdit.Wrap
                font.pixelSize: 20
                color: "#000000"
                selectionColor: "#cccccc"
                selectedTextColor: "#000000"
                text: emailReaderModel.email_body
                textFormat: {
                    // Use RichText for HTML emails, PlainText for plain text
                    if (emailReaderModel.email_content_type === "text/html") {
                        return TextEdit.RichText
                    } else {
                        return TextEdit.PlainText
                    }
                }
                background: Rectangle {
                    color: "#ffffff"
                    border.color: "#cccccc"
                    border.width: 1
                }
            }
        }

        // Thread section at bottom (READ-06)
        Rectangle {
            Layout.fillWidth: true
            height: threadSection.height + 16
            color: "#f0f0f0"
            border.top: "grey"
            visible: emailReaderModel.get_thread_count() > 1

            ColumnLayout {
                id: threadSection
                anchors.left: parent.left
                anchors.right: parent.right
                anchors.margins: 8
                spacing: 4

                Text {
                    text: "Thread (" + emailReaderModel.get_thread_count() + ")"
                    font.pixelSize: 18
                    font.bold: true
                    color: "#333333"
                }

                ListView {
                    Layout.fillWidth: true
                    Layout.preferredHeight: Math.min(emailReaderModel.get_thread_count() * 44, 176)
                    clip: true
                    model: emailReaderModel.get_thread_count()

                    delegate: Rectangle {
                        width: ListView.view.width
                        height: 44
                        color: threadMa.pressed ? "#f0f0f0" : "#ffffff"
                        border.bottom: "grey"

                        RowLayout {
                            anchors.fill: parent
                            anchors.margins: 8

                            Text {
                                text: emailReaderModel.get_thread_email_date(index)
                                font.pixelSize: 14
                                color: "#666666"
                            }

                            Text {
                                text: emailReaderModel.get_thread_email_subject(index)
                                font.pixelSize: 16
                                elide: Text.ElideRight
                                Layout.fillWidth: true
                                maximumLineCount: 1
                            }
                        }

                        MouseArea {
                            id: threadMa
                            anchors.fill: parent
                            onClicked: {
                                var tid = emailReaderModel.get_thread_email_id(index)
                                emailReaderModel.load_email(tid)
                            }
                        }
                    }
                }
            }
        }
    }

    Component.onCompleted: {
        emailReaderModel.load_email(appModel.selected_email_id)
        emailReaderModel.load_thread(emailReaderModel.email_thread_id)
    }
}