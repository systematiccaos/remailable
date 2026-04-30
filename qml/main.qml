import QtQuick 2.17
import QtQuick.Window 2.17
import QtQuick.Layouts 1.17

// Import the CXX-Qt QML module defined in build.rs
import io.remailable.Remailable 1.0

Window {
    id: root
    visible: true
    width: 1620
    height: 2160
    color: "#ffffff"
    title: "remailable"

    // AppLoad required signals for reMarkable Paper Pro compatibility
    signal close
    function unloading() {
        // Cleanup — called by AppLoad before unloading the frontend.
        // Future phases will add state persistence here.
    }

    // Data models (instantiated by CXX-Qt)
    AppModel { id: appModel }
    AccountListModel { id: accountListModel }
    FolderListModel { id: folderListModel }
    EmailListModel { id: emailListModel }
    EmailReaderModel { id: emailReaderModel }

    ColumnLayout {
        anchors.fill: parent
        spacing: 0

        // Sync status indicator at top
        SyncIndicator {
            Layout.fillWidth: true
        }

        // Main content area with view switching
        Loader {
            id: contentLoader
            Layout.fillWidth: true
            Layout.fillHeight: true
            source: {
                switch (appModel.current_view) {
                    case "account_settings": return "AccountSettings.qml"
                    case "folder_list": return "FolderList.qml"
                    case "email_list": return "EmailList.qml"
                    case "email_reader": return "EmailReader.qml"
                    case "account_list":
                    default: return "AccountList.qml"
                }
            }
        }
    }

    Component.onCompleted: {
        appModel.current_view = "account_list"
        accountListModel.refresh_accounts()
    }
}