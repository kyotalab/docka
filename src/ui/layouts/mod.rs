//! Layout management module for UI components.
//!
//! This module provides layout managers for organizing UI components
//! and handling responsive design for different terminal sizes.
//!
//! UIコンポーネント用レイアウト管理モジュール。
//! UIコンポーネントの整理と異なるターミナルサイズでの
//! レスポンシブデザイン処理のためのレイアウトマネージャーを提供します。
//!
//! # Architecture
//!
//! The layout system is designed to provide flexible and responsive layouts
//! that adapt to different terminal sizes while maintaining usability.
//!
//! レイアウトシステムは、使いやすさを維持しながら異なるターミナルサイズに
//! 適応する柔軟でレスポンシブなレイアウトを提供するよう設計されています。
//!
//! # Usage
//!
//! ```rust
//! use docka::ui::layouts::{SimpleLayout, LayoutAreas};
//! use ratatui::layout::Rect;
//!
//! // Calculate standard layout
//! let terminal_area = Rect::new(0, 0, 80, 24);
//! let layout = SimpleLayout::calculate(terminal_area);
//!
//! // Use responsive layout for smaller terminals
//! let small_area = Rect::new(0, 0, 40, 8);
//! let responsive_layout = SimpleLayout::calculate_responsive(small_area);
//!
//! // Check if help should be shown
//! if SimpleLayout::should_show_help(terminal_area) {
//!     // Render help area
//! }
//! ```

/// Simple 3-section vertical layout implementation.
///
/// This module provides a basic layout manager that divides the terminal
/// into three vertical sections: main content, status bar, and help line.
///
/// 3セクション縦分割レイアウトのシンプル実装。
/// ターミナルを3つの縦セクション（メインコンテンツ、ステータスバー、ヘルプライン）
/// に分割する基本レイアウトマネージャーを提供します。
pub mod simple_layout;

// Re-export commonly used types for convenient access
// 便利なアクセスのためによく使用される型を再エクスポート

/// Simple layout manager for 3-section vertical layout
/// 3セクション縦分割レイアウトのシンプルレイアウトマネージャー
pub use simple_layout::SimpleLayout;

/// Layout areas structure for organizing terminal space
/// ターミナルスペースを整理するためのレイアウトエリア構造体
pub use simple_layout::LayoutAreas;
