// src/ui/mod.rs
// User interface module for TUI components
// TUIコンポーネント用ユーザーインターフェースモジュール

//! User interface module providing TUI components and event handling.
//!
//! This module contains all TUI-related functionality including application state
//! management, event processing, widgets, and layout management using the ratatui framework.
//!
//! TUIコンポーネントとイベント処理を提供するユーザーインターフェースモジュール。
//! ratatuiフレームワークを使用したアプリケーション状態管理、イベント処理、
//! ウィジェット、レイアウト管理を含む全TUI関連機能を含みます。
//!
//! # Architecture
//!
//! ```text
//! UI Layer
//! ├── app.rs              # Application state management
//! ├── events.rs           # Event handling and processing
//! ├── widgets/            # UI widgets (Phase 1.2.2)
//! │   ├── container_list.rs
//! │   └── status_bar.rs
//! └── layouts/            # Layout management (Phase 1.2.3)
//!     └── simple_layout.rs
//! ```
//!
//! # Usage
//!
//! ```rust,no_run
//! use std::sync::Arc;
//! use docka::infrastructure::BollardDockerRepository;
//! use docka::ui::{app::App, events::{handle_key_event, process_app_event}};
//! use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Create Docker repository
//!     let docker_repo = Arc::new(BollardDockerRepository::new().await?);
//!
//!     // Create application state
//!     let mut app = App::new(docker_repo);
//!
//!     // Process user input
//!     let key_event = KeyEvent::new(KeyCode::Char('j'), KeyModifiers::NONE);
//!     let app_event = handle_key_event(key_event);
//!     process_app_event(&mut app, app_event).await?;
//!
//!     Ok(())
//! }
//! ```

/// Application state management for TUI application.
///
/// This module provides the main App struct that manages application state,
/// view state transitions, and container navigation.
///
/// TUIアプリケーション用アプリケーション状態管理。
/// アプリケーション状態、ビュー状態遷移、コンテナナビゲーションを
/// 管理するメインApp構造体を提供します。
pub mod app;

/// Event handling and processing for user interactions.
///
/// This module handles keyboard input, converts raw events to application events,
/// and processes application state updates.
///
/// ユーザー交互作用のイベント処理と処理。
/// キーボード入力を処理し、生イベントをアプリケーションイベントに変換し、
/// アプリケーション状態更新を処理します。
pub mod events;

/// Layout management for UI components.
///
/// This module provides layout managers for organizing UI components
/// and handling responsive design for different terminal sizes.
///
/// UIコンポーネント用レイアウト管理。
/// UIコンポーネントの整理と異なるターミナルサイズでの
/// レスポンシブデザイン処理のためのレイアウトマネージャーを提供します。
pub mod layouts;

/// Widget implementations for TUI components.
///
/// This module contains reusable TUI widgets for displaying containers,
/// status information, and other UI elements.
///
/// TUIコンポーネント用ウィジェット実装。
/// コンテナ、ステータス情報、その他UI要素を表示するための
/// 再利用可能なTUIウィジェットを含みます。
pub mod widgets;

// Re-export commonly used types
// よく使用される型を再エクスポート

/// Main application state struct.
/// メインアプリケーション状態構造体。
pub use app::{App, ViewState};

/// Application event types and processing functions.
/// アプリケーションイベント型と処理関数。
pub use events::{AppEvent, EventStats, handle_key_event, process_app_event, validate_key_input};

/// Layout management types and functions.
/// レイアウト管理型と関数。
pub use layouts::{LayoutAreas, SimpleLayout};

/// Widget types and functions.
/// ウィジェット型と関数。
pub use widgets::StatusBar;
