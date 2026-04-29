/// CXX-Qt bridge module for the remailable app.
///
/// Defines QObjects that connect the Rust backend (Storage, IMAP/SMTP, Sync)
/// to the QML frontend for account management and sync status display.
use std::pin::Pin;

use crate::account::AccountConfig;
use crate::imap_conn;
use crate::smtp_conn;
use crate::storage::Storage;
use crate::sync::SyncEngine;

#[cxx_qt::bridge]
pub mod qobject {
    unsafe extern "C++" {
        include!("cxx-qt-lib/qstring.h");
        type QString = cxx_qt_lib::QString;
    }

    extern "RustQt" {
        /// Root application model — holds navigation state, sync status, and validation state.
        #[qobject]
        #[qml_element]
        #[qproperty(QString, current_view)]
        #[qproperty(QString, active_account_id)]
        #[qproperty(QString, sync_status_text)]
        #[qproperty(QString, validation_status)]
        #[qproperty(QString, validation_error)]
        type AppModel = super::AppModelRust;
    }

    extern "RustQt" {
        /// Account list model — manages account CRUD and connection validation.
        #[qobject]
        #[qml_element]
        #[qproperty(i32, account_count)]
        type AccountListModel = super::AccountListModelRust;
    }

    // AccountListModel invokables
    extern "RustQt" {
        /// Load accounts from the database into the model
        #[qinvokable]
        fn refresh_accounts(self: Pin<&mut AccountListModel>);

        /// Add a new account (validates IMAP/SMTP first)
        #[qinvokable]
        fn add_account(
            self: Pin<&mut AccountListModel>,
            display_name: &QString,
            imap_host: &QString,
            imap_port: i32,
            username: &QString,
            password: &QString,
            smtp_host: &QString,
            smtp_port: i32,
        ) -> bool;

        /// Remove an account by ID
        #[qinvokable]
        fn remove_account(self: Pin<&mut AccountListModel>, account_id: &QString) -> bool;

        /// Validate IMAP and SMTP connection for the given credentials
        #[qinvokable]
        fn validate_connection(
            self: Pin<&mut AccountListModel>,
            imap_host: &QString,
            imap_port: i32,
            username: &QString,
            password: &QString,
            smtp_host: &QString,
            smtp_port: i32,
        ) -> bool;

        /// Trigger sync for all accounts
        #[qinvokable]
        fn sync_all(self: Pin<&mut AccountListModel>);

        /// Get account ID by index
        #[qinvokable]
        fn get_account_id(self: &AccountListModel, index: i32) -> QString;

        /// Get account display name by index
        #[qinvokable]
        fn get_account_display_name(self: &AccountListModel, index: i32) -> QString;

        /// Get account IMAP host by index
        #[qinvokable]
        fn get_account_imap_host(self: &AccountListModel, index: i32) -> QString;
    }
}

// ---------------------------------------------------------------------------
// Global Storage
// ---------------------------------------------------------------------------

/// Global storage handle — initialized once in main.rs before Qt starts.
///
/// Uses `once_cell::sync::Lazy` so the Storage is created on first access,
/// and `Mutex` for safe interior mutability from the CXX-Qt bridge methods.
pub static STORAGE: once_cell::sync::Lazy<std::sync::Mutex<Storage>> =
    once_cell::sync::Lazy::new(|| {
        let db_path = dirs::data_dir()
            .unwrap_or_else(|| std::path::PathBuf::from("."))
            .join("remailable")
            .join("remailable.db");

        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent).ok();
        }

        Mutex::new(
            Storage::open(db_path.to_str().unwrap_or("remailable.db"))
                .expect("Failed to open database"),
        )
    });

use std::sync::Mutex;

// ---------------------------------------------------------------------------
// Helper conversions
// ---------------------------------------------------------------------------

/// Convert a QString reference to a Rust String.
fn qstring_to_string(s: &cxx_qt_lib::QString) -> String {
    s.to_string()
}

/// Convert a Rust &str to a QString.
fn string_to_qstring(s: &str) -> cxx_qt_lib::QString {
    cxx_qt_lib::QString::from(s)
}

// ---------------------------------------------------------------------------
// Rust backing structs
// ---------------------------------------------------------------------------

/// Rust struct backing the AppModel QObject.
///
/// CXX-Qt 0.8 requires backing struct fields to match the types declared
/// in #[qproperty]. For QString properties, the Rust field type is QString.
#[derive(Default)]
pub struct AppModelRust {
    current_view: cxx_qt_lib::QString,
    active_account_id: cxx_qt_lib::QString,
    sync_status_text: cxx_qt_lib::QString,
    validation_status: cxx_qt_lib::QString,
    validation_error: cxx_qt_lib::QString,
}

/// Rust struct backing the AccountListModel QObject.
#[derive(Default)]
pub struct AccountListModelRust {
    account_count: i32,
    /// Internal cache of accounts loaded from storage (not a QProperty)
    accounts: Vec<AccountConfig>,
}

// ---------------------------------------------------------------------------
// AccountListModel implementations
// ---------------------------------------------------------------------------

use cxx_qt::CxxQtType;

impl qobject::AccountListModel {
    /// Load accounts from the database and update account_count.
    pub fn refresh_accounts(self: Pin<&mut Self>) {
        let accounts = STORAGE.lock().expect("Storage mutex poisoned").list_accounts().unwrap_or_default();
        let count = accounts.len() as i32;
        let mut rust = self.rust_mut();
        rust.accounts = accounts;
        rust.account_count = count;
    }

    /// Add a new account. Validates IMAP/SMTP first, then saves if IMAP succeeds.
    /// Returns true if the account was saved.
    pub fn add_account(
        self: Pin<&mut Self>,
        display_name: &cxx_qt_lib::QString,
        imap_host: &cxx_qt_lib::QString,
        imap_port: i32,
        username: &cxx_qt_lib::QString,
        password: &cxx_qt_lib::QString,
        smtp_host: &cxx_qt_lib::QString,
        smtp_port: i32,
    ) -> bool {
        let display_name = qstring_to_string(display_name);
        let imap_host = qstring_to_string(imap_host);
        let username = qstring_to_string(username);
        let password = qstring_to_string(password);
        let smtp_host = qstring_to_string(smtp_host);

        let config = AccountConfig::new(
            display_name,
            imap_host,
            imap_port as u16,
            username,
            password,
            smtp_host,
            smtp_port as u16,
        );

        // Validate IMAP — must succeed to save
        match imap_conn::validate_imap(&config) {
            Ok(imap_conn::ConnectionResult::Success) => {}
            _ => return false,
        }

        // Validate SMTP — log but don't block saving
        let _ = smtp_conn::validate_smtp(&config);

        // Save to storage
        match STORAGE.lock().expect("Storage mutex poisoned").save_account(&config) {
            Ok(()) => {
                self.refresh_accounts();
                true
            }
            Err(_) => false,
        }
    }

    /// Remove an account by ID. Returns true if removed.
    pub fn remove_account(self: Pin<&mut Self>, account_id: &cxx_qt_lib::QString) -> bool {
        let id = qstring_to_string(account_id);
        match STORAGE.lock().expect("Storage mutex poisoned").delete_account(&id) {
            Ok(()) => {
                self.refresh_accounts();
                true
            }
            Err(_) => false,
        }
    }

    /// Validate IMAP and SMTP connections for the given credentials.
    /// Returns true if both succeed.
    pub fn validate_connection(
        self: Pin<&mut Self>,
        imap_host: &cxx_qt_lib::QString,
        imap_port: i32,
        username: &cxx_qt_lib::QString,
        password: &cxx_qt_lib::QString,
        smtp_host: &cxx_qt_lib::QString,
        smtp_port: i32,
    ) -> bool {
        let imap_host = qstring_to_string(imap_host);
        let username = qstring_to_string(username);
        let password = qstring_to_string(password);
        let smtp_host = qstring_to_string(smtp_host);

        let config = AccountConfig::new(
            "Validation Test".into(),
            imap_host,
            imap_port as u16,
            username,
            password,
            smtp_host,
            smtp_port as u16,
        );

        // Validate IMAP
        let imap_ok = matches!(
            imap_conn::validate_imap(&config),
            Ok(imap_conn::ConnectionResult::Success)
        );

        // Validate SMTP
        let smtp_ok = matches!(
            smtp_conn::validate_smtp(&config),
            Ok(imap_conn::ConnectionResult::Success)
        );

        imap_ok && smtp_ok
    }

    /// Trigger sync for all accounts.
    pub fn sync_all(self: Pin<&mut Self>) {
        let accounts = STORAGE.lock().expect("Storage mutex poisoned").list_accounts().unwrap_or_default();

        // Sync needs mutable access to storage, so we lock it
        let _results = {
            let storage = STORAGE.lock().expect("Storage mutex poisoned");
            let engine = SyncEngine::new(&storage);
            // Note: This is a blocking call on the main thread.
            // A future phase should move this to a background thread.
            engine.sync_all_accounts(&accounts)
        };
    }

    /// Get the account ID at the given index.
    pub fn get_account_id(self: &Self, index: i32) -> cxx_qt_lib::QString {
        let rust = self.rust();
        if index < 0 || index as usize >= rust.accounts.len() {
            return string_to_qstring("");
        }
        string_to_qstring(&rust.accounts[index as usize].id)
    }

    /// Get the account display name at the given index.
    pub fn get_account_display_name(self: &Self, index: i32) -> cxx_qt_lib::QString {
        let rust = self.rust();
        if index < 0 || index as usize >= rust.accounts.len() {
            return string_to_qstring("");
        }
        string_to_qstring(&rust.accounts[index as usize].display_name)
    }

    /// Get the account IMAP host at the given index.
    pub fn get_account_imap_host(self: &Self, index: i32) -> cxx_qt_lib::QString {
        let rust = self.rust();
        if index < 0 || index as usize >= rust.accounts.len() {
            return string_to_qstring("");
        }
        string_to_qstring(&rust.accounts[index as usize].imap_host)
    }
}