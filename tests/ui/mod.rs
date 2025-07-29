// tests/ui/mod.rs
//! UI module organization for integration tests
//! 統合テスト用UIモジュール構成

/// Widget integration tests
/// ウィジェット統合テスト
///
/// This module contains integration tests for all TUI widgets,
/// ensuring they work correctly with the application state
/// and render properly in various scenarios.
///
/// このモジュールは全TUIウィジェットの統合テストを含み、
/// アプリケーション状態と正しく連携し、様々なシナリオで
/// 適切にレンダリングされることを確認します。
pub mod widgets;

// Future modules for UI integration tests
// UI統合テスト用の将来のモジュール

// /// Layout integration tests
// /// レイアウト統合テスト
// pub mod layouts;

// /// Event handling integration tests
// /// イベント処理統合テスト
// pub mod events;

// /// Theme and styling integration tests
// /// テーマとスタイリング統合テスト
// pub mod styles;

/// Integration tests for UI components
/// UIコンポーネント統合テスト
pub mod integration_tests;
