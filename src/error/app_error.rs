// src/error/app_error.rs
// Custom error types for docka application
// dockaアプリケーション用カスタムエラー型

use thiserror::Error;

/// Main error type for docka application.
///
/// This enum represents all possible error conditions that can occur
/// during Docker operations, UI rendering, and system interactions.
/// Each variant is designed to provide meaningful error messages
/// and appropriate recovery guidance.
///
/// dockaアプリケーションのメインエラー型。
/// Docker操作、UIレンダリング、システム相互作用中に発生する可能性のある
/// 全てのエラー条件を表します。各バリアントは意味のあるエラーメッセージと
/// 適切な回復ガイダンスを提供するよう設計されています。
#[derive(Error, Debug)]
pub enum DockaError {
    /// Docker daemon is not running or not accessible.
    ///
    /// This error occurs when the Docker daemon is not running,
    /// not installed, or not accessible due to permission issues.
    ///
    /// Docker daemonが動作していないか、アクセスできない。
    /// Docker daemonが動作していない、インストールされていない、
    /// または権限の問題でアクセスできない場合に発生します。
    #[error(
        "Docker daemon is not running or not accessible. Please ensure Docker is installed and running."
    )]
    DockerDaemonNotRunning,

    /// Container with specified name/id was not found.
    ///
    /// This error occurs when attempting to operate on a container
    /// that doesn't exist or has been removed.
    ///
    /// 指定された名前/IDのコンテナが見つからない。
    /// 存在しないまたは削除されたコンテナに対して操作を試行した場合に発生します。
    #[error("Container '{name}' not found")]
    ContainerNotFound {
        /// The name or ID of the container that was not found.
        /// 見つからなかったコンテナの名前またはID。
        name: String,
    },

    /// Image with specified name/id was not found.
    ///
    /// This error occurs when attempting to use a Docker image
    /// that doesn't exist locally or in the registry.
    ///
    /// 指定された名前/IDのイメージが見つからない。
    /// ローカルまたはレジストリに存在しないDockerイメージを使用しようとした場合に発生します。
    #[error("Image '{name}' not found")]
    ImageNotFound {
        /// The name or ID of the image that was not found.
        /// 見つからなかったイメージの名前またはID。
        name: String,
    },

    /// Invalid input provided by user.
    ///
    /// This error occurs when user input doesn't meet validation criteria
    /// or is in an unexpected format.
    ///
    /// ユーザーから無効な入力が提供された。
    /// ユーザー入力が検証基準を満たさないか、予期しない形式の場合に発生します。
    #[error("Invalid input: {message}")]
    InvalidInput {
        /// Detailed description of what input was invalid and why.
        /// どの入力が無効で、なぜ無効かの詳細説明。
        message: String,
    },

    /// Operation not permitted due to insufficient privileges.
    ///
    /// This error occurs when attempting to perform operations
    /// that require higher privileges than currently available.
    ///
    /// 権限不足により操作が許可されない。
    /// 現在利用可能な権限よりも高い権限が必要な操作を試行した場合に発生します。
    #[error("Permission denied: {operation}")]
    PermissionDenied {
        /// The operation that was denied due to insufficient privileges.
        /// 権限不足により拒否された操作。
        operation: String,
    },

    /// Docker API communication error.
    ///
    /// This error is automatically converted from `bollard::errors::Error`
    /// and represents low-level Docker API communication failures.
    ///
    /// Docker API通信エラー。
    /// `bollard::errors::Errorから自動変換され`、
    /// 低レベルのDocker API通信失敗を表します。
    #[error("Docker API error: {0}")]
    DockerApi(#[from] bollard::errors::Error),

    /// I/O operation failed.
    ///
    /// This error is automatically converted from `std::io::Error`
    /// and represents file system or network I/O failures.
    ///
    /// I/O操作が失敗。
    /// `std::io::Errorから自動変換され`、
    /// ファイルシステムまたはネットワークI/O失敗を表します。
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// JSON serialization/deserialization error.
    ///
    /// This error is automatically converted from `serde_json::Error`
    /// and represents data format conversion failures.
    ///
    /// JSONシリアライゼーション/デシリアライゼーションエラー。
    /// `serde_json::Errorから自動変換され`、
    /// データフォーマット変換失敗を表します。
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    /// Cache operation failed.
    ///
    /// This error occurs when cache read/write operations fail
    /// or when cache consistency issues are detected.
    ///
    /// キャッシュ操作が失敗。
    /// キャッシュ読み書き操作が失敗した場合や
    /// キャッシュ整合性の問題が検出された場合に発生します。
    #[error("Cache operation failed: {message}")]
    Cache {
        /// Detailed description of the cache operation failure.
        /// キャッシュ操作失敗の詳細説明。
        message: String,
    },

    /// TUI rendering error.
    ///
    /// This error occurs when terminal UI rendering fails
    /// due to terminal size, capability, or display issues.
    ///
    /// TUIレンダリングエラー。
    /// ターミナルサイズ、機能、表示の問題により
    /// ターミナルUIレンダリングが失敗した場合に発生します。
    #[error("UI rendering error: {message}")]
    UiRendering {
        /// Detailed description of the UI rendering failure.
        /// UIレンダリング失敗の詳細説明。
        message: String,
    },

    /// Async task execution error.
    ///
    /// This error is automatically converted from `tokio::task::JoinError`
    /// and represents failures in async task execution or cancellation.
    ///
    /// 非同期タスク実行エラー。
    /// `tokio::task::JoinErrorから自動変換され`、
    /// 非同期タスク実行またはキャンセルの失敗を表します。
    #[error("Task execution error: {0}")]
    TaskExecution(#[from] tokio::task::JoinError),

    /// Configuration error.
    ///
    /// This error occurs when configuration files are invalid,
    /// missing required settings, or contain incompatible values.
    /// Used primarily in Phase 3 for advanced configuration features.
    ///
    /// 設定エラー。
    /// 設定ファイルが無効、必要な設定が不足、または非互換な値を含む場合に発生します。
    /// 主にPhase 3の高度な設定機能で使用されます。
    #[error("Configuration error: {message}")]
    Configuration {
        /// Detailed description of the configuration error.
        /// 設定エラーの詳細説明。
        message: String,
    },

    /// Feature not implemented yet.
    ///
    /// This error is used during staged development to indicate
    /// features that are planned but not yet implemented.
    ///
    /// 機能が未実装。
    /// 段階的開発中に、計画されているがまだ実装されていない
    /// 機能を示すために使用されます。
    #[error("Feature not implemented: {feature}")]
    NotImplemented {
        /// The name of the feature that is not yet implemented.
        /// まだ実装されていない機能の名前。
        feature: String,
    },

    /// Internal application error.
    ///
    /// This error represents unexpected conditions that should not
    /// normally occur and may indicate bugs in the application logic.
    ///
    /// 内部アプリケーションエラー。
    /// 通常は発生すべきでない予期しない条件を表し、
    /// アプリケーションロジックのバグを示している可能性があります。
    #[error("Internal error: {message}")]
    Internal {
        /// Detailed description of the internal error for debugging.
        /// デバッグ用の内部エラーの詳細説明。
        message: String,
    },
}

/// Convenient Result type alias for docka operations
/// docka操作用の便利なResult型エイリアス
pub type DockaResult<T> = Result<T, DockaError>;

impl DockaError {
    /// Create a new `InvalidInput` error
    /// `新しいInvalidInputエラーを作成`
    pub fn invalid_input(message: impl Into<String>) -> Self {
        Self::InvalidInput {
            message: message.into(),
        }
    }

    /// Create a new `PermissionDenied` error
    /// `新しいPermissionDeniedエラーを作成`
    pub fn permission_denied(operation: impl Into<String>) -> Self {
        Self::PermissionDenied {
            operation: operation.into(),
        }
    }

    /// Create a new Cache error
    /// 新しいCacheエラーを作成
    pub fn cache_error(message: impl Into<String>) -> Self {
        Self::Cache {
            message: message.into(),
        }
    }

    /// Create a new `UiRendering` error
    /// `新しいUiRenderingエラーを作成`
    pub fn ui_rendering(message: impl Into<String>) -> Self {
        Self::UiRendering {
            message: message.into(),
        }
    }

    /// Create a new Configuration error
    /// 新しいConfigurationエラーを作成
    pub fn configuration(message: impl Into<String>) -> Self {
        Self::Configuration {
            message: message.into(),
        }
    }

    /// Create a new `NotImplemented` error
    /// `新しいNotImplementedエラーを作成`
    pub fn not_implemented(feature: impl Into<String>) -> Self {
        Self::NotImplemented {
            feature: feature.into(),
        }
    }

    /// Create a new Internal error
    /// 新しいInternalエラーを作成
    pub fn internal(message: impl Into<String>) -> Self {
        Self::Internal {
            message: message.into(),
        }
    }

    /// Check if error is recoverable
    /// エラーが回復可能かチェック
    #[must_use]
    pub const fn is_recoverable(&self) -> bool {
        match self {
            // Recoverable errors - these can typically be resolved by user action or retry
            // 回復可能なエラー - ユーザーアクションやリトライで解決可能
            Self::DockerApi(_)
            | Self::Cache { .. }
            | Self::Configuration { .. }
            | Self::PermissionDenied { .. }
            | Self::InvalidInput { .. }
            | Self::ContainerNotFound { .. }
            | Self::ImageNotFound { .. } => true,

            // Non-recoverable errors - these indicate system-level failures
            // 回復不可能なエラー - システムレベルの失敗を示す
            Self::DockerDaemonNotRunning
            | Self::Io(_)
            | Self::Serialization(_)
            | Self::TaskExecution(_)
            | Self::UiRendering { .. }
            | Self::Internal { .. }
            | Self::NotImplemented { .. } => false,
        }
    }

    /// Get user-friendly error message
    /// ユーザーフレンドリーなエラーメッセージを取得
    #[must_use]
    pub fn user_message(&self) -> String {
        match self {
            Self::DockerDaemonNotRunning => {
                "Docker is not running. Please start Docker and try again.".to_string()
            }
            Self::ContainerNotFound { name } => {
                format!("Container '{name}' was not found. It may have been removed.")
            }
            Self::ImageNotFound { name } => {
                format!("Image '{name}' was not found.")
            }
            Self::InvalidInput { .. } => {
                "Invalid input. Please check your command and try again.".to_string()
            }
            Self::PermissionDenied { .. } => {
                "Permission denied. Please check your Docker permissions.".to_string()
            }
            Self::DockerApi(_) => "Docker operation failed. Please try again.".to_string(),
            Self::Cache { .. } => "Cache operation failed. Data will be refreshed.".to_string(),
            Self::UiRendering { .. } => {
                "Display error occurred. Please resize your terminal.".to_string()
            }
            Self::NotImplemented { feature } => {
                format!("Feature '{feature}' is not yet available.")
            }
            _ => "An unexpected error occurred. Please try again.".to_string(),
        }
    }
}
