use cxx_qt_build::{CxxQtBuilder, QmlModule};

fn main() {
    // Skip CXX-Qt build for backend-only builds
    // Set REMAILABLE_NO_QT=1 to build without Qt (for the headless backend)
    if std::env::var("REMAILABLE_NO_QT").is_ok() {
        println!("cargo:rerun-if-env-changed=REMAILABLE_NO_QT");
        return;
    }

    CxxQtBuilder::new_qml_module(
        QmlModule::new("io.remailable.Remailable")
            .qml_file("qml/main.qml")
            .qml_file("qml/AccountSettings.qml")
            .qml_file("qml/AccountList.qml")
            .qml_file("qml/SyncIndicator.qml")
            .qml_file("qml/FolderList.qml")
            .qml_file("qml/EmailList.qml")
            .qml_file("qml/EmailReader.qml")
            .qml_file("qml/SearchBar.qml")
            .qml_file("qml/AttachmentList.qml"),
    )
    .files(["src/cxxqt.rs"])
    .build();
}