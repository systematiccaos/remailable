pub mod cxxqt;
pub mod account;
pub mod storage;

use cxx_qt::casting::Upcast;
use cxx_qt_lib::{QGuiApplication, QQmlApplicationEngine, QQmlEngine, QUrl};

fn main() {
    // Initialize local storage
    let db_path = dirs::data_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("remailable")
        .join("remailable.db");

    // Ensure the data directory exists
    if let Some(parent) = db_path.parent() {
        std::fs::create_dir_all(parent).ok();
    }

    match storage::Storage::open(db_path.to_str().unwrap_or("remailable.db")) {
        Ok(_db) => println!("remailable: database opened at {:?}", db_path),
        Err(e) => eprintln!("remailable: failed to open database: {}", e),
    }

    // Create the Qt application and QML engine
    let mut app = QGuiApplication::new();
    let mut engine = QQmlApplicationEngine::new();

    // Load the main QML file (bundled via qrc by CXX-Qt build)
    if let Some(engine) = engine.as_mut() {
        engine.load(&QUrl::from("qrc:/qt/qml/io.remailable.Remailable/qml/main.qml"));
    }

    // Connect to the QML engine quit signal
    if let Some(engine) = engine.as_mut() {
        let engine: core::pin::Pin<&mut QQmlEngine> = engine.upcast_pin();
        engine.on_quit(|_| {
            println!("remailable: QML engine quit");
        }).release();
    }

    // Start the Qt event loop
    if let Some(app) = app.as_mut() {
        app.exec();
    }
}