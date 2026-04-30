/// CXX-Qt bridge module for the remailable app.
///
/// Defines QObjects that connect the Rust backend (Storage, IMAP/SMTP, Sync)
/// to the QML frontend for account management, folder navigation, email browsing,
/// email reading, and sync status display.
use std::pin::Pin;

use crate::account::{AccountConfig, AttachmentMetadata, EmailMetadata};
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
            email: &QString,
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
    // EmailListModel — email list browsing (READ-02, READ-05, READ-06, READ-07)
    // -----------------------------------------------------------------------
    extern "RustQt" {
        /// Email list model — provides email browsing within a folder, with search and thread modes.
        #[qobject]
        #[qml_element]
        #[qproperty(i32, email_count)]
        #[qproperty(QString, current_folder)]
        #[qproperty(bool, is_searching)]
        #[qproperty(bool, thread_mode)]
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

        /// Search emails by subject or sender for the current account
        #[qinvokable]
        fn search_emails(self: Pin<&mut EmailListModel>, query: &QString);

        /// Clear search results and return to normal folder view
        #[qinvokable]
        fn clear_search(self: Pin<&mut EmailListModel>);

        /// Refresh emails grouped by thread for the given account+folder
        #[qinvokable]
        fn refresh_threaded(self: Pin<&mut EmailListModel>, account_id: &QString, folder: &QString);
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
        #[qproperty(QString, attachment_email_id)]
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

        /// Switch to HTML body rendering
        #[qinvokable]
        fn show_html(self: Pin<&mut EmailReaderModel>);

        /// Switch to plain text body rendering
        #[qinvokable]
        fn show_plain_text(self: Pin<&mut EmailReaderModel>);
    }

    // -----------------------------------------------------------------------
    // AttachmentListModel — attachment browsing and download (ATCH-01, ATCH-02, ATCH-03)
    // -----------------------------------------------------------------------
    extern "RustQt" {
        /// Attachment list model — provides attachment metadata and download for an email.
        #[qobject]
        #[qml_element]
        #[qproperty(i32, attachment_count)]
        type AttachmentListModel = super::AttachmentListModelRust;
    }

    extern "RustQt" {
        /// Load attachment metadata for the given email_id
        #[qinvokable]
        fn load_attachments(self: Pin<&mut AttachmentListModel>, email_id: &QString);

        /// Get attachment filename at index
        #[qinvokable]
        fn get_attachment_filename(self: &AttachmentListModel, index: i32) -> QString;

        /// Get attachment size in bytes at index
        #[qinvokable]
        fn get_attachment_size(self: &AttachmentListModel, index: i32) -> i32;

        /// Get attachment content type at index
        #[qinvokable]
        fn get_attachment_content_type(self: &AttachmentListModel, index: i32) -> QString;

        /// Download attachment at index to local device storage, returns file path
        #[qinvokable]
        fn download_attachment(self: Pin<&mut AttachmentListModel>, index: i32) -> QString;

        /// Check if attachment at index is already downloaded
        #[qinvokable]
        fn is_attachment_downloaded(self: &AttachmentListModel, index: i32) -> bool;
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
    is_searching: bool,
    thread_mode: bool,
    /// Internal cache of emails loaded from storage
    emails: Vec<EmailMetadata>,
    /// Internal cache of account_id for search/refresh operations
    account_id: String,
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
    attachment_email_id: cxx_qt_lib::QString,
    /// Internal cache of thread emails
    thread_emails: Vec<EmailMetadata>,
}

/// Rust struct backing the AttachmentListModel QObject.
#[derive(Default)]
pub struct AttachmentListModelRust {
    attachment_count: i32,
    /// Internal cache of attachments loaded from storage
    attachments: Vec<AttachmentMetadata>,
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
        email: &cxx_qt_lib::QString,
        imap_host: &cxx_qt_lib::QString,
        imap_port: i32,
        username: &cxx_qt_lib::QString,
        password: &cxx_qt_lib::QString,
        smtp_host: &cxx_qt_lib::QString,
        smtp_port: i32,
    ) -> bool {
        let display_name = qstring_to_string(display_name);
        let email = qstring_to_string(email);
        let imap_host = qstring_to_string(imap_host);
        let username = qstring_to_string(username);
        let password = qstring_to_string(password);
        let smtp_host = qstring_to_string(smtp_host);

        let config = AccountConfig::new(
            display_name,
            email,
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
            "".into(),
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
    /// Also stores account_id internally for search/refresh operations.
    pub fn refresh_emails(
        self: Pin<&mut Self>,
        account_id: &cxx_qt_lib::QString,
        folder: &cxx_qt_lib::QString,
    ) {
        let account_id_str = qstring_to_string(account_id);
        let folder = qstring_to_string(folder);
        let emails = STORAGE
            .lock()
            .expect("Storage mutex poisoned")
            .list_emails_by_folder(&account_id_str, &folder)
            .unwrap_or_default();
        let count = emails.len() as i32;
        let mut rust = self.rust_mut();
        rust.emails = emails;
        rust.email_count = count;
        rust.current_folder = string_to_qstring(&folder);
        rust.account_id = account_id_str;
        rust.is_searching = false;
        rust.thread_mode = false;
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

    /// Search emails by subject or sender for the current account.
    pub fn search_emails(self: Pin<&mut Self>, query: &cxx_qt_lib::QString) {
        let query_str = qstring_to_string(query);
        let account_id = {
            let rust = self.rust();
            rust.account_id.clone()
        };

        let emails = STORAGE
            .lock()
            .expect("Storage mutex poisoned")
            .search_emails(&account_id, &query_str)
            .unwrap_or_default();
        let count = emails.len() as i32;
        let mut rust = self.rust_mut();
        rust.emails = emails;
        rust.email_count = count;
        rust.is_searching = true;
    }

    /// Clear search results and return to normal folder view.
    pub fn clear_search(self: Pin<&mut Self>) {
        let (account_id, current_folder) = {
            let rust = self.rust();
            (rust.account_id.clone(), qstring_to_string(&rust.current_folder))
        };

        let emails = STORAGE
            .lock()
            .expect("Storage mutex poisoned")
            .list_emails_by_folder(&account_id, &current_folder)
            .unwrap_or_default();
        let count = emails.len() as i32;
        let mut rust = self.rust_mut();
        rust.emails = emails;
        rust.email_count = count;
        rust.is_searching = false;
    }

    /// Load emails grouped by thread for the selected folder.
    /// Flattens thread groups with thread headers into a single list.
    pub fn refresh_threaded(
        self: Pin<&mut Self>,
        account_id: &cxx_qt_lib::QString,
        folder: &cxx_qt_lib::QString,
    ) {
        let account_id_str = qstring_to_string(account_id);
        let folder = qstring_to_string(folder);
        // Get flat list for the folder — threading is visual in QML via thread_id
        let emails = STORAGE
            .lock()
            .expect("Storage mutex poisoned")
            .list_emails_by_folder(&account_id_str, &folder)
            .unwrap_or_default();
        let count = emails.len() as i32;
        let mut rust = self.rust_mut();
        rust.emails = emails;
        rust.email_count = count;
        rust.current_folder = string_to_qstring(&folder);
        rust.account_id = account_id_str;
        rust.thread_mode = true;
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
        rust.attachment_email_id = string_to_qstring(&email_id);
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

    /// Switch to HTML body rendering.
    pub fn show_html(self: Pin<&mut Self>) {
        self.rust_mut().email_content_type = string_to_qstring("text/html");
    }

    /// Switch to plain text body rendering.
    pub fn show_plain_text(self: Pin<&mut Self>) {
        self.rust_mut().email_content_type = string_to_qstring("text/plain");
    }
}

// ---------------------------------------------------------------------------
// AttachmentListModel implementations
// ---------------------------------------------------------------------------

impl qobject::AttachmentListModel {
    /// Load attachments for the given email_id from storage.
    pub fn load_attachments(self: Pin<&mut Self>, email_id: &cxx_qt_lib::QString) {
        let email_id = qstring_to_string(email_id);
        let attachments = STORAGE
            .lock()
            .expect("Storage mutex poisoned")
            .list_attachments(&email_id)
            .unwrap_or_default();
        let count = attachments.len() as i32;
        let mut rust = self.rust_mut();
        rust.attachments = attachments;
        rust.attachment_count = count;
    }

    /// Get the attachment filename at the given index.
    pub fn get_attachment_filename(self: &Self, index: i32) -> cxx_qt_lib::QString {
        let rust = self.rust();
        if index < 0 || index as usize >= rust.attachments.len() {
            return string_to_qstring("");
        }
        string_to_qstring(&rust.attachments[index as usize].filename)
    }

    /// Get the attachment size in bytes at the given index.
    pub fn get_attachment_size(self: &Self, index: i32) -> i32 {
        let rust = self.rust();
        if index < 0 || index as usize >= rust.attachments.len() {
            return 0;
        }
        rust.attachments[index as usize].size as i32
    }

    /// Get the attachment content type at the given index.
    pub fn get_attachment_content_type(self: &Self, index: i32) -> cxx_qt_lib::QString {
        let rust = self.rust();
        if index < 0 || index as usize >= rust.attachments.len() {
            return string_to_qstring("");
        }
        string_to_qstring(&rust.attachments[index as usize].content_type)
    }

    /// Download an attachment at the given index to local device storage.
    /// Returns the download file path as a QString, or empty string on failure.
    pub fn download_attachment(self: Pin<&mut Self>, index: i32) -> cxx_qt_lib::QString {
        let (filename, email_id, account_id, att_id, _size);
        {
            let rust = self.rust();
            if index < 0 || index as usize >= rust.attachments.len() {
                return string_to_qstring("");
            }
            let att = &rust.attachments[index as usize];
            filename = att.filename.clone();
            email_id = att.email_id.clone();
            account_id = att.account_id.clone();
            att_id = att.id.clone();
            _size = att.size;
        }

        // Source path: attachments/{account_id}/{email_id}/{filename} in data dir
        let base = dirs::data_dir().unwrap_or_else(|| std::path::PathBuf::from("."));
        let source_path = base.join("remailable")
            .join("attachments")
            .join(&account_id)
            .join(&email_id)
            .join(&filename);

        if !source_path.exists() {
            // Attachment file not yet synced to disk — return empty string
            return string_to_qstring("");
        }

        // Destination path: remailable/downloads/{email_id}/{filename}
        let dest_dir = base.join("remailable").join("downloads").join(&email_id);
        if let Err(_) = std::fs::create_dir_all(&dest_dir) {
            return string_to_qstring("");
        }
        let dest_path = dest_dir.join(&filename);

        // Copy the file (use copy so original stays in attachments dir)
        if std::fs::copy(&source_path, &dest_path).is_err() {
            return string_to_qstring("");
        }

        let download_path = format!(
            "remailable/downloads/{}/{}",
            email_id, filename
        );

        // Mark as downloaded in the database
        let local_path = download_path.clone();
        let _ = STORAGE
            .lock()
            .expect("Storage mutex poisoned")
            .mark_downloaded(&att_id, &local_path);

        // Update the local cache's downloaded flag
        {
            let mut rust = self.rust_mut();
            if index >= 0 && (index as usize) < rust.attachments.len() {
                rust.attachments[index as usize].downloaded = true;
                rust.attachments[index as usize].local_path = download_path;
            }
        }

        string_to_qstring(&format!("{}", dest_path.display()))
    }

    /// Check if the attachment at the given index has been downloaded.
    pub fn is_attachment_downloaded(self: &Self, index: i32) -> bool {
        let rust = self.rust();
        if index < 0 || index as usize >= rust.attachments.len() {
            return false;
        }
        rust.attachments[index as usize].downloaded
    }
}