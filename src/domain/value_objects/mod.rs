// src/domain/value_objects/mod.rs
// Value objects module for domain types
// ドメイン型用値オブジェクトモジュール

//! Value objects for domain-driven design
//! ドメイン駆動設計用値オブジェクト
//!
//! This module contains all value objects used in the domain layer.
//! Value objects are immutable objects that represent descriptive aspects
//! of the domain with no conceptual identity.
//!
//! このモジュールはドメイン層で使用される全ての値オブジェクトを含みます。
//! 値オブジェクトは概念的なアイデンティティを持たない、
//! ドメインの記述的側面を表現する不変オブジェクトです。

/// Strong-typed container identifier
/// Strong-typedコンテナ識別子
pub mod container_id;

/// Container status enumeration with state transitions
/// 状態遷移を持つコンテナステータス列挙型
pub mod container_status;

// Re-export for convenient access
// 便利なアクセスのため再エクスポート

/// Strong-typed container identifier ensuring type safety
/// 型安全性を保証するstrong-typedコンテナ識別子
pub use container_id::ContainerId;

/// Docker container status enumeration
/// Dockerコンテナステータス列挙型
pub use container_status::ContainerStatus;
