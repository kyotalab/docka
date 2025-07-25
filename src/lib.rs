// src/lib.rs
// docka library root - main library entry point
// dockaライブラリルート - メインライブラリエントリーポイント

#![deny(missing_docs)]
#![warn(clippy::all)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]

//! # docka
//!
//! A lightweight TUI Docker management tool with fzf-like interface.
//! GUI不要のTUI Docker管理ツール（fzf風インターフェース）
//!
//! ## Features
//!
//! - Lightweight and fast Docker container management
//! - TUI interface with keyboard-driven navigation
//! - Real-time container monitoring
//! - Memory efficient (< 200MB) and fast startup (< 3s)
//!
//! ## Architecture
//!
//! The application follows a layered architecture:
//! - **Domain Layer**: Core business logic and entities
//! - **Application Layer**: Use cases and services
//! - **Infrastructure Layer**: External integrations (Docker API, cache)
//! - **UI Layer**: Terminal user interface
//!
//! ## Phase 1 Implementation Status
//!
//! Currently implementing MVP functionality:
//! - ✅ Error handling system
//! - ✅ Domain entities and value objects
//! - ✅ Repository trait definitions
//! - 🚧 Docker API integration
//! - 🚧 Basic TUI components
//!
//! ## Usage
//!
//! ```rust,no_run
//! use docka::{DockaResult, DockaError, DockerRepository};
//! use docka::domain::{Container, ContainerStatus, ContainerId};
//!
//! async fn example_docker_operations<R: DockerRepository>(
//!     repo: &R
//! ) -> DockaResult<()> {
//!     // List all containers
//!     let containers = repo.list_containers().await?;
//!     println!("Found {} containers", containers.len());
//!
//!     // Operate on first container if available
//!     if let Some(container) = containers.first() {
//!         println!("Container {} is {}", container.display_name(), container.status);
//!
//!         if container.can_stop() {
//!             repo.stop_container(&container.id).await?;
//!             println!("Stopped container {}", container.display_name());
//!         }
//!     }
//!
//!     Ok(())
//! }
//! ```

/// Error handling types and utilities for docka application.
///
/// This module provides a comprehensive error handling system designed for
/// Docker API integration, TUI operations, and application-level error management.
///
/// dockaアプリケーション用エラーハンドリング型とユーティリティ。
/// Docker API統合、TUI操作、アプリケーションレベルのエラー管理のための
/// 包括的なエラーハンドリングシステムを提供します。
///
/// # Examples
///
/// ```rust
/// use docka::error::{DockaError, DockaResult};
///
/// fn validate_container_id(id: &str) -> DockaResult<()> {
///     if id.is_empty() {
///         return Err(DockaError::invalid_input("Container ID cannot be empty"));
///     }
///     Ok(())
/// }
/// ```
pub mod error;

/// Domain layer containing core business logic and entities.
///
/// This layer defines the core domain models and business rules for Docker
/// container management, independent of external frameworks or infrastructure.
///
/// コアビジネスロジックとエンティティを含むドメイン層。
/// 外部フレームワークやインフラに依存しない、Dockerコンテナ管理の
/// コアドメインモデルとビジネスルールを定義します。
pub mod domain;

// Phase 1 implementation modules - uncomment as implemented
// Phase 1実装モジュール - 実装時にコメントアウト解除

// /// Application layer containing use cases and services.
// ///
// /// This layer orchestrates domain operations and coordinates between
// /// the domain layer and infrastructure layer.
// ///
// /// ユースケースとサービスを含むアプリケーション層。
// /// ドメイン操作を調整し、ドメイン層とインフラ層間の連携を担当します。
// pub mod app;

// /// Infrastructure layer for external integrations.
// ///
// /// This layer handles Docker API communication, caching, logging,
// /// and other external system integrations.
// ///
// /// 外部統合用インフラ層。
// /// Docker API通信、キャッシング、ログ、その他外部システム統合を処理します。
// pub mod infrastructure;

// /// User interface layer for terminal-based interaction.
// ///
// /// This layer provides TUI components, event handling, and user interaction
// /// management using the ratatui framework.
// ///
// /// ターミナルベース交互作用用ユーザーインターフェース層。
// /// ratatuiフレームワークを使用したTUIコンポーネント、イベント処理、
// /// ユーザー交互作用管理を提供します。
// pub mod ui;

// /// Actor system for concurrent task management.
// ///
// /// This layer implements the actor pattern for handling asynchronous
// /// operations and maintaining UI responsiveness.
// ///
// /// 並行タスク管理用アクターシステム。
// /// 非同期操作の処理とUI応答性維持のためのアクターパターンを実装します。
// pub mod actors;

// /// Utility functions and helper types.
// ///
// /// This module contains formatting utilities, common helper functions,
// /// and shared types used across the application.
// ///
// /// ユーティリティ関数とヘルパー型。
// /// フォーマットユーティリティ、共通ヘルパー関数、
// /// アプリケーション全体で使用される共有型を含みます。
// pub mod utils;

// Re-export commonly used types for convenient access
// 便利なアクセスのためによく使用される型を再エクスポート

/// Main error type for docka operations.
/// docka操作用メインエラー型。
pub use error::DockaError;

/// Convenient Result type alias for docka operations.
/// docka操作用便利なResult型エイリアス。
pub use error::DockaResult;

/// Container domain entity and related types.
/// コンテナドメインエンティティと関連型。
pub use domain::{Container, ContainerBuilder, ContainerFilter, ContainerId, ContainerStatus};

/// Image domain entity (basic implementation for Phase 1).
/// イメージドメインエンティティ（Phase 1用基本実装）。
pub use domain::{Image, ImageBuilder};

/// Repository trait for Docker API operations.
/// Docker `API操作用リポジトリtrait`。
pub use domain::DockerRepository;

// Test utilities (only available in test builds)
// テストユーティリティ（テストビルドでのみ利用可能）

/// Mock Docker repository implementation for testing.
/// テスト用モックDockerリポジトリ実装。
#[cfg(test)]
pub use domain::MockDockerRepository;
