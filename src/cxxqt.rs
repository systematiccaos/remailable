/// CXX-Qt bridge module for the remailable app.
///
/// Defines the root QObject that serves as the application model.
/// Currently minimal — properties and methods will be added in future phases.
#[cxx_qt::bridge]
pub mod qobject {
    extern "RustQt" {
        // The root QObject for the application.
        // Will hold app state in future phases (accounts, folders, emails).
        #[qobject]
        #[qml_element]
        type AppModel = super::AppModelRust;
    }
}

/// Rust struct backing the AppModel QObject.
/// Empty for now — will gain fields as features are implemented.
#[derive(Default)]
pub struct AppModelRust;