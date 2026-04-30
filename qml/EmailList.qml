import QtQuick 2.17
import QtQuick.Layouts 1.17
import io.remailable.Remailable 1.0

Item {
    id: emailList

    ColumnLayout {
        anchors.fill: parent
        spacing: 0

        // Header: back button + folder name + unread count + thread toggle
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
                        onClicked: appModel.current_view = "folder_list"
                    }
                }

                Text {
                    text: appModel.selected_folder
                    font.pixelSize: 28
                    font.bold: true
                    Layout.fillWidth: true
                }

                // Thread/List toggle button
                Rectangle {
                    height: 44
                    width: 90
                    color: threadMouse.pressed ? "#cccccc" : (emailListModel.thread_mode ? "#d0d0d0" : "#e0e0e0")
                    border.color: "#999999"
                    border.width: 1
                    radius: 4

                    Text {
                        anchors.centerIn: parent
                        text: emailListModel.thread_mode ? "List" : "Threads"
                        font.pixelSize: 14
                        font.bold: emailListModel.thread_mode
                    }

                    MouseArea {
                        id: threadMouse
                        anchors.fill: parent
                        onClicked: {
                            if (emailListModel.thread_mode) {
                                emailListModel.refresh_emails(appModel.active_account_id, appModel.selected_folder)
                            } else {
                                emailListModel.refresh_threaded(appModel.active_account_id, appModel.selected_folder)
                            }
                        }
                    }
                }

                Text {
                    id: unreadCount
                    font.pixelSize: 18
                    color: "#666666"
                    text: {
                        var count = 0;
                        for (var i = 0; i < emailListModel.email_count; i++) {
                            if (!emailListModel.get_email_read(i)) count++;
                        }
                        return count > 0 ? count + " unread" : ""
                    }
                }
            }
        }

        // Search bar
        SearchBar {
            id: searchBar
            Layout.fillWidth: true
        }

        // Search status indicator
        Rectangle {
            visible: emailListModel.is_searching
            Layout.fillWidth: true
            height: 36
            color: "#f5f5e0"
            border.color: "transparent"

            RowLayout {
                anchors.fill: parent
                anchors.margins: 8

                Text {
                    text: "\uD83D\uDD0D Search results"
                    font.pixelSize: 14
                    font.italic: true
                    color: "#666666"
                    Layout.fillWidth: true
                }

                Rectangle {
                    height: 28
                    width: 70
                    color: clearSearchMouse.pressed ? "#cccccc" : "#e0e0e0"
                    border.color: "#999999"
                    border.width: 1
                    radius: 4

                    Text {
                        anchors.centerIn: parent
                        text: "Clear"
                        font.pixelSize: 12
                    }

                    MouseArea {
                        id: clearSearchMouse
                        anchors.fill: parent
                        onClicked: {
                            emailListModel.clear_search()
                            searchBar.searchText = ""
                        }
                    }
                }
            }
        }

        // Email list
        ListView {
            id: listView
            Layout.fillWidth: true
            Layout.fillHeight: true
            clip: true

            model: emailListModel.email_count

            delegate: Rectangle {
                width: listView.width
                height: 80
                color: {
                    if (ma.pressed) return "#f0f0f0"
                    if (!emailListModel.get_email_read(index)) return "#f0f5f0"  // Subtle grey-green tint for unread
                    return "#ffffff"
                }
                border.color: "transparent"

                RowLayout {
                    anchors.fill: parent
                    anchors.margins: 12
                    spacing: 8

                    // Thread indent for replies in thread mode
                    Item {
                        width: emailListModel.thread_mode ? 30 : 0
                        height: 1
                        visible: emailListModel.thread_mode
                    }

                    // Read/unread indicator (e-ink optimized: ● for unread, ○ for read)
                    Text {
                        text: emailListModel.get_email_read(index) ? "\u25CB" : "\u25CF"
                        font.pixelSize: 14
                        color: emailListModel.get_email_read(index) ? "#cccccc" : "#000000"
                        Layout.alignment: Qt.AlignVCenter

                        MouseArea {
                            anchors.fill: parent
                            onClicked: emailListModel.toggle_email_read(index)
                        }
                    }

                    // Subject + from + date column
                    ColumnLayout {
                        Layout.fillWidth: true
                        spacing: 2

                        // Subject line (single line, elided)
                        Text {
                            text: emailListModel.get_email_subject(index)
                            font.pixelSize: 20
                            font.bold: !emailListModel.get_email_read(index)
                            elide: Text.ElideRight
                            Layout.fillWidth: true
                            maximumLineCount: 1
                        }

                        // From + date row
                        RowLayout {
                            Layout.fillWidth: true
                            spacing: 8

                            Text {
                                text: emailListModel.get_email_from(index)
                                font.pixelSize: 14
                                color: "#666666"
                                elide: Text.ElideRight
                                Layout.fillWidth: true
                                maximumLineCount: 1
                            }

                            Text {
                                text: emailListModel.get_email_date(index)
                                font.pixelSize: 14
                                color: "#666666"
                            }
                        }

                        // Thread indicator (show reply count for thread parents in thread mode)
                        Text {
                            visible: emailListModel.thread_mode && emailListModel.get_email_thread_id(index).length > 0
                            text: "\u25B6 " + "thread"
                            font.pixelSize: 12
                            color: "#888888"
                        }
                    }

                    // Attachment indicator
                    Text {
                        text: emailListModel.get_email_has_attachments(index) ? "\uD83D\uDCCE" : ""
                        font.pixelSize: 16
                        color: "#666666"
                        visible: emailListModel.get_email_has_attachments(index)
                        Layout.alignment: Qt.AlignVCenter
                    }
                }

                MouseArea {
                    id: ma
                    anchors.fill: parent
                    onClicked: {
                        appModel.selected_email_id = emailListModel.get_email_id(index)
                        appModel.current_view = "email_reader"
                    }
                }
            }
        }
    }

    Component.onCompleted: emailListModel.refresh_emails(appModel.active_account_id, appModel.selected_folder)
}