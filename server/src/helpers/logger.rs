//! # カスタムロガー
//! 
//! ヘッダーにステータスを付与してログを出力
//! 
//! ## 関数
//! 
//! - `log`: ログ出力
//! 
//! ## マクロ
//! 
//! - `app_log`:     以下ログを出力するための土台　※直接的には使用しない
//! - `success_log`: 成功ログ
//! - `info_log`:    情報ログ
//! - `warning_log`: 警告ログ
//! - `error_log`:   エラーログ
//! - `debug_log`:   デバッグログ

use chrono::Local;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::Path;
use std::sync::mpsc::{self, Sender};
use std::thread;
use lazy_static::lazy_static;

lazy_static! {
    pub static ref LOG_PATH: String = format!("/server/log/actix.log");
}

lazy_static! {
    pub static ref LOGGER: Logger = Logger::init();
}

/// Header 列挙子
#[derive(Debug, Clone)]
pub enum Header {
    SUCCESS,
    INFO,
    WARNING,
    ERROR,
    DEBUG,
}

impl Header {
    fn as_str(&self) -> &'static str {
        match self {
            Header::SUCCESS => "SUCCESS",
            Header::INFO => "INFO",
            Header::WARNING => "WARNING",
            Header::ERROR => "ERROR",
            Header::DEBUG => "DEBUG",
        }
    }
}

pub struct Logger {
    sender: Sender<(Header, String)>
}

impl Logger {
    /// ログスレッド初期化
    pub fn init() -> Self {
        let (sender, receiver) = mpsc::channel::<(Header, String)>();

        thread::spawn(move || {
            while let Ok((header, message)) = receiver.recv() {
                if let Err(e) = Logger::write_log(header, message) {
                    eprintln!("Failed to write log: {}", e);
                }
            }
        }); 

        Self { sender }
    }

    /// ログをキューに追加
    pub fn log(&self, header: Header, message: &str) {
        if let Err(e) = self.sender.send((header, message.to_string())) {
            eprintln!("Failed to send log message: {}", e);
        }
    }

    /// ログ書き込み
    fn write_log(header: Header, message: String) -> std::io::Result<()> {
        let log_path = Path::new(&*LOG_PATH);

        if let Some(parent_dir) = log_path.parent() {
            if !parent_dir.exists() {
                std::fs::create_dir_all(parent_dir)?;
            }
        }

        let log_message = format!(
            "[{}] {} {}",
            Local::now().format("%m-%d-%Y %H:%M:%S"),
            header.as_str(),
            message
        );

        let mut log_file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&*LOG_PATH)?;

        writeln!(log_file, "{}", log_message)?;
        Ok(())
    }
}

#[macro_export]
macro_rules! app_log {
    ($header:expr, $msg:expr, $($arg:tt)*) => {
        let formatted_message = format!($msg, $($arg)*);
        crate::helpers::logger::LOGGER.log($header, &formatted_message);
    };
    ($header:expr, $msg:expr) => {
        crate::helpers::logger::LOGGER.log($header, $msg);
    };
}

#[macro_export]
macro_rules! success_log {
    ($msg:expr, $($arg:tt)*) => {
        app_log!(crate::helpers::logger::Header::SUCCESS, $msg, $($arg)*);
    };
    ($msg:expr) => {
        app_log!(crate::helpers::logger::Header::SUCCESS, $msg);
    };
}

#[macro_export]
macro_rules! info_log {
    ($msg:expr, $($arg:tt)*) => {
        app_log!(crate::helpers::logger::Header::INFO, $msg, $($arg)*);
    };
    ($msg:expr) => {
        app_log!(crate::helpers::logger::Header::INFO, $msg);
    };
}

#[macro_export]
macro_rules! warning_log {
    ($msg:expr, $($arg:tt)*) => {
        app_log!(crate::helpers::logger::Header::WARNING, $msg, $($arg)*);
    };
    ($msg:expr) => {
        app_log!(crate::helpers::logger::Header::WARNING, $msg);
    };
}

#[macro_export]
macro_rules! error_log {
    ($msg:expr, $($arg:tt)*) => {
        app_log!(crate::helpers::logger::Header::ERROR, $msg, $($arg)*);
    };
    ($msg:expr) => {
        app_log!(crate::helpers::logger::Header::ERROR, $msg);
    };
}

#[macro_export]
macro_rules! debug_log {
    ($msg:expr, $($arg:tt)*) => {
        app_log!(crate::helpers::logger::Header::DEBUG, $msg, $($arg)*);
    };
    ($msg:expr) => {
        app_log!(crate::helpers::logger::Header::DEBUG, $msg);
    };
}