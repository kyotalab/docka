// src/domain/mod.rs
// Domain layer module root
// ドメイン層モジュールルート

//! Domain layer containing core business logic and entities
//! コアビジネスロジックとエンティティを含むドメイン層
//!
//! This layer defines the core domain models and business rules for Docker
//! container management, independent of external frameworks or infrastructure.
//! The domain layer follows Domain-Driven Design principles.
//!
//! この層は外部フレームワークやインフラに依存しない、
//! Dockerコンテナ管理のコアドメインモデルとビジネスルールを定義します。
//! ドメイン層はドメイン駆動設計の原則に従います。
//!
//! # Architecture
//!
//! ```text
//! Domain Layer
//! ├── entities/     # Domain entities with identity
//! ├── value_objects/ # Immutable value objects
//! └── repositories/ # Repository interfaces (traits)
//! ```
//!
//! # Design Principles
//!
//! - **Domain Independence**: No dependencies on external frameworks
//! - **Type Safety**: Strong typing with value objects
//! - **Business Logic**: Core business rules enforcement
//! - **Testability**: Pure functions and dependency injection

/// Domain entities representing core business objects
/// コアビジネスオブジェクトを表すドメインエンティティ
pub mod entities;

/// Value objects for type safety and domain modeling
/// 型安全性とドメインモデリング用値オブジェクト
pub mod value_objects;

/// Repository trait definitions for data access
/// データアクセス用リポジトリtrait定義
pub mod repositories;

// Re-export commonly used types for convenient access
// よく使用される型を便利なアクセスのため再エクスポート

/// Container domain entity with business logic
/// ビジネスロジックを持つコンテナドメインエンティティ
pub use entities::{Container, ContainerBuilder, ContainerFilter};

/// Image domain entity with metadata management
/// メタデータ管理を持つイメージドメインエンティティ
pub use entities::{Image, ImageBuilder};

/// Strong-typed container identifier
/// Strong-typedコンテナ識別子
pub use value_objects::{ContainerId, ContainerStatus};

// Phase 2/3 で追加予定の再エクスポート (コメントアウト)
// Re-exports to be added in Phase 2/3 (commented out)

// /// Volume domain entity (Phase 2)
// /// ボリュームドメインエンティティ（Phase 2）
// pub use entities::Volume;

// /// Network domain entity (Phase 2)
// /// ネットワークドメインエンティティ（Phase 2）
// pub use entities::Network;

// /// Repository traits (Phase 1.2)
// /// リポジトリtrait（Phase 1.2）
// pub use repositories::{DockerRepository, CacheRepository};
