// src/infrastructure/docker/bollard_client.rs
// Fixed version addressing deprecated API warnings
// 非推奨API警告に対応した修正版

use crate::domain::entities::{Container, ContainerFilter};
use crate::domain::repositories::DockerRepository;
use crate::domain::value_objects::{ContainerId, ContainerStatus};
use crate::error::{DockaError, DockaResult};
use async_trait::async_trait;
// Fixed: Use new OpenAPI generated types for all container operations
// 修正: 全てのコンテナ操作で新しいOpenAPI生成型を使用
use bollard::Docker;
use bollard::query_parameters::{
    ListContainersOptions, ListContainersOptionsBuilder, RemoveContainerOptions,
    RemoveContainerOptionsBuilder, RestartContainerOptions, RestartContainerOptionsBuilder,
    StartContainerOptions, StartContainerOptionsBuilder, StopContainerOptions,
    StopContainerOptionsBuilder,
};
use std::sync::Arc;
use tracing::{debug, error, info, warn};

/// Bollard-based implementation of `DockerRepository`
/// `DockerRepository`の`Bollard`ベース実装
///
/// This implementation provides concrete Docker API operations using the bollard crate,
/// which is the official Rust Docker SDK. It handles all Docker daemon communication
/// and converts between bollard types and our domain entities.
///
/// この実装は`bollard`クレートを使用した具体的な`Docker` `API`操作を提供します。
/// `bollard`は公式の`Rust` `Docker` `SDK`です。全ての`Docker` `daemon`通信を処理し、
/// `bollard`型と我々のドメインエンティティ間の変換を行います。
///
/// # Design Principles
///
/// - **Error Conversion**: All bollard errors are converted to `DockaError`
/// - **Type Safety**: Strong typing with `ContainerId` and domain entities
/// - **Async Operations**: Non-blocking operations for UI responsiveness
/// - **Resource Management**: Efficient connection management with `Arc`
/// - **Modern API**: Uses latest bollard `OpenAPI` generated types
///
/// # Thread Safety
///
/// This implementation is thread-safe and can be shared across async tasks
/// using Arc. The underlying bollard Docker client handles connection pooling.
///
/// # Examples
///
/// ```rust,no_run
/// use docka::infrastructure::docker::BollardDockerRepository;
/// use docka::domain::repositories::DockerRepository;
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let repo = BollardDockerRepository::new().await?;
///     let containers = repo.list_containers().await?;
///     println!("Found {} containers", containers.len());
///     Ok(())
/// }
/// ```
#[derive(Debug, Clone)]
pub struct BollardDockerRepository {
    /// Shared Docker client instance
    /// 共有Dockerクライアントインスタンス
    client: Arc<Docker>,
}

impl BollardDockerRepository {
    /// Create a new `BollardDockerRepository` with default connection
    /// `デフォルト接続で新しいBollardDockerRepositoryを作成`
    ///
    /// This method attempts to connect to Docker daemon using default settings:
    /// - Unix socket on Linux/macOS
    /// - Named pipe on Windows
    /// - Environment variables (`DOCKER_HOST`, etc.) if available
    ///
    /// このメソッドはデフォルト設定でDocker daemonへの接続を試行します：
    /// - Linux/macOSではUnixソケット
    /// - `WindowsではNamed` pipe
    /// - `利用可能な場合は環境変数（DOCKER_HOST等`）
    ///
    /// # Errors
    /// * `DockaError::DockerDaemonNotRunning` - When Docker daemon is not accessible
    /// * `DockaError::PermissionDenied` - When lacking Docker permissions
    /// * `DockaError::DockerApi` - On other connection failures
    pub async fn new() -> DockaResult<Self> {
        info!("Initializing Bollard Docker client");

        // Attempt to connect using default settings
        // デフォルト設定での接続を試行
        let docker = Docker::connect_with_defaults().map_err(|e| {
            error!("Failed to connect to Docker daemon: {}", e);
            match e {
                bollard::errors::Error::DockerResponseServerError {
                    status_code: 403, ..
                } => DockaError::permission_denied("Docker daemon access"),
                _ => DockaError::DockerDaemonNotRunning,
            }
        })?;

        // Verify connection with a ping
        // pingで接続を確認
        Self::verify_connection(&docker).await?;

        let repo = Self {
            client: Arc::new(docker),
        };

        info!("Successfully connected to Docker daemon");
        Ok(repo)
    }

    /// Create `BollardDockerRepository` with custom Docker client
    /// `カスタムDockerクライアントでBollardDockerRepositoryを作成`
    ///
    /// This method allows dependency injection for testing and custom configurations.
    /// このメソッドはテストとカスタム設定のための依存性注入を可能にします。
    ///
    /// # Arguments
    /// * `docker` - Pre-configured Docker client
    #[must_use]
    pub fn with_client(docker: Docker) -> Self {
        Self {
            client: Arc::new(docker),
        }
    }

    /// Verify Docker daemon connection and permissions
    /// Docker daemon接続と権限を確認
    ///
    /// # Errors
    /// * `DockaError::DockerDaemonNotRunning` - When ping fails
    /// * `DockaError::PermissionDenied` - When lacking permissions
    async fn verify_connection(docker: &Docker) -> DockaResult<()> {
        debug!("Verifying Docker daemon connection");

        docker.ping().await.map_err(|e| {
            error!("Docker daemon ping failed: {}", e);
            match e {
                bollard::errors::Error::DockerResponseServerError {
                    status_code: 403, ..
                } => DockaError::permission_denied("Docker daemon ping"),
                _ => DockaError::DockerDaemonNotRunning,
            }
        })?;

        debug!("Docker daemon connection verified successfully");
        Ok(())
    }

    /// Get Docker client reference for advanced operations
    /// 高度な操作のためのDockerクライアント参照を取得
    ///
    /// This method provides access to the underlying bollard client
    /// for operations not covered by the repository interface.
    ///
    /// このメソッドはリポジトリインターフェースでカバーされていない
    /// 操作のため、基礎となるbollardクライアントへのアクセスを提供します。
    #[must_use]
    #[allow(clippy::missing_const_for_fn)] // Arc<T> deref is not const-compatible
    pub fn client(&self) -> &Docker {
        &self.client
    }

    /// Create `ListContainersOptions` using the new Builder API
    /// 新しいBuilder `APIを使用してListContainersOptionsを作成`
    ///
    /// This helper method encapsulates the creation of `ListContainersOptions`
    /// using the new `OpenAPI` generated Builder pattern.
    ///
    /// `このヘルパーメソッドは新しいOpenAPI生成Builderパターンを使用した`
    /// `ListContainersOptionsの作成をカプセル化します。`
    fn create_list_options(all: bool) -> ListContainersOptions {
        ListContainersOptionsBuilder::default().all(all).build()
    }

    /// Create `StartContainerOptions` using the new Builder API
    /// 新しいBuilder `APIを使用してStartContainerOptionsを作成`
    ///
    /// For basic container start operations, we typically don't need additional options.
    /// This helper provides a clean way to create default options.
    ///
    /// 基本的なコンテナ開始操作では、通常追加オプションは不要です。
    /// このヘルパーはデフォルトオプションを作成するクリーンな方法を提供します。
    fn create_start_options() -> StartContainerOptions {
        StartContainerOptionsBuilder::default().build()
    }

    /// Create `StopContainerOptions` with timeout using the new Builder API
    /// 新しいBuilder `APIを使用してタイムアウト付きStopContainerOptionsを作成`
    ///
    /// # Arguments
    /// * `timeout_seconds` - Timeout in seconds before force killing the container
    fn create_stop_options(timeout_seconds: u32) -> StopContainerOptions {
        StopContainerOptionsBuilder::default()
            .t(i64::from(timeout_seconds).try_into().unwrap())
            .build()
    }

    /// Create `RestartContainerOptions` using the new Builder API
    /// 新しいBuilder `APIを使用してRestartContainerOptionsを作成`
    ///
    /// # Arguments
    /// * `timeout_seconds` - Timeout in seconds before force killing the container during restart
    ///
    /// For container restart operations, we can specify a timeout. If not specified,
    /// Docker uses a default timeout (usually 10 seconds).
    ///
    /// コンテナ再起動操作では、タイムアウトを指定できます。指定しない場合、
    /// Dockerはデフォルトタイムアウト（通常10秒）を使用します。
    fn create_restart_options(timeout_seconds: Option<u32>) -> RestartContainerOptions {
        let mut builder = RestartContainerOptionsBuilder::default();

        if let Some(timeout) = timeout_seconds {
            builder = builder.t(i64::from(timeout).try_into().unwrap());
        }

        builder.build()
    }

    /// Create `RemoveContainerOptions` using the new Builder API
    /// 新しいBuilder `APIを使用してRemoveContainerOptionsを作成`
    ///
    /// # Arguments
    /// * `force` - Whether to force remove running containers
    /// * `remove_volumes` - Whether to remove associated volumes (Phase 2 feature)
    fn create_remove_options(force: bool) -> RemoveContainerOptions {
        RemoveContainerOptionsBuilder::default()
            .force(force)
            // Phase 2: Add volume removal option
            // Phase 2: ボリューム削除オプションを追加
            // .v(remove_volumes)
            .build()
    }
    /// Create `ListContainersOptions` with filters (for Phase 2)
    /// フィルタ付きListContainersOptionsを作成（Phase 2用）
    ///
    /// This method will be enhanced in Phase 2 to support server-side filtering
    /// for improved performance with large numbers of containers.
    ///
    /// このメソッドはPhase 2で拡張され、大量のコンテナでの
    /// パフォーマンス向上のためサーバーサイドフィルタリングをサポートします。
    #[allow(dead_code)] // Will be used in Phase 2
    fn create_filtered_list_options(_filter: &ContainerFilter) -> ListContainersOptions {
        // Phase 2: Implement server-side filtering with filters like:
        // Phase 2: 以下のようなフィルタでサーバーサイドフィルタリングを実装:
        // ListContainersOptionsBuilder::default()
        //     .all(true)
        //     .filters(create_docker_filters(filter))
        //     .build()

        // For now, return basic options
        // 現在は基本オプションを返す
        Self::create_list_options(true)
    }
}

#[async_trait]
impl DockerRepository for BollardDockerRepository {
    async fn list_containers(&self) -> DockaResult<Vec<Container>> {
        debug!("Listing all containers");

        // Use new OpenAPI generated options with Builder pattern
        // 新しいOpenAPI生成オプションをBuilderパターンで使用
        let options = Self::create_list_options(true);

        let containers = self
            .client
            .list_containers(Some(options))
            .await
            .map_err(|e| {
                error!("Failed to list containers: {}", e);
                DockaError::DockerApi(e)
            })?;

        debug!("Retrieved {} containers from Docker API", containers.len());

        // Convert bollard containers to domain entities
        // bollardコンテナをドメインエンティティに変換
        let mut domain_containers = Vec::with_capacity(containers.len());
        for container in containers {
            match Self::convert_container(container) {
                Ok(domain_container) => domain_containers.push(domain_container),
                Err(e) => {
                    warn!("Skipping invalid container: {}", e);
                    // Continue processing other containers instead of failing entirely
                    // 完全に失敗するのではなく、他のコンテナの処理を続行
                }
            }
        }

        info!(
            "Successfully converted {} containers",
            domain_containers.len()
        );
        Ok(domain_containers)
    }

    async fn list_containers_filtered(
        &self,
        filter: &ContainerFilter,
    ) -> DockaResult<Vec<Container>> {
        debug!("Listing containers with filter: {:?}", filter);

        // For Phase 1, we'll implement client-side filtering
        // Phase 1では、クライアントサイドフィルタリングを実装
        // Phase 2 will optimize with server-side filtering using create_filtered_list_options
        // Phase 2では、create_filtered_list_optionsを使用してサーバーサイドフィルタリングで最適化
        let all_containers = self.list_containers().await?;

        let filtered_containers: Vec<Container> = all_containers
            .into_iter()
            .filter(|container| filter.matches(container))
            .collect();

        debug!("Filtered containers: {} matches", filtered_containers.len());

        Ok(filtered_containers)
    }

    async fn get_container(&self, id: &ContainerId) -> DockaResult<Container> {
        debug!("Getting container: {}", id);

        // First try to find the container in the list
        // まずリスト内でコンテナを見つけようとする
        let containers = self.list_containers().await?;

        containers
            .into_iter()
            .find(|container| container.id == *id || container.id.matches(id.as_str()))
            .ok_or_else(|| {
                warn!("Container not found: {}", id);
                DockaError::ContainerNotFound {
                    name: id.to_string(),
                }
            })
    }

    async fn start_container(&self, id: &ContainerId) -> DockaResult<()> {
        info!("Starting container: {}", id);

        // Verify container exists and can be started
        // コンテナが存在し、開始可能であることを確認
        let container = self.get_container(id).await?;
        if !container.can_start() {
            return Err(DockaError::invalid_input(format!(
                "Container {} cannot be started from status {}",
                id, container.status
            )));
        }

        // Use consistent options creation pattern
        // 一貫性のあるオプション作成パターンを使用
        let options = Some(Self::create_start_options());

        self.client
            .start_container(id.as_str(), options)
            .await
            .map_err(|e| {
                error!("Failed to start container {}: {}", id, e);
                DockaError::DockerApi(e)
            })?;

        info!("Successfully started container: {}", id);
        Ok(())
    }

    async fn stop_container(&self, id: &ContainerId) -> DockaResult<()> {
        self.stop_container_with_timeout(id, 10).await
    }

    async fn stop_container_with_timeout(
        &self,
        id: &ContainerId,
        timeout_seconds: u32,
    ) -> DockaResult<()> {
        info!("Stopping container: {} (timeout: {}s)", id, timeout_seconds);

        // Verify container exists and can be stopped
        // コンテナが存在し、停止可能であることを確認
        let container = self.get_container(id).await?;
        if !container.can_stop() {
            return Err(DockaError::invalid_input(format!(
                "Container {} cannot be stopped from status {}",
                id, container.status
            )));
        }

        // Use new Builder API for stop options
        // 停止オプションに新しいBuilder APIを使用
        let options = Self::create_stop_options(timeout_seconds);

        self.client
            .stop_container(id.as_str(), Some(options))
            .await
            .map_err(|e| {
                error!("Failed to stop container {}: {}", id, e);
                DockaError::DockerApi(e)
            })?;

        info!("Successfully stopped container: {}", id);
        Ok(())
    }

    async fn remove_container(&self, id: &ContainerId, force: bool) -> DockaResult<()> {
        info!("Removing container: {} (force: {})", id, force);

        // Verify container exists and can be removed (unless forced)
        // コンテナが存在し、削除可能であることを確認（強制でない限り）
        if !force {
            let container = self.get_container(id).await?;
            if !container.can_remove() {
                return Err(DockaError::invalid_input(format!(
                    "Container {} cannot be removed from status {} (use force=true to override)",
                    id, container.status
                )));
            }
        }

        // Use new Builder API for remove options
        // 削除オプションに新しいBuilder APIを使用
        let options = Some(Self::create_remove_options(force));

        self.client
            .remove_container(id.as_str(), options)
            .await
            .map_err(|e| {
                error!("Failed to remove container {}: {}", id, e);
                DockaError::DockerApi(e)
            })?;

        info!("Successfully removed container: {}", id);
        Ok(())
    }

    async fn restart_container(&self, id: &ContainerId) -> DockaResult<()> {
        info!("Restarting container: {}", id);

        // Verify container exists and can be restarted
        // コンテナが存在し、再起動可能であることを確認
        let container = self.get_container(id).await?;
        if !container.can_restart() {
            return Err(DockaError::invalid_input(format!(
                "Container {} cannot be restarted from status {}",
                id, container.status
            )));
        }

        let options = Some(Self::create_restart_options(Some(10))); // Default 10 second timeout
        self.client
            .restart_container(id.as_str(), options)
            .await
            .map_err(|e| {
                error!("Failed to restart container {}: {}", id, e);
                DockaError::DockerApi(e)
            })?;

        info!("Successfully restarted container: {}", id);
        Ok(())
    }

    async fn pause_container(&self, id: &ContainerId) -> DockaResult<()> {
        info!("Pausing container: {}", id);

        // Verify container exists and can be paused
        // コンテナが存在し、一時停止可能であることを確認
        let container = self.get_container(id).await?;
        if !container.can_pause() {
            return Err(DockaError::invalid_input(format!(
                "Container {} cannot be paused from status {}",
                id, container.status
            )));
        }

        self.client
            .pause_container(id.as_str())
            .await
            .map_err(|e| {
                error!("Failed to pause container {}: {}", id, e);
                DockaError::DockerApi(e)
            })?;

        info!("Successfully paused container: {}", id);
        Ok(())
    }

    async fn unpause_container(&self, id: &ContainerId) -> DockaResult<()> {
        info!("Unpausing container: {}", id);

        // Verify container exists and can be unpaused
        // コンテナが存在し、一時停止解除可能であることを確認
        let container = self.get_container(id).await?;
        if !container.can_unpause() {
            return Err(DockaError::invalid_input(format!(
                "Container {} cannot be unpaused from status {}",
                id, container.status
            )));
        }

        self.client
            .unpause_container(id.as_str())
            .await
            .map_err(|e| {
                error!("Failed to unpause container {}: {}", id, e);
                DockaError::DockerApi(e)
            })?;

        info!("Successfully unpaused container: {}", id);
        Ok(())
    }
}

impl BollardDockerRepository {
    /// Convert bollard container to domain Container entity
    /// bollardコンテナをドメインContainerエンティティに変換
    ///
    /// This method handles the complex conversion between bollard's API types
    /// and our strongly-typed domain entities.
    ///
    /// `このメソッドはbollardのAPI型と我々の強く型付けされた`
    /// ドメインエンティティ間の複雑な変換を処理します。
    pub fn convert_container(
        bollard_container: bollard::models::ContainerSummary,
    ) -> DockaResult<Container> {
        // Extract container ID (required field)
        // コンテナIDを抽出（必須フィールド）
        let id = bollard_container
            .id
            .ok_or_else(|| DockaError::internal("Container missing ID from Docker API"))?;

        let container_id = ContainerId::new(id)?;

        // Extract container name (remove leading slash from Docker API)
        // コンテナ名を抽出（Docker APIの先頭スラッシュを削除）
        let name = bollard_container
            .names
            .and_then(|names| names.first().cloned())
            .unwrap_or_default()
            .strip_prefix('/')
            .unwrap_or("")
            .to_string();

        // Extract image name
        // イメージ名を抽出
        let image = bollard_container
            .image
            .unwrap_or_else(|| "unknown".to_string());

        // Convert status
        // ステータスを変換
        let status = bollard_container
            .status
            .as_deref()
            .map_or(ContainerStatus::Dead, ContainerStatus::from_docker_string);

        // Convert creation timestamp
        // 作成タイムスタンプを変換
        let created_at = bollard_container
            .created
            .map_or_else(chrono::Utc::now, |timestamp| {
                chrono::DateTime::from_timestamp(timestamp, 0).unwrap_or_else(chrono::Utc::now)
            });

        // Convert labels (bollard now uses HashMap<String, String> directly)
        // ラベルを変換（bollardは現在HashMap<String, String>を直接使用）
        let labels = bollard_container.labels.unwrap_or_default();

        // Extract command (bollard now provides command as String, not Vec<String>)
        // コマンドを抽出（bollardは現在commandをVec<String>ではなくStringで提供）
        let command = bollard_container.command.filter(|cmd| !cmd.is_empty());

        // Build the domain container
        // ドメインコンテナを構築
        let mut builder = Container::builder()
            .id(container_id.as_str())
            .name(name)
            .image(image)
            .status(status)
            .created_at(created_at)
            .labels(labels);

        // Only set command if it exists and is not empty
        // コマンドが存在し、空でない場合のみ設定
        if let Some(cmd) = command {
            builder = builder.command(cmd);
        }

        builder.build()
    }
}

// Tests remain the same as they test the conversion logic, not the API calls
// テストは変換ロジックをテストするものであり、API呼び出しではないため同じまま
#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::value_objects::ContainerStatus;
    use bollard::models::ContainerSummary;
    use chrono::Utc;
    use std::collections::HashMap;

    /// Create a test ContainerSummary for testing conversion logic
    /// 変換ロジックテスト用のテストContainerSummaryを作成
    fn create_test_container_summary(
        id: &str,
        name: &str,
        image: &str,
        status: &str,
    ) -> ContainerSummary {
        ContainerSummary {
            id: Some(id.to_string()),
            names: Some(vec![format!("/{name}")]), // Docker API adds leading slash
            image: Some(image.to_string()),
            status: Some(status.to_string()),
            created: Some(Utc::now().timestamp()),
            // Fixed: command is now String, not Vec<String>
            // 修正: commandは現在Vec<String>ではなくString
            command: Some("/bin/bash -c sleep infinity".to_string()),
            // Fixed: labels is now HashMap<String, String>, not HashMap<String, Option<String>>
            // 修正: labelsは現在HashMap<String, Option<String>>ではなくHashMap<String, String>
            labels: Some({
                let mut labels = HashMap::new();
                labels.insert("env".to_string(), "test".to_string());
                labels.insert("version".to_string(), "1.0".to_string());
                labels
            }),
            ..Default::default()
        }
    }

    #[test]
    fn test_create_list_options() {
        // Test the new ListContainersOptions creation
        // 新しいListContainersOptions作成のテスト
        let options_all = BollardDockerRepository::create_list_options(true);
        // Note: We can't directly test the internal structure due to the builder pattern,
        // but we can verify that it compiles and creates a valid options object
        // 注意: Builderパターンのため内部構造を直接テストできませんが、
        // コンパイルし、有効なオプションオブジェクトを作成することを確認できます
        drop(options_all); // Just verify it compiles

        let options_running = BollardDockerRepository::create_list_options(false);
        drop(options_running); // Just verify it compiles
    }

    #[test]
    fn test_convert_container_success() {
        // Test successful container conversion
        // 成功したコンテナ変換のテスト
        let bollard_container = create_test_container_summary(
            "test-container-123",
            "web-app",
            "nginx:latest",
            "running",
        );

        let result = BollardDockerRepository::convert_container(bollard_container);
        assert!(result.is_ok(), "Container conversion should succeed");

        let container = result.unwrap();
        assert_eq!(container.id.as_str(), "test-container-123");
        assert_eq!(container.name, "web-app");
        assert_eq!(container.image, "nginx:latest");
        assert_eq!(container.status, ContainerStatus::Running);
        assert_eq!(container.get_label("env"), Some(&"test".to_string()));
        assert_eq!(container.get_label("version"), Some(&"1.0".to_string()));
        assert_eq!(
            container.command,
            Some("/bin/bash -c sleep infinity".to_string())
        );
    }

    // Additional tests remain the same...
    // 追加のテストは同じまま...
    // (Including all the previous test methods for convert_container functionality)

    #[test]
    fn test_convert_container_minimal_data() {
        // Test container conversion with minimal required data
        // 最小限の必須データでのコンテナ変換テスト
        let bollard_container = ContainerSummary {
            id: Some("minimal-123".to_string()),
            names: None,
            image: None,
            status: None,
            created: None,
            command: None,
            labels: None,
            ..Default::default()
        };

        let result = BollardDockerRepository::convert_container(bollard_container);
        assert!(
            result.is_ok(),
            "Minimal container conversion should succeed"
        );

        let container = result.unwrap();
        assert_eq!(container.id.as_str(), "minimal-123");
        assert!(container.name.is_empty()); // Should be empty when names is None
        assert_eq!(container.image, "unknown"); // Default fallback
        assert_eq!(container.status, ContainerStatus::Dead); // Default fallback
        assert!(container.labels.is_empty());
        assert!(container.command.as_ref().map_or(true, |c| c.is_empty())); // Should be empty or None
    }

    #[test]
    fn test_convert_container_missing_id() {
        // Test container conversion fails when ID is missing
        // IDが不足している場合のコンテナ変換失敗テスト
        let bollard_container = ContainerSummary {
            id: None, // Missing ID should cause failure
            names: Some(vec!["/test-container".to_string()]),
            image: Some("nginx:latest".to_string()),
            status: Some("running".to_string()),
            ..Default::default()
        };

        let result = BollardDockerRepository::convert_container(bollard_container);
        assert!(result.is_err(), "Conversion should fail when ID is missing");

        if let Err(DockaError::Internal { message }) = result {
            assert!(message.contains("Container missing ID"));
        } else {
            panic!("Expected Internal error for missing ID");
        }
    }

    #[test]
    fn test_convert_container_invalid_id() {
        // Test container conversion fails with invalid container ID
        // 無効なコンテナIDでのコンテナ変換失敗テスト
        let bollard_container = ContainerSummary {
            id: Some("invalid@id!".to_string()), // Invalid characters
            names: Some(vec!["/test-container".to_string()]),
            image: Some("nginx:latest".to_string()),
            status: Some("running".to_string()),
            ..Default::default()
        };

        let result = BollardDockerRepository::convert_container(bollard_container);
        assert!(result.is_err(), "Conversion should fail with invalid ID");

        if let Err(DockaError::InvalidInput { message }) = result {
            assert!(message.contains("invalid characters"));
        } else {
            panic!("Expected InvalidInput error for invalid ID: {:?}", result);
        }
    }

    #[test]
    fn test_convert_container_status_parsing() {
        // Test various container status conversions
        // 様々なコンテナステータス変換のテスト
        let test_cases = vec![
            ("running", ContainerStatus::Running),
            ("stopped", ContainerStatus::Stopped),
            ("starting", ContainerStatus::Starting),
            ("paused", ContainerStatus::Paused),
            ("exited (0)", ContainerStatus::Exited { exit_code: 0 }),
            ("exited (1)", ContainerStatus::Exited { exit_code: 1 }),
            ("unknown_status", ContainerStatus::Dead), // Unknown status defaults to Dead
        ];

        for (docker_status, expected_status) in test_cases {
            let bollard_container = create_test_container_summary(
                "test-status",
                "test-container",
                "nginx:latest",
                docker_status,
            );

            let result = BollardDockerRepository::convert_container(bollard_container);
            assert!(
                result.is_ok(),
                "Status conversion should succeed for: {}",
                docker_status
            );

            let container = result.unwrap();
            assert_eq!(
                container.status, expected_status,
                "Status mismatch for Docker status: {}",
                docker_status
            );
        }
    }

    #[test]
    fn test_convert_container_name_processing() {
        // Test container name processing (removing leading slash)
        // コンテナ名処理のテスト（先頭スラッシュの削除）
        let test_cases = vec![
            (Some(vec!["/my-container".to_string()]), "my-container"),
            (
                Some(vec!["/complex-name_123".to_string()]),
                "complex-name_123",
            ),
            (Some(vec![]), ""), // Empty names array
            (None, ""),         // No names field
        ];

        for (docker_names, expected_name) in test_cases {
            let bollard_container = ContainerSummary {
                id: Some("test-name-123".to_string()),
                names: docker_names,
                image: Some("nginx:latest".to_string()),
                status: Some("running".to_string()),
                ..Default::default()
            };

            let result = BollardDockerRepository::convert_container(bollard_container);
            assert!(result.is_ok(), "Name conversion should succeed");

            let container = result.unwrap();
            assert_eq!(
                container.name, expected_name,
                "Name mismatch for Docker names: {:?}",
                container.name
            );
        }
    }

    #[test]
    fn test_convert_container_labels_processing() {
        // Test container labels processing
        // コンテナラベル処理のテスト
        // Note: In new bollard API, empty string labels are valid values, not filtered out
        // 注意: 新しいbollard APIでは、空文字列ラベルは有効な値であり、フィルタアウトされません
        let mut docker_labels = HashMap::new();
        docker_labels.insert("env".to_string(), "production".to_string()); // String
        docker_labels.insert("version".to_string(), "2.1.0".to_string());
        docker_labels.insert("maintainer".to_string(), "team@company.com".to_string());

        let bollard_container = ContainerSummary {
            id: Some("test-labels-123".to_string()),
            names: Some(vec!["/test-labels".to_string()]),
            image: Some("nginx:latest".to_string()),
            status: Some("running".to_string()),
            labels: Some(docker_labels),
            ..Default::default()
        };

        let result = BollardDockerRepository::convert_container(bollard_container);
        assert!(result.is_ok(), "Labels conversion should succeed");

        let container = result.unwrap();

        // Check that non-None labels are present
        // Noneでないラベルが存在することを確認
        assert_eq!(container.get_label("env"), Some(&"production".to_string()));
        assert_eq!(container.get_label("version"), Some(&"2.1.0".to_string()));
        assert_eq!(
            container.get_label("maintainer"),
            Some(&"team@company.com".to_string())
        );

        // Check that None labels are filtered out
        // Noneラベルがフィルタアウトされていることを確認
        assert_eq!(container.get_label("empty_label"), None);

        // Verify label count (should be 3, not 4)
        // ラベル数の確認（4ではなく3であるべき）
        assert_eq!(container.labels.len(), 3);
    }

    #[test]
    fn test_convert_container_command_processing() {
        // Test container command processing
        // コンテナコマンド処理のテスト
        let test_cases = vec![
            (
                Some("nginx -g daemon off;".to_string()),
                Some("nginx -g daemon off;".to_string()),
            ),
            (Some("/bin/bash".to_string()), Some("/bin/bash".to_string())),
            (Some("".to_string()), None), // 空文字列は除外
            (None, None),                 // コマンドなし
        ];

        for (docker_command, expected_command) in test_cases {
            let bollard_container = ContainerSummary {
                id: Some("test-command-123".to_string()),
                names: Some(vec!["/test-command".to_string()]),
                image: Some("nginx:latest".to_string()),
                status: Some("running".to_string()),
                command: docker_command.clone(),
                ..Default::default()
            };

            let result = BollardDockerRepository::convert_container(bollard_container);
            assert!(result.is_ok(), "Command conversion should succeed");

            let container = result.unwrap();
            assert_eq!(
                container.command, expected_command,
                "Command mismatch for Docker command: {:?}",
                docker_command
            );
        }
    }

    #[test]
    fn test_convert_container_timestamp_processing() {
        // Test container timestamp processing
        // コンテナタイムスタンプ処理のテスト
        let now = Utc::now();
        let test_timestamp = now.timestamp();

        let bollard_container = ContainerSummary {
            id: Some("test-timestamp-123".to_string()),
            names: Some(vec!["/test-timestamp".to_string()]),
            image: Some("nginx:latest".to_string()),
            status: Some("running".to_string()),
            created: Some(test_timestamp),
            ..Default::default()
        };

        let result = BollardDockerRepository::convert_container(bollard_container);
        assert!(result.is_ok(), "Timestamp conversion should succeed");

        let container = result.unwrap();

        // Check that timestamp is approximately correct (within 1 second)
        // タイムスタンプがほぼ正確であることを確認（1秒以内）
        let expected_time = chrono::DateTime::from_timestamp(test_timestamp, 0).unwrap();
        let time_diff = (container.created_at - expected_time).num_seconds().abs();
        assert!(
            time_diff <= 1,
            "Timestamp should be within 1 second of expected"
        );
    }

    #[test]
    fn test_convert_container_missing_timestamp() {
        // Test container conversion with missing timestamp (should use current time)
        // タイムスタンプが不足しているコンテナ変換のテスト（現在時刻を使用すべき）
        let before_conversion = Utc::now();

        let bollard_container = ContainerSummary {
            id: Some("test-no-timestamp-123".to_string()),
            names: Some(vec!["/test-no-timestamp".to_string()]),
            image: Some("nginx:latest".to_string()),
            status: Some("running".to_string()),
            created: None, // Missing timestamp
            ..Default::default()
        };

        let result = BollardDockerRepository::convert_container(bollard_container);
        assert!(
            result.is_ok(),
            "Conversion should succeed with missing timestamp"
        );

        let container = result.unwrap();
        let after_conversion = Utc::now();

        // Check that created_at is between before and after conversion
        // created_atが変換前後の時刻の間にあることを確認
        assert!(
            container.created_at >= before_conversion && container.created_at <= after_conversion,
            "Missing timestamp should be filled with current time"
        );
    }

    // Integration test helper functions
    // 統合テスト用ヘルパー関数

    /// Create a test BollardDockerRepository with mock client for integration tests
    /// 統合テスト用にモッククライアントを持つテストBollardDockerRepositoryを作成
    ///
    /// Note: This is a placeholder for when we implement mock Docker client.
    /// For now, real integration tests require actual Docker daemon.
    /// 注意：これはモックDockerクライアントを実装する際のプレースホルダーです。
    /// 現在、実際の統合テストには実際のDocker daemonが必要です。
    #[allow(dead_code)]
    fn create_test_repository() -> BollardDockerRepository {
        // This would be implemented with a mock Docker client in a real test environment
        // これは実際のテスト環境ではモックDockerクライアントで実装されるでしょう
        unimplemented!("Mock Docker client not implemented yet")
    }

    // Performance test helpers
    // パフォーマンステスト用ヘルパー

    #[test]
    fn test_convert_container_performance() {
        // Test conversion performance with large label sets
        // 大きなラベルセットでの変換パフォーマンステスト
        let mut large_labels = HashMap::new();
        for i in 0..1000 {
            large_labels.insert(format!("label_{}", i), format!("value_{}", i));
        }

        let bollard_container = ContainerSummary {
            id: Some("perf-test-123".to_string()),
            names: Some(vec!["/perf-test".to_string()]),
            image: Some("nginx:latest".to_string()),
            status: Some("running".to_string()),
            labels: Some(large_labels),
            ..Default::default()
        };

        let start = std::time::Instant::now();
        let result = BollardDockerRepository::convert_container(bollard_container);
        let duration = start.elapsed();

        assert!(result.is_ok(), "Performance test conversion should succeed");
        assert!(
            duration.as_millis() < 100,
            "Conversion should complete within 100ms"
        );

        let container = result.unwrap();
        assert_eq!(
            container.labels.len(),
            1000,
            "All labels should be converted"
        );
    }

    #[test]
    fn test_convert_container_thread_safety() {
        // Test that container conversion is thread-safe
        // コンテナ変換がスレッドセーフであることをテスト
        use std::sync::Arc;
        use std::thread;

        let test_containers: Vec<_> = (0..10)
            .map(|i| {
                create_test_container_summary(
                    &format!("thread-test-{}", i),
                    &format!("container-{}", i),
                    "nginx:latest",
                    "running",
                )
            })
            .collect();

        let containers = Arc::new(test_containers);
        let mut handles = vec![];

        // Spawn multiple threads to test concurrent conversion
        // 並行変換をテストするために複数スレッドを起動
        for i in 0..5 {
            let containers_clone = Arc::clone(&containers);
            let handle = thread::spawn(move || {
                let results: Vec<_> = containers_clone
                    .iter()
                    .map(|container| BollardDockerRepository::convert_container(container.clone()))
                    .collect();

                // All conversions should succeed
                // 全ての変換が成功するべき
                for (idx, result) in results.iter().enumerate() {
                    assert!(
                        result.is_ok(),
                        "Thread {} conversion {} should succeed",
                        i,
                        idx
                    );
                }

                results.len()
            });
            handles.push(handle);
        }

        // Wait for all threads to complete
        // 全スレッドの完了を待機
        for handle in handles {
            let count = handle.join().expect("Thread should not panic");
            assert_eq!(count, 10, "Each thread should process 10 containers");
        }
    }
}
