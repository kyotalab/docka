// src/infrastructure/mod.rs
// Infrastructure layer module root
// インフラ層モジュールルート

//! Infrastructure layer for external integrations
//! 外部統合用インフラ層
//!
//! This layer handles all external system integrations including Docker API,
//! caching, logging, and configuration management. It provides concrete
//! implementations of repository traits defined in the domain layer.
//!
//! この層はDocker API、キャッシング、ログ、設定管理を含む全ての外部システム
//! 統合を処理します。ドメイン層で定義されたリポジトリtraitの具体的な実装を提供します。
//!
//! # Architecture
//!
//! ```text
//! Infrastructure Layer
//! ├── docker/           # Docker API integration
//! ├── cache/            # Caching implementations
//! ├── logging/          # Logging configuration
//! └── config/           # Configuration management (Phase 3)
//! ```
//!
//! # Design Principles
//!
//! - **Dependency Inversion**: Implements interfaces defined in domain layer
//! - **Error Handling**: Converts external errors to domain errors
//! - **Resource Management**: Efficient connection and resource pooling
//! - **Configurability**: Support for various deployment environments
//!
//! # Phase 1 Implementation Status
//!
//! - ✅ Docker API integration (bollard-based)
//! - 🚧 Basic caching (Phase 1.3)
//! - 🚧 Logging setup (Phase 1.3)
//! - 📋 Configuration management (Phase 3)

/// Docker API integration module
/// Docker API統合モジュール
///
/// Provides Docker daemon communication through the bollard crate,
/// implementing the DockerRepository trait with full async support.
///
/// bollardクレートを通じてDocker daemon通信を提供し、
/// 完全な非同期サポートでDockerRepository traitを実装します。
pub mod docker;

// Phase 1.3 で実装予定
// To be implemented in Phase 1.3

// /// Cache implementations module
// /// キャッシュ実装モジュール
// ///
// /// Provides various caching strategies for improving performance,
// /// including in-memory and persistent caching options.
// ///
// /// パフォーマンス向上のための様々なキャッシュ戦略、
// /// インメモリおよび永続キャッシュオプションを提供します。
// pub mod cache;

// /// Logging configuration module
// /// ログ設定モジュール
// ///
// /// Provides structured logging setup and configuration for the application,
// /// supporting various output formats and levels.
// ///
// /// アプリケーションのための構造化ログセットアップと設定、
// /// 様々な出力フォーマットとレベルのサポートを提供します。
// pub mod logging;

// Phase 3 で実装予定
// To be implemented in Phase 3

// /// Configuration management module
// /// 設定管理モジュール
// ///
// /// Provides application configuration management including file-based
// /// configuration, environment variables, and runtime configuration.
// ///
// /// ファイルベース設定、環境変数、ランタイム設定を含む
// /// アプリケーション設定管理を提供します。
// pub mod config;

// Re-export commonly used types for convenient access
// よく使用される型を便利なアクセスのため再エクスポート

/// Bollard-based Docker repository implementation
/// `Bollard`ベースの`Docker`リポジトリ実装
///
/// This is the primary Docker API client implementation for the application.
/// It provides full `DockerRepository` trait implementation with async support.
///
/// これはアプリケーションのプライマリ`Docker` `API`クライアント実装です。
/// 非同期サポート付きの完全な`DockerRepository` `trait`実装を提供します。
///
/// # Examples
///
/// ```rust,no_run
/// use docka::infrastructure::BollardDockerRepository;
/// use docka::domain::repositories::DockerRepository;
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let docker_repo = BollardDockerRepository::new().await?;
///     let containers = docker_repo.list_containers().await?;
///     println!("Found {} containers", containers.len());
///     Ok(())
/// }
/// ```
pub use docker::BollardDockerRepository;

// Phase 1.3 で有効化予定
// To be enabled in Phase 1.3

// /// Cache repository implementation
// /// キャッシュリポジトリ実装
// pub use cache::SimpleCacheRepository;

// /// Logging utilities
// /// ログユーティリティ
// pub use logging::{init_logging, LogLevel, LogFormat};

// Phase 3 で有効化予定
// To be enabled in Phase 3

// /// Configuration management
// /// 設定管理
// pub use config::{AppConfig, ConfigLoader, ConfigError};
