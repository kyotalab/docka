// src/infrastructure/docker/bollard_client.rs
// Bollard-based implementation of DockerRepository trait
// DockerRepository traitのBollardベース実装

use crate::domain::entities::{Container, ContainerFilter};
use crate::domain::repositories::DockerRepository;
use crate::domain::value_objects::{ContainerId, ContainerStatus};
use crate::error::{DockaError, DockaResult};
use async_trait::async_trait;
use bollard::Docker;
use bollard::container::{
    ListContainersOptions, RemoveContainerOptions, StartContainerOptions, StopContainerOptions,
};
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{debug, error, info, warn};

/// Bollard-based implementation of DockerRepository
/// DockerRepositoryのBollardベース実装
///
/// This implementation provides concrete Docker API operations using the bollard crate,
/// which is the official Rust Docker SDK. It handles all Docker daemon communication
/// and converts between bollard types and our domain entities.
///
/// このemplementationはbollardクレートを使用した具体的なDocker API操作を提供します。
/// bollardは公式のRust Docker SDKです。全てのDocker daemon通信を処理し、
/// bollard型と我々のドメインエンティティ間の変換を行います。
///
/// # Design Principles
///
/// - **Error Conversion**: All bollard errors are converted to DockaError
/// - **Type Safety**: Strong typing with ContainerId and domain entities
/// - **Async Operations**: Non-blocking operations for UI responsiveness
/// - **Resource Management**: Efficient connection management with Arc
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
    /// Create a new BollardDockerRepository with default connection
    /// デフォルト接続で新しいBollardDockerRepositoryを作成
    ///
    /// This method attempts to connect to Docker daemon using default settings:
    /// - Unix socket on Linux/macOS
    /// - Named pipe on Windows
    /// - Environment variables (DOCKER_HOST, etc.) if available
    ///
    /// このメソッドはデフォルト設定でDocker daemonへの接続を試行します：
    /// - Linux/macOSではUnixソケット
    /// - WindowsではNamed pipe
    /// - 利用可能な場合は環境変数（DOCKER_HOST等）
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

    /// Create BollardDockerRepository with custom Docker client
    /// カスタムDockerクライアントでBollardDockerRepositoryを作成
    ///
    /// This method allows dependency injection for testing and custom configurations.
    /// このメソッドはテストとカスタム設定のための依存性注入を可能にします。
    ///
    /// # Arguments
    /// * `docker` - Pre-configured Docker client
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
    pub fn client(&self) -> &Docker {
        &self.client
    }
}

#[async_trait]
impl DockerRepository for BollardDockerRepository {
    async fn list_containers(&self) -> DockaResult<Vec<Container>> {
        debug!("Listing all containers");

        // Use default options to list all containers (running and stopped)
        // 全コンテナ（実行中および停止中）をリストするためにデフォルトオプションを使用
        let options = Some(ListContainersOptions::<String> {
            all: true,
            ..Default::default()
        });

        let containers = self.client.list_containers(options).await.map_err(|e| {
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
        // Phase 2 will optimize with server-side filtering
        // Phase 2では、サーバーサイドフィルタリングで最適化
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

        self.client
            .start_container(id.as_str(), None::<StartContainerOptions<String>>)
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

        let options = StopContainerOptions {
            t: timeout_seconds as i64,
        };

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

        let options = Some(RemoveContainerOptions {
            force,
            ..Default::default()
        });

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

        self.client
            .restart_container(id.as_str(), None)
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
    /// このメソッドはbollardのAPI型と我々の強く型付けされた
    /// ドメインエンティティ間の複雑な変換を処理します。
    fn convert_container(
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
            .map(ContainerStatus::from_docker_string)
            .unwrap_or(ContainerStatus::Dead);

        // Convert creation timestamp
        // 作成タイムスタンプを変換
        let created_at = bollard_container
            .created
            .map(|timestamp| {
                chrono::DateTime::from_timestamp(timestamp, 0).unwrap_or_else(chrono::Utc::now)
            })
            .unwrap_or_else(chrono::Utc::now);

        // Convert labels
        // ラベルを変換
        let labels = bollard_container
            .labels
            .unwrap_or_default()
            .into_iter()
            .filter_map(|(k, v)| v.map(|value| (k, value)))
            .collect();

        // Extract command (convert Vec<String> to single string)
        // コマンドを抽出（Vec<String>を単一文字列に変換）
        let command = bollard_container
            .command
            .map(|cmd_vec| cmd_vec.join(" "))
            .filter(|cmd| !cmd.is_empty());

        // Build the domain container
        // ドメインコンテナを構築
        Container::builder()
            .id(container_id.as_str())
            .name(name)
            .image(image)
            .status(status)
            .created_at(created_at)
            .labels(labels)
            .command(command.unwrap_or_default())
            .build()
    }
}
