import QtQuick 2.17
import QtQuick.Layouts 1.17
import io.remailable.Remailable 1.0

// Search bar component for filtering emails by subject or sender.
// E-ink optimized: large fonts, high contrast, 44px+ touch targets.
Item {
    id: searchBar
    height: 60

    property alias searchText: searchField.text
    property bool isSearching: emailListModel.is_searching

    RowLayout {
        anchors.fill: parent
        spacing: 8

        // Search text input
        Rectangle {
            Layout.fillWidth: true
            height: 44
            color: "#ffffff"
            border.color: "#999999"
            border.width: 1
            radius: 4

            TextInput {
                id: searchField
                anchors.fill: parent
                anchors.margins: 8
                font.pixelSize: 18
                color: "#000000"
                verticalAlignment: Text.AlignVCenter

                // Accept Return/Enter as search trigger
                onAccepted: {
                    if (text.length > 0) {
                        emailListModel.search_emails(text)
                    }
                }

                Text {
                    anchors.fill: parent
                    font.pixelSize: 18
                    color: "#999999"
                    text: "Search by subject or sender..."
                    visible: searchField.text.length === 0 && !searchField.activeFocus
                    verticalAlignment: Text.AlignVCenter
                }
            }
        }

        // Clear button (visible when text is entered or searching)
        Rectangle {
            visible: searchField.text.length > 0 || emailListModel.is_searching
            width: 44
            height: 44
            color: clearMouse.pressed ? "#cccccc" : "#e0e0e0"
            border.color: "#999999"
            border.width: 1
            radius: 4

            Text {
                anchors.centerIn: parent
                text: "\u2715"  // ✕
                font.pixelSize: 22
                font.bold: true
                color: "#333333"
            }

            MouseArea {
                id: clearMouse
                anchors.fill: parent
                onClicked: {
                    searchField.text = ""
                    emailListModel.clear_search()
                }
            }
        }

        // Search button
        Rectangle {
            width: 44
            height: 44
            color: searchMouse.pressed ? "#cccccc" : "#e0e0e0"
            border.color: "#999999"
            border.width: 1
            radius: 4

            Text {
                anchors.centerIn: parent
                text: "\uD83D\uDD0D"  // 🔍 magnifying glass
                font.pixelSize: 20
            }

            MouseArea {
                id: searchMouse
                anchors.fill: parent
                onClicked: {
                    if (searchField.text.length > 0) {
                        emailListModel.search_emails(searchField.text)
                    }
                }
            }
        }
    }
}