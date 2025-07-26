// src/infrastructure/docker/mod.rs
// Docker infrastructure module
// Dockerインフラモジュール

//! Docker infrastructure module providing Docker API integration
//! Docker API統合を提供するDockerインフラモジュール
//!
//! This module contains implementations for Docker API communication,
//! data mapping between Docker API and domain entities, and error handling
//! specific to Docker operations.
//!
//! このモジュールはDocker API通信、Docker APIとドメインエンティティ間の
//! データマッピング、Docker操作固有のエラーハンドリングの実装を含みます。
//!
//! # Architecture
//!
//! ```text
//! Infrastructure/Docker Layer
//! ├── bollard_client.rs    # Main Docker API client implementation
//! ├── api_mapper.rs        # API response to domain entity mapping
//! └── error_handler.rs     # Docker-specific error handling
//! ```
//!
//! # Usage
//!
//! ```rust,no_run
//! use docka::infrastructure::docker::BollardDockerRepository;
//! use docka::domain::repositories::DockerRepository;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Create Docker repository
//!     let docker_repo = BollardDockerRepository::new().await?;
//!
//!     // Use through trait interface
//!     let containers = docker_repo.list_containers().await?;
//!     println!("Found {} containers", containers.len());
//!
//!     Ok(())
//! }
//! ```

/// Bollard-based Docker API client implementation
/// `Bollard`ベースの`Docker` `API`クライアント実装
///
/// This module provides the main implementation of the `DockerRepository` trait
/// using the bollard crate for Docker API communication.
///
/// このモジュールは`Docker` `API`通信に`bollard`クレートを使用した
/// `DockerRepository` `trait`のメイン実装を提供します。
pub mod bollard_client;

/// Bollard-based implementation of `DockerRepository` trait
/// `DockerRepository` `trait`の`Bollard`ベース実装
///
/// This is the primary implementation for Docker API operations in Phase 1.
/// It provides full async support and comprehensive error handling.
///
/// これは`Phase` 1での`Docker` `API`操作のプライマリ実装です。
/// 完全な非同期サポートと包括的なエラーハンドリングを提供します。
pub use bollard_client::BollardDockerRepository;

// Phase 1.2 で実装予定
// To be implemented in Phase 1.2

// /// API response mapping utilities
// /// APIレスポンスマッピングユーティリティ
// ///
// /// This module contains utilities for converting between Docker API responses
// /// and domain entities, handling complex data transformations.
// ///
// /// このモジュールはDocker APIレスポンスとドメインエンティティ間の変換、
// /// 複雑なデータ変換の処理のためのユーティリティを含みます。
// pub mod api_mapper;

// Phase 1.3 で実装予定
// To be implemented in Phase 1.3

// /// Docker-specific error handling
// /// Docker固有のエラーハンドリング
// ///
// /// This module provides specialized error handling for Docker API operations,
// /// including retry logic and connection management.
// ///
// /// このモジュールはDocker API操作のための特殊化されたエラーハンドリング、
// /// リトライロジックと接続管理を含みます。
// pub mod error_handler;

// Phase 1.2 で有効化予定
// To be enabled in Phase 1.2

// /// API mapping utilities re-export
// /// APIマッピングユーティリティの再エクスポート
// pub use api_mapper::{ContainerMapper, ImageMapper, ErrorMapper};

// Phase 1.3 で有効化予定
// To be enabled in Phase 1.3

// /// Docker error handling utilities re-export
// /// Dockerエラーハンドリングユーティリティの再エクスポート
// pub use error_handler::{DockerErrorHandler, RetryPolicy, ConnectionManager};
