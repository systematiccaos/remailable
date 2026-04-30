/// CXX-Qt bridge module for the remailable app.
///
/// Defines QObjects that connect the Rust backend (Storage, IMAP/SMTP, Sync)
/// to the QML frontend for account management, folder navigation, email browsing,
/// email reading, and sync status display.
use std::pin::Pin;

use crate::account::{AccountConfig, EmailMetadata};
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
        #[qproperty(QString, selected_folder)]
        #[qproperty(QString, selected_email_id)]
        #[qproperty(QString, active_account_name)]
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

    // -----------------------------------------------------------------------
    // FolderListModel — folder navigation (READ-01)
    // -----------------------------------------------------------------------
    extern "RustQt" {
        /// Folder list model — provides folder navigation for the active account.
        #[qobject]
        #[qml_element]
        #[qproperty(i32, folder_count)]
        type FolderListModel = super::FolderListModelRust;
    }

    extern "RustQt" {
        /// Refresh folder list from storage for the given account
        #[qinvokable]
        fn refresh_folders(self: Pin<&mut FolderListModel>, account_id: &QString);

        /// Get folder name at index
        #[qinvokable]
        fn get_folder_name(self: &FolderListModel, index: i32) -> QString;

        /// Select folder at index — sets AppModel.selected_folder and navigates
        #[qinvokable]
        fn select_folder(self: Pin<&mut FolderListModel>, index: i32);
    }

    // -----------------------------------------------------------------------
    // EmailListModel — email list browsing (READ-02, READ-05, READ-06)
    // -----------------------------------------------------------------------
    extern "RustQt" {
        /// Email list model — provides email browsing within a folder.
        #[qobject]
        #[qml_element]
        #[qproperty(i32, email_count)]
        #[qproperty(QString, current_folder)]
        type EmailListModel = super::EmailListModelRust;
    }

    extern "RustQt" {
        /// Refresh email list from storage for the given account+folder
        #[qinvokable]
        fn refresh_emails(self: Pin<&mut EmailListModel>, account_id: &QString, folder: &QString);

        /// Get email ID at index
        #[qinvokable]
        fn get_email_id(self: &EmailListModel, index: i32) -> QString;

        /// Get email subject at index
        #[qinvokable]
        fn get_email_subject(self: &EmailListModel, index: i32) -> QString;

        /// Get email from address at index
        #[qinvokable]
        fn get_email_from(self: &EmailListModel, index: i32) -> QString;

        /// Get email date at index
        #[qinvokable]
        fn get_email_date(self: &EmailListModel, index: i32) -> QString;

        /// Get whether email at index is read
        #[qinvokable]
        fn get_email_read(self: &EmailListModel, index: i32) -> bool;

        /// Get whether email at index has attachments
        #[qinvokable]
        fn get_email_has_attachments(self: &EmailListModel, index: i32) -> bool;

        /// Get email thread ID at index
        #[qinvokable]
        fn get_email_thread_id(self: &EmailListModel, index: i32) -> QString;

        /// Toggle read/unread status of email at index
        #[qinvokable]
        fn toggle_email_read(self: Pin<&mut EmailListModel>, index: i32);
    }

    // -----------------------------------------------------------------------
    // EmailReaderModel — email body reading + thread view (READ-03, READ-04, READ-05, READ-06)
    // -----------------------------------------------------------------------
    extern "RustQt" {
        /// Email reader model — loads and displays a single email's content and thread.
        #[qobject]
        #[qml_element]
        #[qproperty(QString, email_subject)]
        #[qproperty(QString, email_from)]
        #[qproperty(QString, email_date)]
        #[qproperty(QString, email_body)]
        #[qproperty(QString, email_content_type)]
        #[qproperty(QString, email_thread_id)]
        #[qproperty(bool, email_is_read)]
        #[qproperty(bool, email_has_attachments)]
        type EmailReaderModel = super::EmailReaderModelRust;
    }

    extern "RustQt" {
        /// Load an email by ID — reads body from disk, sets all properties, marks as read
        #[qinvokable]
        fn load_email(self: Pin<&mut EmailReaderModel>, email_id: &QString);

        /// Load all emails for a thread by thread_id
        #[qinvokable]
        fn load_thread(self: Pin<&mut EmailReaderModel>, thread_id: &QString);

        /// Get number of emails in the current thread
        #[qinvokable]
        fn get_thread_count(self: &EmailReaderModel) -> i32;

        /// Get thread email ID at index
        #[qinvokable]
        fn get_thread_email_id(self: &EmailReaderModel, index: i32) -> QString;

        /// Get thread email subject at index
        #[qinvokable]
        fn get_thread_email_subject(self: &EmailReaderModel, index: i32) -> QString;

        /// Get thread email date at index
        #[qinvokable]
        fn get_thread_email_date(self: &EmailReaderModel, index: i32) -> QString;
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
    selected_folder: cxx_qt_lib::QString,
    selected_email_id: cxx_qt_lib::QString,
    active_account_name: cxx_qt_lib::QString,
}

/// Rust struct backing the AccountListModel QObject.
#[derive(Default)]
pub struct AccountListModelRust {
    account_count: i32,
    /// Internal cache of accounts loaded from storage (not a QProperty)
    accounts: Vec<AccountConfig>,
}

/// Rust struct backing the FolderListModel QObject.
#[derive(Default)]
pub struct FolderListModelRust {
    folder_count: i32,
    /// Internal cache of folder names loaded from storage
    folders: Vec<String>,
}

/// Rust struct backing the EmailListModel QObject.
#[derive(Default)]
pub struct EmailListModelRust {
    email_count: i32,
    current_folder: cxx_qt_lib::QString,
    /// Internal cache of emails loaded from storage
    emails: Vec<EmailMetadata>,
}

/// Rust struct backing the EmailReaderModel QObject.
#[derive(Default)]
pub struct EmailReaderModelRust {
    email_subject: cxx_qt_lib::QString,
    email_from: cxx_qt_lib::QString,
    email_date: cxx_qt_lib::QString,
    email_body: cxx_qt_lib::QString,
    email_content_type: cxx_qt_lib::QString,
    email_thread_id: cxx_qt_lib::QString,
    email_is_read: bool,
    email_has_attachments: bool,
    /// Internal cache of thread emails
    thread_emails: Vec<EmailMetadata>,
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

// ---------------------------------------------------------------------------
// FolderListModel implementations
// ---------------------------------------------------------------------------

impl qobject::FolderListModel {
    /// Load folders from the database and update folder_count.
    pub fn refresh_folders(self: Pin<&mut Self>, account_id: &cxx_qt_lib::QString) {
        let account_id = qstring_to_string(account_id);
        let folders = STORAGE
            .lock()
            .expect("Storage mutex poisoned")
            .list_folders(&account_id)
            .unwrap_or_default();
        let count = folders.len() as i32;
        let mut rust = self.rust_mut();
        rust.folders = folders;
        rust.folder_count = count;
    }

    /// Get the folder name at the given index.
    pub fn get_folder_name(self: &Self, index: i32) -> cxx_qt_lib::QString {
        let rust = self.rust();
        if index < 0 || index as usize >= rust.folders.len() {
            return string_to_qstring("");
        }
        string_to_qstring(&rust.folders[index as usize])
    }

    /// Select a folder at the given index — returns the folder name for QML to consume.
    /// QML should read get_folder_name(index) and set appModel.selected_folder accordingly.
    /// This method validates the index; QML handles navigation state.
    pub fn select_folder(self: Pin<&mut Self>, index: i32) {
        let rust = self.rust();
        if index < 0 || index as usize >= rust.folders.len() {
            return;
        }
        // QML will call get_folder_name(index) and set appModel.selected_folder
        let _folder_name = &rust.folders[index as usize];
    }
}

// ---------------------------------------------------------------------------
// EmailListModel implementations
// ---------------------------------------------------------------------------

impl qobject::EmailListModel {
    /// Load emails from the database and update email_count.
    pub fn refresh_emails(
        self: Pin<&mut Self>,
        account_id: &cxx_qt_lib::QString,
        folder: &cxx_qt_lib::QString,
    ) {
        let account_id = qstring_to_string(account_id);
        let folder = qstring_to_string(folder);
        let emails = STORAGE
            .lock()
            .expect("Storage mutex poisoned")
            .list_emails_by_folder(&account_id, &folder)
            .unwrap_or_default();
        let count = emails.len() as i32;
        let mut rust = self.rust_mut();
        rust.emails = emails;
        rust.email_count = count;
        rust.current_folder = string_to_qstring(&folder);
    }

    /// Get the email ID at the given index.
    pub fn get_email_id(self: &Self, index: i32) -> cxx_qt_lib::QString {
        let rust = self.rust();
        if index < 0 || index as usize >= rust.emails.len() {
            return string_to_qstring("");
        }
        string_to_qstring(&rust.emails[index as usize].id)
    }

    /// Get the email subject at the given index.
    pub fn get_email_subject(self: &Self, index: i32) -> cxx_qt_lib::QString {
        let rust = self.rust();
        if index < 0 || index as usize >= rust.emails.len() {
            return string_to_qstring("");
        }
        string_to_qstring(&rust.emails[index as usize].subject)
    }

    /// Get the email from address at the given index.
    pub fn get_email_from(self: &Self, index: i32) -> cxx_qt_lib::QString {
        let rust = self.rust();
        if index < 0 || index as usize >= rust.emails.len() {
            return string_to_qstring("");
        }
        string_to_qstring(&rust.emails[index as usize].from_addr)
    }

    /// Get the email date at the given index.
    pub fn get_email_date(self: &Self, index: i32) -> cxx_qt_lib::QString {
        let rust = self.rust();
        if index < 0 || index as usize >= rust.emails.len() {
            return string_to_qstring("");
        }
        string_to_qstring(&rust.emails[index as usize].date)
    }

    /// Get whether the email at the given index is read.
    pub fn get_email_read(self: &Self, index: i32) -> bool {
        let rust = self.rust();
        if index < 0 || index as usize >= rust.emails.len() {
            return true; // Default to read for out-of-bounds
        }
        rust.emails[index as usize].read
    }

    /// Get whether the email at the given index has attachments.
    pub fn get_email_has_attachments(self: &Self, index: i32) -> bool {
        let rust = self.rust();
        if index < 0 || index as usize >= rust.emails.len() {
            return false;
        }
        rust.emails[index as usize].has_attachments
    }

    /// Get the email thread ID at the given index.
    pub fn get_email_thread_id(self: &Self, index: i32) -> cxx_qt_lib::QString {
        let rust = self.rust();
        if index < 0 || index as usize >= rust.emails.len() {
            return string_to_qstring("");
        }
        string_to_qstring(&rust.emails[index as usize].thread_id)
    }

    /// Toggle the read/unread status of the email at the given index.
    pub fn toggle_email_read(self: Pin<&mut Self>, index: i32) {
        let email_id;
        let new_read;
        {
            let rust = self.rust();
            if index < 0 || index as usize >= rust.emails.len() {
                return;
            }
            email_id = rust.emails[index as usize].id.clone();
            new_read = !rust.emails[index as usize].read;
        }
        if STORAGE
            .lock()
            .expect("Storage mutex poisoned")
            .mark_email_read(&email_id, new_read)
            .is_ok()
        {
            // Update the local cache
            let mut rust = self.rust_mut();
            let idx = index as usize;
            rust.emails[idx].read = new_read;
        }
    }
}

// ---------------------------------------------------------------------------
// EmailReaderModel implementations
// ---------------------------------------------------------------------------

impl qobject::EmailReaderModel {
    /// Load an email by ID — sets all properties, reads body from disk, marks as read.
    pub fn load_email(self: Pin<&mut Self>, email_id: &cxx_qt_lib::QString) {
        let email_id = qstring_to_string(email_id);
        let email = match STORAGE
            .lock()
            .expect("Storage mutex poisoned")
            .get_email(&email_id)
        {
            Ok(Some(e)) => e,
            _ => return,
        };

        // Read body from disk
        let body = {
            let base = dirs::data_dir().unwrap_or_else(|| std::path::PathBuf::from("."));
            let body_path = base.join(&email.body_path);
            std::fs::read_to_string(&body_path).unwrap_or_default()
        };

        // Auto-mark as read
        let _ = STORAGE
            .lock()
            .expect("Storage mutex poisoned")
            .mark_email_read(&email_id, true);

        let mut rust = self.rust_mut();
        rust.email_subject = string_to_qstring(&email.subject);
        rust.email_from = string_to_qstring(&email.from_addr);
        rust.email_date = string_to_qstring(&email.date);
        rust.email_body = string_to_qstring(&body);
        rust.email_content_type = string_to_qstring(&email.content_type);
        rust.email_thread_id = string_to_qstring(&email.thread_id);
        rust.email_is_read = true; // We just marked it read
        rust.email_has_attachments = email.has_attachments;
    }

    /// Load all emails in a thread by thread_id.
    pub fn load_thread(self: Pin<&mut Self>, thread_id: &cxx_qt_lib::QString) {
        let thread_id = qstring_to_string(thread_id);
        let thread_emails = STORAGE
            .lock()
            .expect("Storage mutex poisoned")
            .list_thread(&thread_id)
            .unwrap_or_default();
        let mut rust = self.rust_mut();
        rust.thread_emails = thread_emails;
    }

    /// Get the number of emails in the current thread.
    pub fn get_thread_count(self: &Self) -> i32 {
        self.rust().thread_emails.len() as i32
    }

    /// Get the thread email ID at the given index.
    pub fn get_thread_email_id(self: &Self, index: i32) -> cxx_qt_lib::QString {
        let rust = self.rust();
        if index < 0 || index as usize >= rust.thread_emails.len() {
            return string_to_qstring("");
        }
        string_to_qstring(&rust.thread_emails[index as usize].id)
    }

    /// Get the thread email subject at the given index.
    pub fn get_thread_email_subject(self: &Self, index: i32) -> cxx_qt_lib::QString {
        let rust = self.rust();
        if index < 0 || index as usize >= rust.thread_emails.len() {
            return string_to_qstring("");
        }
        string_to_qstring(&rust.thread_emails[index as usize].subject)
    }

    /// Get the thread email date at the given index.
    pub fn get_thread_email_date(self: &Self, index: i32) -> cxx_qt_lib::QString {
        let rust = self.rust();
        if index < 0 || index as usize >= rust.thread_emails.len() {
            return string_to_qstring("");
        }
        string_to_qstring(&rust.thread_emails[index as usize].date)
    }
}