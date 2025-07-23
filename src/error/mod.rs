// src/error/mod.rs
// Error handling module for docka application
// dockaアプリケーション用エラーハンドリングモジュール

//! Error handling module for docka application.
//!
//! This module provides comprehensive error types and utilities for handling
//! various failure scenarios in Docker operations, UI rendering, and system interactions.
//!
//! dockaアプリケーション用エラーハンドリングモジュール。
//! Docker操作、UIレンダリング、システム相互作用における
//! 様々な失敗シナリオを処理するための包括的なエラー型とユーティリティを提供します。

/// Application-specific error types and Result type alias.
///
/// Contains the main `DockaError` enum with all error variants used throughout
/// the application, along with convenient constructors and utility methods.
///
/// アプリケーション固有のエラー型とResult型エイリアス。
/// `アプリケーション全体で使用される全エラーバリアントを含むメインのDockaError列挙型`、
/// 便利なコンストラクタとユーティリティメソッドを含みます。
pub mod app_error;

// Re-export main types for convenient access
// 便利なアクセスのためにメイン型を再エクスポート

/// Main error type for docka operations.
/// docka操作用メインエラー型。
pub use app_error::DockaError;

/// Convenient Result type alias for docka operations.
/// docka操作用便利なResult型エイリアス。
pub use app_error::DockaResult;
