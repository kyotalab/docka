// src/domain/repositories/mod.rs
// Repository trait definitions for data access
// データアクセス用リポジトリtrait定義

//! Repository trait definitions for data access abstraction
//! データアクセス抽象化用リポジトリtrait定義
//!
//! This module defines repository interfaces that abstract data access operations.
//! Repositories follow the Repository pattern to decouple domain logic from
//! data access implementation details.
//!
//! このモジュールはデータアクセス操作を抽象化するリポジトリインターフェースを定義します。
//! リポジトリはリポジトリパターンに従い、ドメインロジックを
//! データアクセス実装詳細から分離します。
//!
//! # Phase 1 Implementation Status
//!
//! - 🚧 DockerRepository trait (Task 1.1.2 - 次回実装予定)
//! - 🚧 CacheRepository trait (Task 1.3.4 実装予定)
//! - 🚧 ConfigRepository trait (Phase 3 実装予定)

// Phase 1.2 で実装予定
// To be implemented in Phase 1.2

// /// Docker API operations repository trait
// /// Docker API操作リポジトリtrait
// pub mod docker_repository;

// /// Cache operations repository trait
// /// キャッシュ操作リポジトリtrait
// pub mod cache_repository;

// Phase 3 で実装予定
// To be implemented in Phase 3

// /// Configuration management repository trait
// /// 設定管理リポジトリtrait
// pub mod config_repository;

// Re-export for convenient access (Phase 1.2 で有効化予定)
// Re-exports for convenient access (to be enabled in Phase 1.2)

// /// Docker API operations repository
// /// Docker API操作リポジトリ
// pub use docker_repository::DockerRepository;

// /// Cache operations repository
// /// キャッシュ操作リポジトリ
// pub use cache_repository::CacheRepository;

// Phase 3 で追加予定
// To be added in Phase 3

// /// Configuration repository
// /// 設定リポジトリ
// pub use config_repository::ConfigRepository;
