pub mod cxxqt;
pub mod account;
pub mod storage;
pub mod imap_conn;
pub mod smtp_conn;
pub mod sync;

use cxx_qt::casting::Upcast;
use cxx_qt_lib::{QGuiApplication, QQmlApplicationEngine, QQmlEngine, QUrl};

fn main() {
    // Force Storage initialization by accessing the Lazy static.
    // This ensures the database is open before the Qt event loop starts.
    drop(cxxqt::STORAGE.lock().expect("Failed to initialize storage"));

    // Create the Qt application and QML engine
    let mut app = QGuiApplication::new();
    let mut engine = QQmlApplicationEngine::new();

    // Load the main QML file (bundled via qrc by CXX-Qt build)
    if let Some(engine) = engine.as_mut() {
        engine.load(&QUrl::from("qrc:/qt/qml/io/remailable/Remailable/qml/main.qml"));
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