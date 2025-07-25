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
//! - ✅ DockerRepository trait (Task 1.1.2 - 完了)
//! - 🚧 CacheRepository trait (Task 1.3.4 実装予定)
//! - 🚧 ConfigRepository trait (Phase 3 実装予定)
//!
//! # Usage Examples
//!
//! ```rust,no_run
//! use docka::domain::repositories::DockerRepository;
//! use docka::domain::value_objects::ContainerId;
//!
//! async fn manage_container<R: DockerRepository>(
//!     repo: &R,
//!     container_id: &ContainerId
//! ) -> docka::DockaResult<()> {
//!     let container = repo.get_container(container_id).await?;
//!
//!     if container.can_stop() {
//!         repo.stop_container(container_id).await?;
//!         println!("Container {} stopped", container.display_name());
//!     }
//!
//!     Ok(())
//! }
//! ```

/// Docker API operations repository trait
/// Docker `API操作リポジトリtrait`
pub mod docker_repository;

// Phase 1.3 で実装予定
// To be implemented in Phase 1.3

// /// Cache operations repository trait
// /// キャッシュ操作リポジトリtrait
// pub mod cache_repository;

// Phase 3 で実装予定
// To be implemented in Phase 3

// /// Configuration management repository trait
// /// 設定管理リポジトリtrait
// pub mod config_repository;

// Re-export for convenient access
// 便利なアクセスのため再エクスポート

/// Docker API operations repository trait
/// Docker `API操作リポジトリtrait`
///
/// This trait provides a clean abstraction over Docker API operations,
/// allowing for dependency injection and testing through mock implementations.
///
/// このtraitはDocker API操作に対するクリーンな抽象化を提供し、
/// モック実装を通じた依存性注入とテストを可能にします。
pub use docker_repository::DockerRepository;

// Phase 1.3 で有効化予定
// To be enabled in Phase 1.3

// /// Cache operations repository trait
// /// キャッシュ操作リポジトリtrait
// pub use cache_repository::CacheRepository;

// Phase 3 で追加予定
// To be added in Phase 3

// /// Configuration repository trait
// /// 設定リポジトリtrait
// pub use config_repository::ConfigRepository;

// Test utilities re-export
// テストユーティリティの再エクスポート

/// Mock implementation of `DockerRepository` for testing
/// `テスト用DockerRepositoryのモック実装`
///
/// This mock provides a complete in-memory implementation of the `DockerRepository`
/// trait for unit testing without requiring actual Docker daemon.
///
/// このモックは実際のDocker daemonを必要とせずに単体テストのための
/// `DockerRepositoryトレイトの完全なインメモリ実装を提供します`。
///
/// # Examples
///
/// ```rust
/// # #[tokio::test]
/// # async fn example_test() {
/// use docka::domain::repositories::{DockerRepository, MockDockerRepository};
/// use docka::domain::entities::Container;
/// use docka::domain::value_objects::{ContainerId, ContainerStatus};
///
/// let mock_repo = MockDockerRepository::new();
///
/// // Add test container
/// let container = Container::builder()
///     .id("test-container-123")
///     .name("test-app")
///     .image("nginx:latest")
///     .status(ContainerStatus::Running)
///     .build()
///     .expect("Valid container");
///
/// mock_repo.add_container(container.clone()).await;
///
/// // Test repository operations
/// let containers = mock_repo.list_containers().await.expect("List should succeed");
/// assert_eq!(containers.len(), 1);
/// assert_eq!(containers[0].id, container.id);
/// # }
/// ```
#[cfg(test)]
pub use docker_repository::MockDockerRepository;
