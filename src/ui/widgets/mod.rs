//! Widget implementations for TUI components.
//!
//! This module contains reusable TUI widgets for displaying containers,
//! status information, and other UI elements.
//!
//! TUIコンポーネント用ウィジェット実装。
//! コンテナ、ステータス情報、その他UI要素を表示するための
//! 再利用可能なTUIウィジェットを含みます。
//!
//! # Architecture
//!
//! The widget system is designed to provide modular, reusable components
//! that can be composed to create complex user interfaces.
//!
//! ウィジェットシステムは、複雑なユーザーインターフェースを作成するために
//! 組み合わせ可能なモジュラーで再利用可能なコンポーネントを提供するよう設計されています。
//!
//! # Usage
//!
//! ```rust
//! use docka::ui::widgets::StatusBar;
//! use ratatui::{backend::TestBackend, Terminal, layout::Rect};
//!
//! // Render status bar in a TUI application
//! let backend = TestBackend::new(80, 24);
//! let mut terminal = Terminal::new(backend).unwrap();
//!
//! terminal.draw(|f| {
//!     let area = Rect::new(0, 20, 80, 3);
//!     StatusBar::render(f, &app, area);
//! }).unwrap();
//! ```

/// Status bar widget for displaying application state and information.
///
/// This module provides a status bar widget that displays current application state,
/// container information, and contextual help messages.
///
/// ステータスバーウィジェット - アプリケーション状態と情報表示用。
/// 現在のアプリケーション状態、コンテナ情報、コンテキストヘルプメッセージを
/// 表示するステータスバーウィジェットを提供します。
pub mod status_bar;

// Phase 1.2.2 で実装予定
// To be implemented in Phase 1.2.2

// /// Container list widget for displaying Docker containers.
// ///
// /// This module provides a widget for displaying a list of Docker containers
// /// with navigation, status indicators, and selection highlighting.
// ///
// /// コンテナリストウィジェット - Dockerコンテナ表示用。
// /// ナビゲーション、ステータスインジケーター、選択ハイライト付きの
// /// Dockerコンテナリスト表示ウィジェットを提供します。
// pub mod container_list;

// Re-export commonly used types for convenient access
// 便利なアクセスのためによく使用される型を再エクスポート

/// Status bar widget for displaying application state and information
/// アプリケーション状態と情報を表示するステータスバーウィジェット
pub use status_bar::StatusBar;
