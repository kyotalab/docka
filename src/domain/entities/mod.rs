// src/domain/entities/mod.rs
// Domain entities module
// ドメインエンティティモジュール

//! Domain entities representing core business objects
//! コアビジネスオブジェクトを表すドメインエンティティ
//!
//! This module contains all domain entities which have identity and lifecycle.
//! Entities encapsulate business logic and maintain their state throughout
//! the application lifecycle.
//!
//! このモジュールはアイデンティティとライフサイクルを持つ
//! 全てのドメインエンティティを含みます。エンティティはビジネスロジックを
//! カプセル化し、アプリケーションライフサイクルを通じて状態を維持します。
//!
//! # Phase 1 Implementation
//!
//! - ✅ Container entity with comprehensive business logic
//! - ✅ Image entity with basic functionality
//! - 🚧 Volume entity (Phase 2 planned)
//! - 🚧 Network entity (Phase 2 planned)

/// Container entity with business logic
/// ビジネスロジックを持つコンテナエンティティ
pub mod container;

/// Image entity with metadata management
/// メタデータ管理を持つイメージエンティティ
pub mod image;

// Phase 2 で追加予定のエンティティ (コメントアウト)
// Entities to be added in Phase 2 (commented out)

// /// Volume entity for storage management
// /// ストレージ管理用ボリュームエンティティ
// pub mod volume;

// /// Network entity for container networking
// /// コンテナネットワーキング用ネットワークエンティティ
// pub mod network;

// Re-export for convenient access
// 便利なアクセスのため再エクスポート

/// Docker container domain entity
/// Dockerコンテナドメインエンティティ
pub use container::{Container, ContainerBuilder, ContainerFilter};

/// Docker image domain entity
/// Dockerイメージドメインエンティティ
pub use image::{Image, ImageBuilder};

// Phase 2 で追加予定の再エクスポート (コメントアウト)
// Re-exports to be added in Phase 2 (commented out)

// /// Docker volume entity (Phase 2)
// /// Dockerボリュームエンティティ（Phase 2）
// pub use volume::{Volume, VolumeBuilder};

// /// Docker network entity (Phase 2)
// /// Dockerネットワークエンティティ（Phase 2）
// pub use network::{Network, NetworkBuilder};
