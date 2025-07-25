// src/domain/value_objects/container_status.rs
// Container status enumeration with state transitions
// 状態遷移を持つコンテナステータス列挙型

use serde::{Deserialize, Serialize};
use std::fmt::{self, Display};

/// Docker container status enumeration
/// Dockerコンテナステータス列挙型
///
/// Represents all possible states a Docker container can be in,
/// with methods to check state transitions and display information.
///
/// Dockerコンテナが取りうる全ての状態を表し、
/// 状態遷移のチェックと表示情報のメソッドを提供します。
///
/// # Examples
///
/// ```rust,no_run
/// # use docka::domain::value_objects::ContainerStatus;
/// let status = ContainerStatus::Running;
/// assert!(status.is_active());
/// assert!(status.can_transition_to(&ContainerStatus::Stopped));
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ContainerStatus {
    /// Container is currently running
    /// コンテナが現在実行中
    Running,

    /// Container is stopped but not removed
    /// コンテナは停止しているが削除されていない
    Stopped,

    /// Container is starting up
    /// コンテナが起動中
    Starting,

    /// Container is stopping
    /// コンテナが停止中
    Stopping,

    /// Container has exited (may be successful or error)
    /// コンテナが終了した（成功またはエラーの可能性）
    Exited {
        /// Exit code from the container process
        /// コンテナプロセスの終了コード
        exit_code: i32,
    },

    /// Container is paused
    /// コンテナが一時停止中
    Paused,

    /// Container is restarting
    /// コンテナが再起動中
    Restarting,

    /// Container is being removed
    /// コンテナが削除中
    Removing,

    /// Container is in an unknown or dead state
    /// コンテナが不明または停止状態
    Dead,

    /// Container creation failed
    /// コンテナ作成が失敗
    Created,
}

impl ContainerStatus {
    /// Check if the container is in an active state
    /// コンテナがアクティブ状態かチェック
    ///
    /// Active states are those where the container is doing work
    /// or is in a transitional state.
    ///
    /// アクティブ状態は、コンテナが作業をしているか
    /// 遷移状態にある状態です。
    #[must_use]
    pub const fn is_active(&self) -> bool {
        matches!(
            self,
            Self::Running | Self::Starting | Self::Stopping | Self::Restarting
        )
    }

    /// Check if the container can be started
    /// コンテナが開始可能かチェック
    #[must_use]
    pub const fn can_start(&self) -> bool {
        matches!(
            self,
            Self::Stopped | Self::Exited { .. } | Self::Created | Self::Dead
        )
    }

    /// Check if the container can be stopped
    /// コンテナが停止可能かチェック
    #[must_use]
    pub const fn can_stop(&self) -> bool {
        matches!(self, Self::Running | Self::Paused | Self::Restarting)
    }

    /// Check if the container can be paused
    /// コンテナが一時停止可能かチェック
    #[must_use]
    pub const fn can_pause(&self) -> bool {
        matches!(self, Self::Running)
    }

    /// Check if the container can be resumed from pause
    /// コンテナが一時停止から再開可能かチェック
    #[must_use]
    pub const fn can_unpause(&self) -> bool {
        matches!(self, Self::Paused)
    }

    /// Check if the container can be removed
    /// コンテナが削除可能かチェック
    #[must_use]
    pub const fn can_remove(&self) -> bool {
        !matches!(
            self,
            Self::Running | Self::Starting | Self::Stopping | Self::Removing
        )
    }

    /// Check if the container can be restarted
    /// コンテナが再起動可能かチェック
    #[must_use]
    pub const fn can_restart(&self) -> bool {
        matches!(
            self,
            Self::Running | Self::Stopped | Self::Exited { .. } | Self::Paused
        )
    }

    /// Check if status transition is valid
    /// ステータス遷移が有効かチェック
    ///
    /// This method validates whether a transition from current status
    /// to target status is logically valid.
    ///
    /// この方法は現在のステータスから対象ステータスへの
    /// 遷移が論理的に有効かどうかを検証します。
    #[must_use]
    pub fn can_transition_to(&self, target: &Self) -> bool {
        match (self, target) {
            // From Running
            // Runningから
            (Self::Running, Self::Stopping | Self::Paused | Self::Restarting) => true,

            // From Stopped
            // Stoppedから
            (Self::Stopped, Self::Starting | Self::Removing) => true,

            // From Starting
            // Startingから
            (Self::Starting, Self::Running | Self::Exited { .. } | Self::Dead) => true,

            // From Stopping
            // Stoppingから
            (Self::Stopping, Self::Stopped | Self::Exited { .. }) => true,

            // From Paused
            // Pausedから
            (Self::Paused, Self::Running | Self::Stopping) => true,

            // From Restarting
            // Restartingから
            (Self::Restarting, Self::Running | Self::Exited { .. } | Self::Dead) => true,

            // From Exited
            // Exitedから
            (Self::Exited { .. }, Self::Starting | Self::Removing) => true,

            // From Created
            // Createdから
            (Self::Created, Self::Starting | Self::Removing) => true,

            // From Dead
            // Deadから
            (Self::Dead, Self::Removing) => true,

            // From Removing (terminal state)
            // Removingから（終端状態）
            (Self::Removing, _) => false,

            // Same state (always valid)
            // 同じ状態（常に有効）
            (a, b) if a == b => true,

            // All other transitions are invalid
            // その他の遷移は全て無効
            _ => false,
        }
    }

    /// Get a human-readable description of the status
    /// ステータスの人間が読める説明を取得
    #[must_use]
    pub fn description(&self) -> &'static str {
        match self {
            Self::Running => "Container is running",
            Self::Stopped => "Container is stopped",
            Self::Starting => "Container is starting",
            Self::Stopping => "Container is stopping",
            Self::Exited { exit_code } if *exit_code == 0 => "Container exited successfully",
            Self::Exited { .. } => "Container exited with error",
            Self::Paused => "Container is paused",
            Self::Restarting => "Container is restarting",
            Self::Removing => "Container is being removed",
            Self::Dead => "Container is dead",
            Self::Created => "Container is created",
        }
    }

    /// Get the display color for UI rendering
    /// UI描画用の表示色を取得
    ///
    /// Returns a color indication suitable for terminal UI display.
    /// ターミナルUI表示に適した色指示を返します。
    #[must_use]
    pub const fn display_color(&self) -> &'static str {
        match self {
            Self::Running => "green",
            Self::Starting | Self::Restarting => "yellow",
            Self::Stopped | Self::Created => "blue",
            Self::Stopping | Self::Removing => "cyan",
            Self::Paused => "magenta",
            Self::Exited { exit_code } if *exit_code == 0 => "green",
            Self::Exited { .. } | Self::Dead => "red",
        }
    }

    /// Parse status from Docker API string
    /// Docker API文字列からステータスを解析
    ///
    /// Converts Docker API status strings to ContainerStatus enum.
    /// Docker APIステータス文字列をContainerStatus列挙型に変換します。
    #[must_use]
    pub fn from_docker_string(status: &str) -> Self {
        match status.to_lowercase().as_str() {
            "running" => Self::Running,
            "stopped" => Self::Stopped,
            "starting" => Self::Starting,
            "stopping" => Self::Stopping,
            "paused" => Self::Paused,
            "restarting" => Self::Restarting,
            "removing" => Self::Removing,
            "dead" => Self::Dead,
            "created" => Self::Created,
            s if s.starts_with("exited") => {
                // Parse exit code from string like "exited (0)" or "exited (1)"
                // "exited (0)"や"exited (1)"のような文字列から終了コードを解析
                let exit_code = s
                    .strip_prefix("exited (")
                    .and_then(|s| s.strip_suffix(')'))
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(-1);
                Self::Exited { exit_code }
            }
            _ => {
                // Unknown status defaults to Dead
                // 不明なステータスはDeadにデフォルト
                Self::Dead
            }
        }
    }
}

impl Display for ContainerStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let status_str = match self {
            Self::Running => "Running",
            Self::Stopped => "Stopped",
            Self::Starting => "Starting",
            Self::Stopping => "Stopping",
            Self::Exited { exit_code } => return write!(f, "Exited ({})", exit_code),
            Self::Paused => "Paused",
            Self::Restarting => "Restarting",
            Self::Removing => "Removing",
            Self::Dead => "Dead",
            Self::Created => "Created",
        };
        write!(f, "{}", status_str)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_status_display() {
        // Test status display formatting
        // ステータス表示フォーマットのテスト
        assert_eq!(ContainerStatus::Running.to_string(), "Running");
        assert_eq!(ContainerStatus::Stopped.to_string(), "Stopped");
        assert_eq!(
            ContainerStatus::Exited { exit_code: 0 }.to_string(),
            "Exited (0)"
        );
        assert_eq!(
            ContainerStatus::Exited { exit_code: 1 }.to_string(),
            "Exited (1)"
        );
    }

    #[test]
    fn test_status_properties() {
        // Test status property methods
        // ステータスプロパティメソッドのテスト

        // Active status tests
        // アクティブステータステスト
        assert!(ContainerStatus::Running.is_active());
        assert!(ContainerStatus::Starting.is_active());
        assert!(!ContainerStatus::Stopped.is_active());
        assert!(!ContainerStatus::Exited { exit_code: 0 }.is_active());

        // Operation capability tests
        // 操作可能性テスト
        assert!(ContainerStatus::Stopped.can_start());
        assert!(!ContainerStatus::Running.can_start());
        assert!(ContainerStatus::Running.can_stop());
        assert!(!ContainerStatus::Stopped.can_stop());
        assert!(ContainerStatus::Running.can_pause());
        assert!(!ContainerStatus::Stopped.can_pause());
    }

    #[test]
    fn test_state_transitions() {
        // Test valid state transitions
        // 有効な状態遷移のテスト
        let running = ContainerStatus::Running;
        let stopped = ContainerStatus::Stopped;
        let starting = ContainerStatus::Starting;

        // Valid transitions
        // 有効な遷移
        assert!(running.can_transition_to(&ContainerStatus::Stopping));
        assert!(stopped.can_transition_to(&ContainerStatus::Starting));
        assert!(starting.can_transition_to(&ContainerStatus::Running));

        // Invalid transitions
        // 無効な遷移
        assert!(!stopped.can_transition_to(&ContainerStatus::Paused));
        assert!(!ContainerStatus::Removing.can_transition_to(&ContainerStatus::Running));
    }

    #[test]
    fn test_docker_string_parsing() {
        // Test parsing from Docker API strings
        // Docker API文字列からの解析テスト

        assert_eq!(
            ContainerStatus::from_docker_string("running"),
            ContainerStatus::Running
        );
        assert_eq!(
            ContainerStatus::from_docker_string("STOPPED"),
            ContainerStatus::Stopped
        );
        assert_eq!(
            ContainerStatus::from_docker_string("exited (0)"),
            ContainerStatus::Exited { exit_code: 0 }
        );
        assert_eq!(
            ContainerStatus::from_docker_string("exited (1)"),
            ContainerStatus::Exited { exit_code: 1 }
        );
        assert_eq!(
            ContainerStatus::from_docker_string("unknown"),
            ContainerStatus::Dead
        );
    }

    #[test]
    fn test_display_properties() {
        // Test display-related properties
        // 表示関連プロパティのテスト

        assert_eq!(ContainerStatus::Running.display_color(), "green");
        assert_eq!(ContainerStatus::Starting.display_color(), "yellow");
        assert_eq!(ContainerStatus::Stopped.display_color(), "blue");
        assert_eq!(
            ContainerStatus::Exited { exit_code: 0 }.display_color(),
            "green"
        );
        assert_eq!(
            ContainerStatus::Exited { exit_code: 1 }.display_color(),
            "red"
        );

        assert!(ContainerStatus::Running.description().contains("running"));
        assert!(
            ContainerStatus::Exited { exit_code: 0 }
                .description()
                .contains("successfully")
        );
        assert!(
            ContainerStatus::Exited { exit_code: 1 }
                .description()
                .contains("error")
        );
    }

    #[test]
    fn test_serialization() {
        // Test serde serialization/deserialization
        // serdeシリアライゼーション/デシリアライゼーションのテスト
        let statuses = vec![
            ContainerStatus::Running,
            ContainerStatus::Stopped,
            ContainerStatus::Exited { exit_code: 0 },
            ContainerStatus::Exited { exit_code: 127 },
        ];

        for original in statuses {
            let json = serde_json::to_string(&original).expect("Serialization should work");
            let deserialized: ContainerStatus =
                serde_json::from_str(&json).expect("Deserialization should work");

            assert_eq!(original, deserialized);
        }
    }
}
