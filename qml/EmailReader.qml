import QtQuick 2.17
import QtQuick.Layouts 1.17
import QtQuick.Controls 2.17
import io.remailable.Remailable 1.0

Item {
    id: emailReader

    // Internal state for HTML/plain text toggle
    property bool showingHtml: emailReaderModel.email_content_type === "text/html"

    ColumnLayout {
        anchors.fill: parent
        spacing: 0

        // Header: back button + subject + HTML toggle
        Rectangle {
            Layout.fillWidth: true
            height: 80
            color: "#ffffff"
            border.color: "transparent"

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

                // HTML/Plain text toggle button (only visible when email is HTML)
                Rectangle {
                    visible: emailReaderModel.email_content_type === "text/html" || emailReaderModel.email_content_type.length > 0
                    height: 44
                    width: 70
                    color: htmlToggleMouse.pressed ? "#cccccc" : (showingHtml ? "#d0d0d0" : "#e0e0e0")
                    border.color: "#999999"
                    border.width: 1
                    radius: 4

                    Text {
                        anchors.centerIn: parent
                        text: showingHtml ? "Plain" : "HTML"
                        font.pixelSize: 14
                        font.bold: true
                    }

                    MouseArea {
                        id: htmlToggleMouse
                        anchors.fill: parent
                        onClicked: {
                            if (showingHtml) {
                                emailReaderModel.show_plain_text()
                                showingHtml = false
                            } else {
                                emailReaderModel.show_html()
                                showingHtml = true
                            }
                        }
                    }
                }
            }
        }

        // Email meta: from + date
        Rectangle {
            Layout.fillWidth: true
            height: 48
            color: "#ffffff"
            border.color: "transparent"

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
                textFormat: showingHtml ? TextEdit.RichText : TextEdit.PlainText
                background: Rectangle {
                    color: "#ffffff"
                    border.color: "#cccccc"
                    border.width: 1
                }
            }
        }

        // Attachment section
        AttachmentList {
            Layout.fillWidth: true
            emailId: emailReaderModel.attachment_email_id
            hasAttachments: emailReaderModel.email_has_attachments
            visible: emailReaderModel.email_has_attachments
        }

        // Thread section at bottom
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
                        border.color: "transparent"

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

    // PDF view state
    Item {
        id: pdfView
        property string filePath: ""

        visible: appModel.current_view === "pdf_view"
        anchors.fill: parent

        ColumnLayout {
            anchors.fill: parent
            spacing: 0

            // PDF header with back button
            Rectangle {
                Layout.fillWidth: true
                height: 60
                color: "#ffffff"
                border.color: "transparent"

                RowLayout {
                    anchors.fill: parent
                    anchors.margins: 16

                    Rectangle {
                        height: 44
                        width: 80
                        color: pdfBackMouse.pressed ? "#cccccc" : "#e0e0e0"
                        border.color: "#999999"
                        border.width: 1
                        radius: 4

                        Text {
                            anchors.centerIn: parent
                            text: "\u2190 Back"
                            font.pixelSize: 16
                        }

                        MouseArea {
                            id: pdfBackMouse
                            anchors.fill: parent
                            onClicked: appModel.current_view = "email_reader"
                        }
                    }

                    Text {
                        text: "PDF Viewer"
                        font.pixelSize: 22
                        font.bold: true
                        Layout.fillWidth: true
                    }
                }
            }

            // PDF content area — fallback message for reMarkable Paper Pro
            // Qt.labs.pdf may not be available on the device, so we provide
            // a message with the file path for the user to open in system viewer.
            Rectangle {
                Layout.fillWidth: true
                Layout.fillHeight: true
                color: "#ffffff"

                ColumnLayout {
                    anchors.centerIn: parent
                    spacing: 16

                    Text {
                        text: "\uD83D\uDCD5"
                        font.pixelSize: 48
                        Layout.alignment: Qt.AlignHCenter
                    }

                    Text {
                        text: "PDF Saved to Device"
                        font.pixelSize: 28
                        font.bold: true
                        Layout.alignment: Qt.AlignHCenter
                    }

                    Text {
                        text: pdfView.filePath.length > 0 ? pdfView.filePath : ""
                        font.pixelSize: 16
                        color: "#666666"
                        Layout.alignment: Qt.AlignHCenter
                        wrapMode: Text.Wrap
                        Layout.maximumWidth: 1200
                    }

                    Text {
                        text: "Open this file in the reMarkable document viewer."
                        font.pixelSize: 14
                        color: "#888888"
                        Layout.alignment: Qt.AlignHCenter
                    }
                }
            }
        }
    }

    Component.onCompleted: {
        emailReaderModel.load_email(appModel.selected_email_id)
        emailReaderModel.load_thread(emailReaderModel.email_thread_id)
        // Initialize HTML toggle state based on content type
        showingHtml = emailReaderModel.email_content_type === "text/html"
    }
}