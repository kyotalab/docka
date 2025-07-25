// src/domain/entities/container.rs
// Container domain entity with business logic
// ビジネスロジックを持つコンテナドメインエンティティ

use crate::domain::value_objects::{ContainerId, ContainerStatus};
use crate::error::{DockaError, DockaResult};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Docker container domain entity
/// Dockerコンテナドメインエンティティ
///
/// Represents a Docker container with all its properties and business logic.
/// This entity encapsulates the container's state and provides methods for
/// validation, state queries, and business rule enforcement.
///
/// 全てのプロパティとビジネスロジックを持つDockerコンテナを表します。
/// このエンティティはコンテナの状態をカプセル化し、検証、状態クエリ、
/// ビジネスルール実施のためのメソッドを提供します。
///
/// # Examples
///
/// ```rust
/// use docka::domain::entities::Container;
/// use docka::domain::value_objects::{ContainerId, ContainerStatus};
/// use chrono::Utc;
///
/// let container = Container::builder()
///     .id("web-app-123")
///     .name("web-application")
///     .image("nginx:latest")
///     .status(ContainerStatus::Running)
///     .build()
///     .expect("Valid container");
///
/// assert!(container.is_running());
/// assert!(container.can_stop());
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Container {
    /// Unique container identifier
    /// 一意なコンテナ識別子
    pub id: ContainerId,

    /// Human-readable container name
    /// 人間が読めるコンテナ名
    pub name: String,

    /// Docker image name and tag
    /// Dockerイメージ名とタグ
    pub image: String,

    /// Current container status
    /// 現在のコンテナステータス
    pub status: ContainerStatus,

    /// Container creation timestamp
    /// コンテナ作成タイムスタンプ
    pub created_at: DateTime<Utc>,

    /// Container labels (metadata)
    /// コンテナラベル（メタデータ）
    pub labels: HashMap<String, String>,

    /// Container command
    /// コンテナコマンド
    pub command: Option<String>,

    /// Container working directory
    /// コンテナ作業ディレクトリ
    pub working_dir: Option<String>,
}

impl Container {
    /// Create a new container builder
    /// 新しいコンテナビルダーを作成
    ///
    /// Using the builder pattern ensures all required fields are provided
    /// and allows for flexible container creation with validation.
    ///
    /// ビルダーパターンの使用により、全ての必須フィールドが提供されることを保証し、
    /// 検証付きの柔軟なコンテナ作成を可能にします。
    #[must_use]
    pub fn builder() -> ContainerBuilder {
        ContainerBuilder::new()
    }

    /// Check if container is currently running
    /// コンテナが現在実行中かチェック
    #[must_use]
    pub fn is_running(&self) -> bool {
        self.status == ContainerStatus::Running
    }

    /// Check if container is stopped
    /// コンテナが停止中かチェック
    #[must_use]
    pub const fn is_stopped(&self) -> bool {
        matches!(
            self.status,
            ContainerStatus::Stopped | ContainerStatus::Exited { .. }
        )
    }

    /// Check if container is in a transitional state
    /// コンテナが遷移状態かチェック
    #[must_use]
    pub const fn is_transitioning(&self) -> bool {
        matches!(
            self.status,
            ContainerStatus::Starting
                | ContainerStatus::Stopping
                | ContainerStatus::Restarting
                | ContainerStatus::Removing
        )
    }

    /// Check if container can be started
    /// コンテナが開始可能かチェック
    #[must_use]
    pub const fn can_start(&self) -> bool {
        self.status.can_start()
    }

    /// Check if container can be stopped
    /// コンテナが停止可能かチェック
    #[must_use]
    pub const fn can_stop(&self) -> bool {
        self.status.can_stop()
    }

    /// Check if container can be paused
    /// コンテナが一時停止可能かチェック
    #[must_use]
    pub const fn can_pause(&self) -> bool {
        self.status.can_pause()
    }

    /// Check if container can be resumed from pause
    /// コンテナが一時停止から再開可能かチェック
    #[must_use]
    pub const fn can_unpause(&self) -> bool {
        self.status.can_unpause()
    }

    /// Check if container can be removed
    /// コンテナが削除可能かチェック
    #[must_use]
    pub const fn can_remove(&self) -> bool {
        self.status.can_remove()
    }

    /// Check if container can be restarted
    /// コンテナが再起動可能かチェック
    #[must_use]
    pub const fn can_restart(&self) -> bool {
        self.status.can_restart()
    }

    /// Get container display name
    /// コンテナ表示名を取得
    ///
    /// Returns the container name if available, otherwise returns the short ID.
    /// コンテナ名が利用可能であれば返し、そうでなければ短縮IDを返します。
    #[must_use]
    pub fn display_name(&self) -> &str {
        if self.name.is_empty() {
            self.id.short()
        } else {
            &self.name
        }
    }

    /// Get container age in human-readable format
    /// 人間が読める形式でコンテナの経過時間を取得
    #[must_use]
    pub fn age(&self) -> String {
        let duration = Utc::now().signed_duration_since(self.created_at);

        if duration.num_days() > 0 {
            format!("{} days ago", duration.num_days())
        } else if duration.num_hours() > 0 {
            format!("{} hours ago", duration.num_hours())
        } else if duration.num_minutes() > 0 {
            format!("{} minutes ago", duration.num_minutes())
        } else {
            "Just now".to_string()
        }
    }

    /// Get label value by key
    /// キーによるラベル値の取得
    #[must_use]
    pub fn get_label(&self, key: &str) -> Option<&String> {
        self.labels.get(key)
    }

    /// Check if container has label
    /// コンテナがラベルを持つかチェック
    #[must_use]
    pub fn has_label(&self, key: &str) -> bool {
        self.labels.contains_key(key)
    }

    /// Update container status with validation
    /// 検証付きでコンテナステータスを更新
    ///
    /// This method enforces business rules for status transitions.
    /// このメソッドはステータス遷移のビジネスルールを実施します。
    ///
    /// # Errors
    /// * `DockaError::InvalidInput` - When transition is not allowed
    pub fn update_status(&mut self, new_status: ContainerStatus) -> DockaResult<()> {
        if !self.status.can_transition_to(&new_status) {
            return Err(DockaError::invalid_input(format!(
                "Invalid status transition from {} to {}",
                self.status, new_status
            )));
        }

        self.status = new_status;
        Ok(())
    }

    /// Validate container entity integrity
    /// コンテナエンティティの整合性を検証
    ///
    /// Checks that all container properties are valid and consistent.
    /// 全てのコンテナプロパティが有効で一貫していることをチェックします。
    ///
    /// # Errors
    /// * `DockaError::InvalidInput` - When validation fails
    pub fn validate(&self) -> DockaResult<()> {
        // Validate name format
        // 名前フォーマットの検証
        if self.name.len() > 255 {
            return Err(DockaError::invalid_input(
                "Container name too long (max 255 characters)",
            ));
        }

        // Validate image format (basic check)
        // イメージフォーマットの検証（基本チェック）
        if self.image.is_empty() {
            return Err(DockaError::invalid_input("Container image cannot be empty"));
        }

        if self.image.len() > 255 {
            return Err(DockaError::invalid_input(
                "Container image name too long (max 255 characters)",
            ));
        }

        // Validate creation time is not in the future
        // 作成時間が未来でないことを検証
        if self.created_at > Utc::now() {
            return Err(DockaError::invalid_input(
                "Container creation time cannot be in the future",
            ));
        }

        // Validate labels
        // ラベルの検証
        for (key, value) in &self.labels {
            if key.is_empty() {
                return Err(DockaError::invalid_input(
                    "Container label key cannot be empty",
                ));
            }
            if key.len() > 255 || value.len() > 255 {
                return Err(DockaError::invalid_input(
                    "Container label key or value too long (max 255 characters)",
                ));
            }
        }

        Ok(())
    }
}

/// Builder for creating Container instances with validation
/// 検証付きでContainerインスタンスを作成するビルダー
///
/// The builder pattern ensures that all required fields are provided
/// and validates the container data before creation.
///
/// ビルダーパターンにより、全ての必須フィールドが提供されることを保証し、
/// 作成前にコンテナデータを検証します。
#[derive(Debug, Default)]
pub struct ContainerBuilder {
    id: Option<String>,
    name: Option<String>,
    image: Option<String>,
    status: Option<ContainerStatus>,
    created_at: Option<DateTime<Utc>>,
    labels: HashMap<String, String>,
    command: Option<String>,
    working_dir: Option<String>,
}

impl ContainerBuilder {
    /// Create a new container builder
    /// 新しいコンテナビルダーを作成
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Set container ID
    /// コンテナIDを設定
    #[must_use]
    pub fn id(mut self, id: impl Into<String>) -> Self {
        self.id = Some(id.into());
        self
    }

    /// Set container name
    /// コンテナ名を設定
    #[must_use]
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Set container image
    /// コンテナイメージを設定
    #[must_use]
    pub fn image(mut self, image: impl Into<String>) -> Self {
        self.image = Some(image.into());
        self
    }

    /// Set container status
    /// コンテナステータスを設定
    #[must_use]
    pub const fn status(mut self, status: ContainerStatus) -> Self {
        self.status = Some(status);
        self
    }

    /// Set creation timestamp
    /// 作成タイムスタンプを設定
    #[must_use]
    pub const fn created_at(mut self, created_at: DateTime<Utc>) -> Self {
        self.created_at = Some(created_at);
        self
    }

    /// Add a label
    /// ラベルを追加
    #[must_use]
    pub fn label(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.labels.insert(key.into(), value.into());
        self
    }

    /// Set multiple labels
    /// 複数のラベルを設定
    #[must_use]
    pub fn labels(mut self, labels: HashMap<String, String>) -> Self {
        self.labels = labels;
        self
    }

    /// Set container command
    /// コンテナコマンドを設定
    #[must_use]
    pub fn command(mut self, command: impl Into<String>) -> Self {
        self.command = Some(command.into());
        self
    }

    /// Set working directory
    /// 作業ディレクトリを設定
    #[must_use]
    pub fn working_dir(mut self, working_dir: impl Into<String>) -> Self {
        self.working_dir = Some(working_dir.into());
        self
    }

    /// Build the container with validation
    /// 検証付きでコンテナを構築
    ///
    /// # Errors
    /// * `DockaError::InvalidInput` - When required fields are missing or validation fails
    pub fn build(self) -> DockaResult<Container> {
        // Validate required fields
        // 必須フィールドの検証
        let id_str = self
            .id
            .ok_or_else(|| DockaError::invalid_input("Container ID is required"))?;

        let name = self.name.unwrap_or_default();

        let image = self
            .image
            .ok_or_else(|| DockaError::invalid_input("Container image is required"))?;

        let status = self
            .status
            .ok_or_else(|| DockaError::invalid_input("Container status is required"))?;

        let created_at = self.created_at.unwrap_or_else(Utc::now);

        // Create and validate ContainerId
        // ContainerIdを作成し検証
        let id = ContainerId::new(id_str)?;

        // Create container instance
        // コンテナインスタンスを作成
        let container = Container {
            id,
            name,
            image,
            status,
            created_at,
            labels: self.labels,
            command: self.command,
            working_dir: self.working_dir,
        };

        // Validate the complete container
        // 完全なコンテナを検証
        container.validate()?;

        Ok(container)
    }
}

/// Container filtering criteria
/// コンテナフィルタリング基準
///
/// Used for filtering containers based on various properties.
/// 様々なプロパティに基づいてコンテナをフィルタリングするために使用されます。
#[derive(Debug, Clone, Default)]
pub struct ContainerFilter {
    /// Filter by status
    /// ステータスでフィルタ
    pub status: Option<ContainerStatus>,

    /// Filter by name pattern
    /// 名前パターンでフィルタ
    pub name_pattern: Option<String>,

    /// Filter by image pattern
    /// イメージパターンでフィルタ
    pub image_pattern: Option<String>,

    /// Filter by label key-value pairs
    /// ラベルキー値ペアでフィルタ
    pub labels: HashMap<String, String>,

    /// Include only running containers
    /// 実行中のコンテナのみ含める
    pub only_running: bool,
}

impl ContainerFilter {
    /// Create a new empty filter
    /// 新しい空のフィルタを作成
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Filter for running containers only
    /// 実行中のコンテナのみのフィルタ
    #[must_use]
    pub fn running_only() -> Self {
        Self {
            only_running: true,
            ..Self::default()
        }
    }

    /// Check if container matches this filter
    /// コンテナがこのフィルタにマッチするかチェック
    #[must_use]
    pub fn matches(&self, container: &Container) -> bool {
        // Check status filter
        // ステータスフィルタのチェック
        if let Some(ref status) = self.status {
            if container.status != *status {
                return false;
            }
        }

        // Check running-only filter
        // 実行中のみフィルタのチェック
        if self.only_running && !container.is_running() {
            return false;
        }

        // Check name pattern
        // 名前パターンのチェック
        if let Some(ref pattern) = self.name_pattern {
            if !container.name.contains(pattern) && !container.id.matches(pattern) {
                return false;
            }
        }

        // Check image pattern
        // イメージパターンのチェック
        if let Some(ref pattern) = self.image_pattern {
            if !container.image.contains(pattern) {
                return false;
            }
        }

        // Check label filters
        // ラベルフィルタのチェック
        for (key, value) in &self.labels {
            match container.get_label(key) {
                Some(container_value) if container_value == value => {}
                _ => return false,
            }
        }

        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_container() -> Container {
        Container::builder()
            .id("test-container-123")
            .name("test-app")
            .image("nginx:latest")
            .status(ContainerStatus::Running)
            .created_at(Utc::now())
            .label("environment", "test")
            .label("version", "1.0")
            .command("/bin/bash")
            .working_dir("/app")
            .build()
            .expect("Valid container")
    }

    #[test]
    fn test_container_builder_success() {
        // Test successful container creation
        // 成功したコンテナ作成のテスト
        let container = create_test_container();

        assert_eq!(container.id.as_str(), "test-container-123");
        assert_eq!(container.name, "test-app");
        assert_eq!(container.image, "nginx:latest");
        assert_eq!(container.status, ContainerStatus::Running);
        assert_eq!(
            container.get_label("environment"),
            Some(&"test".to_string())
        );
        assert_eq!(container.command.as_deref(), Some("/bin/bash"));
        assert_eq!(container.working_dir.as_deref(), Some("/app"));
    }

    #[test]
    fn test_container_builder_missing_required_fields() {
        // Test builder with missing required fields
        // 必須フィールドが不足したビルダーのテスト

        // Missing ID
        // ID不足
        let result = Container::builder()
            .name("test")
            .image("nginx:latest")
            .status(ContainerStatus::Running)
            .build();
        assert!(result.is_err());

        // Missing image
        // イメージ不足
        let result = Container::builder()
            .id("test-id")
            .name("test")
            .status(ContainerStatus::Running)
            .build();
        assert!(result.is_err());

        // Missing status
        // ステータス不足
        let result = Container::builder()
            .id("test-id")
            .name("test")
            .image("nginx:latest")
            .build();
        assert!(result.is_err());
    }

    #[test]
    fn test_container_status_queries() {
        // Test container status query methods
        // コンテナステータスクエリメソッドのテスト

        let running_container = Container::builder()
            .id("running-123")
            .image("nginx:latest")
            .status(ContainerStatus::Running)
            .build()
            .unwrap();

        assert!(running_container.is_running());
        assert!(!running_container.is_stopped());
        assert!(!running_container.is_transitioning());
        assert!(!running_container.can_start());
        assert!(running_container.can_stop());

        let stopped_container = Container::builder()
            .id("stopped-123")
            .image("nginx:latest")
            .status(ContainerStatus::Stopped)
            .build()
            .unwrap();

        assert!(!stopped_container.is_running());
        assert!(stopped_container.is_stopped());
        assert!(!stopped_container.is_transitioning());
        assert!(stopped_container.can_start());
        assert!(!stopped_container.can_stop());
    }

    #[test]
    fn test_container_display_name() {
        // Test display name logic
        // 表示名ロジックのテスト

        // Container with name
        // 名前付きコンテナ
        let named_container = Container::builder()
            .id("long-container-id-123")
            .name("web-app")
            .image("nginx:latest")
            .status(ContainerStatus::Running)
            .build()
            .unwrap();
        assert_eq!(named_container.display_name(), "web-app");

        // Container without name
        // 名前なしコンテナ
        let unnamed_container = Container::builder()
            .id("long-container-id-123")
            .name("")
            .image("nginx:latest")
            .status(ContainerStatus::Running)
            .build()
            .unwrap();
        assert_eq!(unnamed_container.display_name(), "long-contain"); // short ID
    }

    #[test]
    fn test_status_update_validation() {
        // Test status update with validation
        // 検証付きステータス更新のテスト

        let mut container = Container::builder()
            .id("test-123")
            .image("nginx:latest")
            .status(ContainerStatus::Running)
            .build()
            .unwrap();

        // Valid transition
        // 有効な遷移
        let result = container.update_status(ContainerStatus::Stopping);
        assert!(result.is_ok());
        assert_eq!(container.status, ContainerStatus::Stopping);

        // Invalid transition
        // 無効な遷移
        let result = container.update_status(ContainerStatus::Running);
        assert!(result.is_err());
    }

    #[test]
    fn test_container_validation() {
        // Test container validation rules
        // コンテナ検証ルールのテスト

        // Valid container
        // 有効なコンテナ
        let valid_container = create_test_container();
        assert!(valid_container.validate().is_ok());

        // Container with too long name
        // 名前が長すぎるコンテナ
        let result = Container::builder()
            .id("test-123")
            .name("a".repeat(256))
            .image("nginx:latest")
            .status(ContainerStatus::Running)
            .build();
        assert!(result.is_err());

        // Container with empty image
        // 空のイメージのコンテナ
        let result = Container::builder()
            .id("test-123")
            .name("test")
            .image("")
            .status(ContainerStatus::Running)
            .build();
        assert!(result.is_err());
    }

    #[test]
    fn test_container_filter() {
        // Test container filtering
        // コンテナフィルタリングのテスト

        let containers = vec![
            Container::builder()
                .id("web-123")
                .name("web-app")
                .image("nginx:latest")
                .status(ContainerStatus::Running)
                .label("env", "prod")
                .build()
                .unwrap(),
            Container::builder()
                .id("db-456")
                .name("database")
                .image("postgres:13")
                .status(ContainerStatus::Stopped)
                .label("env", "dev")
                .build()
                .unwrap(),
            Container::builder()
                .id("api-789")
                .name("api-server")
                .image("node:18")
                .status(ContainerStatus::Running)
                .label("env", "prod")
                .build()
                .unwrap(),
        ];

        // Filter running only
        // 実行中のみフィルタ
        let running_filter = ContainerFilter::running_only();
        let running_containers: Vec<_> = containers
            .iter()
            .filter(|c| running_filter.matches(c))
            .collect();
        assert_eq!(running_containers.len(), 2);

        // Filter by name pattern
        // 名前パターンでフィルタ
        let mut name_filter = ContainerFilter::new();
        name_filter.name_pattern = Some("web".to_string());
        let web_containers: Vec<_> = containers
            .iter()
            .filter(|c| name_filter.matches(c))
            .collect();
        assert_eq!(web_containers.len(), 1);
        assert_eq!(web_containers[0].name, "web-app");

        // Filter by label
        // ラベルでフィルタ
        let mut label_filter = ContainerFilter::new();
        label_filter
            .labels
            .insert("env".to_string(), "prod".to_string());
        let prod_containers: Vec<_> = containers
            .iter()
            .filter(|c| label_filter.matches(c))
            .collect();
        assert_eq!(prod_containers.len(), 2);
    }

    #[test]
    fn test_container_age_formatting() {
        // Test age formatting
        // 経過時間フォーマットのテスト
        use chrono::Duration;

        let now = Utc::now();

        // Recent container (minutes)
        // 最近のコンテナ（分）
        let recent_container = Container::builder()
            .id("recent-123")
            .image("nginx:latest")
            .status(ContainerStatus::Running)
            .created_at(now - Duration::minutes(30))
            .build()
            .unwrap();
        let age = recent_container.age();
        assert!(age.contains("minutes ago") || age.contains("Just now"));

        // Old container (days)
        // 古いコンテナ（日）
        let old_container = Container::builder()
            .id("old-123")
            .image("nginx:latest")
            .status(ContainerStatus::Running)
            .created_at(now - Duration::days(5))
            .build()
            .unwrap();
        let age = old_container.age();
        assert!(age.contains("days ago"));
    }

    #[test]
    fn test_serialization() {
        // Test serde serialization/deserialization
        // serdeシリアライゼーション/デシリアライゼーションのテスト
        let original = create_test_container();

        let json = serde_json::to_string(&original).expect("Serialization should work");
        let deserialized: Container =
            serde_json::from_str(&json).expect("Deserialization should work");

        assert_eq!(original, deserialized);
    }

    #[test]
    fn test_container_pause_unpause_operations() {
        // Test container pause/unpause capability checks
        // コンテナ一時停止/再開可能性チェックのテスト

        let running_container = Container::builder()
            .id("pause-test-123")
            .image("nginx:latest")
            .status(ContainerStatus::Running)
            .build()
            .unwrap();

        let paused_container = Container::builder()
            .id("unpause-test-123")
            .image("nginx:latest")
            .status(ContainerStatus::Paused)
            .build()
            .unwrap();

        let stopped_container = Container::builder()
            .id("stopped-test-123")
            .image("nginx:latest")
            .status(ContainerStatus::Stopped)
            .build()
            .unwrap();

        // Running container can be paused but not unpaused
        // 実行中のコンテナは一時停止可能だが一時停止解除は不可
        assert!(running_container.can_pause());
        assert!(!running_container.can_unpause());

        // Paused container can be unpaused but not paused
        // 一時停止中のコンテナは一時停止解除可能だが一時停止は不可
        assert!(!paused_container.can_pause());
        assert!(paused_container.can_unpause());

        // Stopped container can neither be paused nor unpaused
        // 停止中のコンテナは一時停止も一時停止解除も不可
        assert!(!stopped_container.can_pause());
        assert!(!stopped_container.can_unpause());
    }

    #[test]
    fn test_container_pause_unpause_capability() {
        // Test pause/unpause capability checks
        // 一時停止/再開可能性チェックのテスト

        let test_cases = vec![
            (ContainerStatus::Running, true, false), // can_pause, cannot_unpause
            (ContainerStatus::Paused, false, true),  // cannot_pause, can_unpause
            (ContainerStatus::Stopped, false, false), // cannot_pause, cannot_unpause
            (ContainerStatus::Starting, false, false), // cannot_pause, cannot_unpause
        ];

        for (status, expected_can_pause, expected_can_unpause) in test_cases {
            let container = Container::builder()
                .id("test-container")
                .image("nginx:latest")
                .status(status.clone())
                .build()
                .unwrap();

            assert_eq!(
                container.can_pause(),
                expected_can_pause,
                "can_pause() failed for status: {status}"
            );

            assert_eq!(
                container.can_unpause(),
                expected_can_unpause,
                "can_unpause() failed for status: {status}"
            );
        }
    }
}
