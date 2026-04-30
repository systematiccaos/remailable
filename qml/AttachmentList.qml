import QtQuick 2.17
import QtQuick.Layouts 1.17
import io.remailable.Remailable 1.0

// Attachment list component for viewing and downloading email attachments.
// E-ink optimized: large fonts, high contrast, 44px+ touch targets.
Item {
    id: attachmentList
    height: childrenRect.height

    property string emailId: ""
    property bool hasAttachments: emailReaderModel.email_has_attachments

    ColumnLayout {
        id: column
        anchors.left: parent.left
        anchors.right: parent.right
        spacing: 4

        // Attachments header
        Rectangle {
            Layout.fillWidth: true
            height: 36
            color: "#f0f0f0"
            border.color: "transparent"

            Text {
                anchors.verticalCenter: parent.verticalCenter
                anchors.left: parent.left
                anchors.leftMargin: 12
                text: "Attachments (" + attachmentListModel.attachment_count + ")"
                font.pixelSize: 18
                font.bold: true
                color: "#333333"
            }
        }

        // Attachment list
        ListView {
            id: attachmentListView
            Layout.fillWidth: true
            Layout.preferredHeight: Math.min(attachmentListModel.attachment_count * 80, 320)
            clip: true
            model: attachmentListModel.attachment_count

            delegate: Rectangle {
                width: attachmentListView.width
                height: 80
                color: ma.pressed ? "#f0f0f0" : "#ffffff"
                border.color: "transparent"

                RowLayout {
                    anchors.fill: parent
                    anchors.margins: 12
                    spacing: 8

                    // Attachment info column
                    ColumnLayout {
                        Layout.fillWidth: true
                        spacing: 2

                        // Filename (bold)
                        Text {
                            text: attachmentListModel.get_attachment_filename(index)
                            font.pixelSize: 16
                            font.bold: true
                            color: "#000000"
                            elide: Text.ElideRight
                            Layout.fillWidth: true
                            maximumLineCount: 1
                        }

                        // Size + content type row
                        RowLayout {
                            spacing: 8
                            Layout.fillWidth: true

                            Text {
                                text: formatSize(attachmentListModel.get_attachment_size(index))
                                font.pixelSize: 14
                                color: "#666666"
                            }

                            Text {
                                text: attachmentListModel.get_attachment_content_type(index)
                                font.pixelSize: 14
                                color: "#666666"
                                elide: Text.ElideRight
                                Layout.fillWidth: true
                                maximumLineCount: 1
                            }
                        }

                        // Download status
                        Text {
                            text: attachmentListModel.is_attachment_downloaded(index)
                                ? "\u2713 Downloaded"
                                : "Not downloaded"
                            font.pixelSize: 14
                            color: attachmentListModel.is_attachment_downloaded(index) ? "#006600" : "#996600"
                        }
                    }

                    // Action buttons column
                    ColumnLayout {
                        spacing: 4

                        // Download button
                        Rectangle {
                            width: 100
                            height: 36
                            color: downloadMouse.pressed ? "#cccccc" : "#e0e0e0"
                            border.color: "#999999"
                            border.width: 1
                            radius: 4

                            Text {
                                anchors.centerIn: parent
                                text: attachmentListModel.is_attachment_downloaded(index) ? "Open" : "Download"
                                font.pixelSize: 14
                                color: "#000000"
                            }

                            MouseArea {
                                id: downloadMouse
                                anchors.fill: parent
                                onClicked: {
                                    attachmentListModel.download_attachment(index)
                                }
                            }
                        }

                        // View button (PDF only)
                        Rectangle {
                            visible: isPdf(index)
                            width: 100
                            height: 36
                            color: viewMouse.pressed ? "#cccccc" : "#e0e0e0"
                            border.color: "#999999"
                            border.width: 1
                            radius: 4

                            Text {
                                anchors.centerIn: parent
                                text: "View PDF"
                                font.pixelSize: 14
                                color: "#000000"
                            }

                            MouseArea {
                                id: viewMouse
                                anchors.fill: parent
                                onClicked: {
                                    var filePath = attachmentListModel.download_attachment(index)
                                    if (filePath.length > 0) {
                                        appModel.current_view = "pdf_view"
                                        pdfView.filePath = filePath
                                    }
                                }
                            }
                        }
                    }
                }

                MouseArea {
                    id: ma
                    anchors.fill: parent
                    // Only consume clicks not on buttons
                }
            }
        }
    }

    // Format file size as human-readable (KB/MB)
    function formatSize(bytes) {
        if (bytes < 0) return "0 B"
        if (bytes < 1024) return bytes + " B"
        if (bytes < 1048576) return (bytes / 1024).toFixed(1) + " KB"
        return (bytes / 1048576).toFixed(1) + " MB"
    }

    // Check if attachment is a PDF
    function isPdf(index) {
        var ct = attachmentListModel.get_attachment_content_type(index)
        return ct === "application/pdf"
    }

    Component.onCompleted: {
        if (emailId.length > 0) {
            attachmentListModel.load_attachments(emailId)
        }
    }

    onEmailIdChanged: {
        if (emailId.length > 0) {
            attachmentListModel.load_attachments(emailId)
        }
    }
}