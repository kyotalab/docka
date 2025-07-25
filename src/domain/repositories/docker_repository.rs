// src/domain/repositories/docker_repository.rs
// Docker API operations repository trait
// Docker API操作リポジトリtrait

use crate::domain::entities::{Container, ContainerFilter};
use crate::domain::value_objects::ContainerId;
use crate::error::DockaResult;
use async_trait::async_trait;

/// Repository trait for Docker API operations
/// Docker API操作用リポジトリtrait
///
/// This trait abstracts Docker API operations and provides a clean interface
/// for the application layer. It follows the Repository pattern to decouple
/// domain logic from infrastructure concerns.
///
/// このtraitはDocker API操作を抽象化し、アプリケーション層に
/// クリーンなインターフェースを提供します。リポジトリパターンに従い、
/// ドメインロジックをインフラの関心事から分離します。
///
/// # Design Principles
///
/// - **Async by Design**: All operations are async for non-blocking UI
/// - **Error Handling**: Uses DockaResult for consistent error management
/// - **Type Safety**: Leverages strong typing with ContainerId and Container
/// - **Testability**: Enables dependency injection and mocking
///
/// # Phase Implementation Status
///
/// **Phase 1 (Current)**: Basic container lifecycle operations
/// - Container listing and filtering
/// - Start, stop, restart, pause/unpause operations
/// - Container removal with safety checks
/// - Full CRUD operations for core functionality
///
/// **Phase 2 (Planned)**: Advanced monitoring and interaction
/// - Container logs retrieval and streaming
/// - Real-time resource statistics and monitoring
/// - Command execution within containers
/// - Advanced filtering and search capabilities
///
/// **Phase 3 (Future)**: Enterprise and team features
/// - Batch operations for multiple containers
/// - Custom hooks and automation
/// - Advanced configuration management
/// - Audit logging and compliance features
///
/// # Examples
///
/// ```rust,no_run
/// use docka::domain::repositories::DockerRepository;
/// use docka::domain::value_objects::ContainerId;
///
/// async fn example_usage<R: DockerRepository>(repo: &R) -> docka::DockaResult<()> {
///     let containers = repo.list_containers().await?;
///     println!("Found {} containers", containers.len());
///
///     if let Some(container) = containers.first() {
///         if container.can_stop() {
///             repo.stop_container(&container.id).await?;
///         }
///     }
///     Ok(())
/// }
/// ```
#[async_trait]
pub trait DockerRepository: Send + Sync {
    // =========================================================================
    // Phase 1 Implementation: Core Container Operations
    // Phase 1実装: コアコンテナ操作
    // =========================================================================

    /// List all containers
    /// 全コンテナを一覧表示
    ///
    /// Returns all containers known to Docker daemon, both running and stopped.
    /// Use `list_containers_filtered` for more specific queries.
    ///
    /// Docker daemonが認識している実行中・停止中の全コンテナを返します。
    /// より具体的なクエリには`list_containers_filtered`を使用してください。
    ///
    /// # Errors
    /// * `DockaError::DockerDaemonNotRunning` - When Docker daemon is not accessible
    /// * `DockaError::DockerApi` - On Docker API communication errors
    /// * `DockaError::PermissionDenied` - When lacking Docker access permissions
    async fn list_containers(&self) -> DockaResult<Vec<Container>>;

    /// List containers with filtering
    /// フィルタリング付きコンテナ一覧
    ///
    /// Returns containers matching the specified filter criteria.
    /// This is more efficient than filtering after retrieval.
    ///
    /// 指定されたフィルタ条件に一致するコンテナを返します。
    /// 取得後のフィルタリングよりも効率的です。
    ///
    /// # Arguments
    /// * `filter` - Container filtering criteria
    ///
    /// # Errors
    /// * `DockaError::DockerDaemonNotRunning` - When Docker daemon is not accessible
    /// * `DockaError::DockerApi` - On Docker API communication errors
    async fn list_containers_filtered(
        &self,
        filter: &ContainerFilter,
    ) -> DockaResult<Vec<Container>>;

    /// Get specific container by ID
    /// IDによる特定コンテナ取得
    ///
    /// Retrieves detailed information about a single container.
    /// Returns error if container doesn't exist.
    ///
    /// 単一コンテナの詳細情報を取得します。
    /// コンテナが存在しない場合はエラーを返します。
    ///
    /// # Arguments
    /// * `id` - Container identifier (supports both full and short IDs)
    ///
    /// # Errors
    /// * `DockaError::ContainerNotFound` - When container doesn't exist
    /// * `DockaError::DockerApi` - On Docker API communication errors
    async fn get_container(&self, id: &ContainerId) -> DockaResult<Container>;

    /// Start a stopped container
    /// 停止中のコンテナを開始
    ///
    /// Starts the specified container. The container must be in a startable state.
    /// Use `Container::can_start()` to check if operation is valid.
    ///
    /// 指定されたコンテナを開始します。コンテナは開始可能な状態でなければなりません。
    /// 操作が有効かチェックするには`Container::can_start()`を使用してください。
    ///
    /// # Arguments
    /// * `id` - Container identifier
    ///
    /// # Errors
    /// * `DockaError::ContainerNotFound` - When container doesn't exist
    /// * `DockaError::InvalidInput` - When container cannot be started
    /// * `DockaError::DockerApi` - On Docker API communication errors
    async fn start_container(&self, id: &ContainerId) -> DockaResult<()>;

    /// Stop a running container
    /// 実行中のコンテナを停止
    ///
    /// Gracefully stops the specified container with default timeout.
    /// The container must be in a stoppable state.
    ///
    /// 指定されたコンテナをデフォルトタイムアウトで
    /// グレースフルに停止します。コンテナは停止可能な状態でなければなりません。
    ///
    /// # Arguments
    /// * `id` - Container identifier
    ///
    /// # Errors
    /// * `DockaError::ContainerNotFound` - When container doesn't exist
    /// * `DockaError::InvalidInput` - When container cannot be stopped
    /// * `DockaError::DockerApi` - On Docker API communication errors
    async fn stop_container(&self, id: &ContainerId) -> DockaResult<()>;

    /// Stop a running container with timeout
    /// タイムアウト付きで実行中のコンテナを停止
    ///
    /// Stops container with specified timeout. If timeout is reached,
    /// container will be forcefully killed.
    ///
    /// 指定されたタイムアウトでコンテナを停止します。タイムアウトに
    /// 達した場合、コンテナは強制的に削除されます。
    ///
    /// # Arguments
    /// * `id` - Container identifier
    /// * `timeout_seconds` - Timeout in seconds before force kill
    ///
    /// # Errors
    /// * `DockaError::ContainerNotFound` - When container doesn't exist
    /// * `DockaError::InvalidInput` - When timeout is invalid or container cannot be stopped
    /// * `DockaError::DockerApi` - On Docker API communication errors
    async fn stop_container_with_timeout(
        &self,
        id: &ContainerId,
        timeout_seconds: u32,
    ) -> DockaResult<()>;

    /// Remove a container
    /// コンテナを削除
    ///
    /// Removes the specified container. Container must be stopped first
    /// unless force is used. Use `Container::can_remove()` to check validity.
    ///
    /// 指定されたコンテナを削除します。forceが使用されない限り、
    /// コンテナは最初に停止されていなければなりません。
    /// 妥当性をチェックするには`Container::can_remove()`を使用してください。
    ///
    /// # Arguments
    /// * `id` - Container identifier
    /// * `force` - Whether to force remove running containers
    ///
    /// # Errors
    /// * `DockaError::ContainerNotFound` - When container doesn't exist
    /// * `DockaError::InvalidInput` - When container cannot be removed
    /// * `DockaError::DockerApi` - On Docker API communication errors
    async fn remove_container(&self, id: &ContainerId, force: bool) -> DockaResult<()>;

    /// Restart a container
    /// コンテナを再起動
    ///
    /// Restarts the specified container. This is equivalent to stop + start
    /// but handled atomically by Docker.
    ///
    /// 指定されたコンテナを再起動します。これはstop + startと
    /// 同等ですが、Dockerによってアトミックに処理されます。
    ///
    /// # Arguments
    /// * `id` - Container identifier
    ///
    /// # Errors
    /// * `DockaError::ContainerNotFound` - When container doesn't exist
    /// * `DockaError::InvalidInput` - When container cannot be restarted
    /// * `DockaError::DockerApi` - On Docker API communication errors
    async fn restart_container(&self, id: &ContainerId) -> DockaResult<()>;

    /// Pause a running container
    /// 実行中のコンテナを一時停止
    ///
    /// Pauses all processes in the specified container using cgroups freezer.
    /// Container must be running to be paused.
    ///
    /// cgroups freezerを使用して指定されたコンテナの全プロセスを一時停止します。
    /// コンテナは一時停止するために実行中でなければなりません。
    ///
    /// # Arguments
    /// * `id` - Container identifier
    ///
    /// # Errors
    /// * `DockaError::ContainerNotFound` - When container doesn't exist
    /// * `DockaError::InvalidInput` - When container cannot be paused
    /// * `DockaError::DockerApi` - On Docker API communication errors
    async fn pause_container(&self, id: &ContainerId) -> DockaResult<()>;

    /// Resume a paused container
    /// 一時停止中のコンテナを再開
    ///
    /// Unpauses all processes in the specified container.
    /// Container must be paused to be unpaused.
    ///
    /// 指定されたコンテナの全プロセスの一時停止を解除します。
    /// コンテナは一時停止解除するために一時停止中でなければなりません。
    ///
    /// # Arguments
    /// * `id` - Container identifier
    ///
    /// # Errors
    /// * `DockaError::ContainerNotFound` - When container doesn't exist
    /// * `DockaError::InvalidInput` - When container is not paused
    /// * `DockaError::DockerApi` - On Docker API communication errors
    async fn unpause_container(&self, id: &ContainerId) -> DockaResult<()>;

    // =========================================================================
    // Phase 2 Implementation Plans (Commented for future development)
    // Phase 2実装計画（将来の開発のためコメントアウト）
    // =========================================================================

    // The following methods will be implemented in Phase 2:
    // 以下のメソッドはPhase 2で実装予定です:
    //
    // async fn get_container_logs(&self, id: &ContainerId, lines: Option<usize>) -> DockaResult<Vec<String>>;
    // - Get container logs with optional line limit
    // - オプションの行制限付きでコンテナログを取得
    //
    // async fn get_container_stats(&self, id: &ContainerId) -> DockaResult<ContainerStats>;
    // - Get real-time resource usage statistics
    // - リアルタイムリソース使用統計を取得
    //
    // async fn exec_in_container(&self, id: &ContainerId, command: &[String]) -> DockaResult<ExecResult>;
    // - Execute command inside container
    // - コンテナ内でコマンドを実行
    //
    // async fn stream_container_events(&self) -> DockaResult<impl Stream<Item = ContainerEvent>>;
    // - Stream real-time container events
    // - リアルタイムコンテナイベントをストリーミング

    // =========================================================================
    // Phase 3 Future Extensions (Planned features)
    // Phase 3将来拡張（計画中の機能）
    // =========================================================================

    // async fn batch_start_containers(&self, ids: &[ContainerId]) -> DockaResult<Vec<DockaResult<()>>>;
    // - Batch operation to start multiple containers
    // - 複数コンテナを一括で開始する操作
    //
    // async fn batch_stop_containers(&self, ids: &[ContainerId]) -> DockaResult<Vec<DockaResult<()>>>;
    // - Batch operation to stop multiple containers
    // - 複数コンテナを一括で停止する操作
    //
    // async fn create_container(&self, config: &ContainerConfig) -> DockaResult<Container>;
    // - Create new container from configuration
    // - 設定から新しいコンテナを作成
}

/// Mock implementation for testing
/// テスト用Mock実装
///
/// This implementation provides a fake Docker repository for unit testing
/// without requiring actual Docker daemon or containers.
///
/// この実装は実際のDocker daemonやコンテナを必要とせずに
/// 単体テスト用の偽のDockerリポジトリを提供します。
///
/// # Thread Safety
///
/// The mock repository uses `Arc<RwLock<Vec<Container>>>` for thread-safe
/// access to the container storage, allowing concurrent reads and exclusive writes.
///
/// モックリポジトリはコンテナストレージへのスレッドセーフな
/// アクセスのために`Arc<RwLock<Vec<Container>>>`を使用し、
/// 並行読み取りと排他的書き込みを可能にします。
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
#[derive(Debug, Default)]
pub struct MockDockerRepository {
    /// In-memory container storage
    /// インメモリコンテナストレージ
    pub containers: std::sync::Arc<tokio::sync::RwLock<Vec<Container>>>,
}

#[cfg(test)]
impl MockDockerRepository {
    /// Create a new mock repository
    /// 新しいモックリポジトリを作成
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a container to the mock repository
    /// モックリポジトリにコンテナを追加
    pub async fn add_container(&self, container: Container) {
        let mut containers = self.containers.write().await;
        containers.push(container);
    }

    /// Clear all containers from the mock repository
    /// モックリポジトリから全コンテナをクリア
    pub async fn clear_containers(&self) {
        let mut containers = self.containers.write().await;
        containers.clear();
    }

    /// Get container count
    /// コンテナ数を取得
    pub async fn container_count(&self) -> usize {
        let containers = self.containers.read().await;
        containers.len()
    }
}

#[cfg(test)]
#[async_trait]
impl DockerRepository for MockDockerRepository {
    async fn list_containers(&self) -> DockaResult<Vec<Container>> {
        let containers = self.containers.read().await;
        Ok(containers.clone())
    }

    async fn list_containers_filtered(
        &self,
        filter: &ContainerFilter,
    ) -> DockaResult<Vec<Container>> {
        let containers = self.containers.read().await;
        Ok(containers
            .iter()
            .filter(|c| filter.matches(c))
            .cloned()
            .collect())
    }

    async fn get_container(&self, id: &ContainerId) -> DockaResult<Container> {
        let containers = self.containers.read().await;
        containers
            .iter()
            .find(|c| c.id == *id)
            .cloned()
            .ok_or_else(|| crate::error::DockaError::ContainerNotFound {
                name: id.to_string(),
            })
    }

    async fn start_container(&self, id: &ContainerId) -> DockaResult<()> {
        let mut containers = self.containers.write().await;
        if let Some(container) = containers.iter_mut().find(|c| c.id == *id) {
            if !container.can_start() {
                return Err(crate::error::DockaError::invalid_input(format!(
                    "Container {} cannot be started from status {}",
                    id, container.status
                )));
            }
            container.status = crate::domain::value_objects::ContainerStatus::Running;
            Ok(())
        } else {
            Err(crate::error::DockaError::ContainerNotFound {
                name: id.to_string(),
            })
        }
    }

    async fn stop_container(&self, id: &ContainerId) -> DockaResult<()> {
        self.stop_container_with_timeout(id, 10).await
    }

    async fn stop_container_with_timeout(
        &self,
        id: &ContainerId,
        _timeout_seconds: u32,
    ) -> DockaResult<()> {
        let mut containers = self.containers.write().await;
        if let Some(container) = containers.iter_mut().find(|c| c.id == *id) {
            if !container.can_stop() {
                return Err(crate::error::DockaError::invalid_input(format!(
                    "Container {} cannot be stopped from status {}",
                    id, container.status
                )));
            }
            container.status = crate::domain::value_objects::ContainerStatus::Stopped;
            Ok(())
        } else {
            Err(crate::error::DockaError::ContainerNotFound {
                name: id.to_string(),
            })
        }
    }

    async fn remove_container(&self, id: &ContainerId, force: bool) -> DockaResult<()> {
        let mut containers = self.containers.write().await;
        if let Some(pos) = containers.iter().position(|c| c.id == *id) {
            let container = &containers[pos];
            if !force && !container.can_remove() {
                return Err(crate::error::DockaError::invalid_input(format!(
                    "Container {} cannot be removed from status {} (use force=true to override)",
                    id, container.status
                )));
            }
            containers.remove(pos);
            Ok(())
        } else {
            Err(crate::error::DockaError::ContainerNotFound {
                name: id.to_string(),
            })
        }
    }

    async fn restart_container(&self, id: &ContainerId) -> DockaResult<()> {
        let mut containers = self.containers.write().await;
        if let Some(container) = containers.iter_mut().find(|c| c.id == *id) {
            if !container.can_restart() {
                return Err(crate::error::DockaError::invalid_input(format!(
                    "Container {} cannot be restarted from status {}",
                    id, container.status
                )));
            }
            container.status = crate::domain::value_objects::ContainerStatus::Running;
            Ok(())
        } else {
            Err(crate::error::DockaError::ContainerNotFound {
                name: id.to_string(),
            })
        }
    }

    async fn pause_container(&self, id: &ContainerId) -> DockaResult<()> {
        let mut containers = self.containers.write().await;
        if let Some(container) = containers.iter_mut().find(|c| c.id == *id) {
            if !container.can_pause() {
                return Err(crate::error::DockaError::invalid_input(format!(
                    "Container {} cannot be paused from status {}",
                    id, container.status
                )));
            }
            container.status = crate::domain::value_objects::ContainerStatus::Paused;
            Ok(())
        } else {
            Err(crate::error::DockaError::ContainerNotFound {
                name: id.to_string(),
            })
        }
    }

    async fn unpause_container(&self, id: &ContainerId) -> DockaResult<()> {
        let mut containers = self.containers.write().await;
        if let Some(container) = containers.iter_mut().find(|c| c.id == *id) {
            if !container.can_unpause() {
                return Err(crate::error::DockaError::invalid_input(format!(
                    "Container {} cannot be unpaused from status {}",
                    id, container.status
                )));
            }
            container.status = crate::domain::value_objects::ContainerStatus::Running;
            Ok(())
        } else {
            Err(crate::error::DockaError::ContainerNotFound {
                name: id.to_string(),
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entities::Container;
    use crate::domain::value_objects::{ContainerId, ContainerStatus};

    async fn create_test_container(id: &str, status: ContainerStatus) -> Container {
        Container::builder()
            .id(id)
            .name(format!("test-{}", id))
            .image("nginx:latest")
            .status(status)
            .build()
            .expect("Valid test container")
    }

    #[tokio::test]
    async fn test_mock_repository_list_containers() {
        // Test basic container listing
        // 基本的なコンテナ一覧のテスト
        let repo = MockDockerRepository::new();

        // Initially empty
        // 初期状態では空
        let containers = repo.list_containers().await.unwrap();
        assert!(containers.is_empty());

        // Add test containers
        // テストコンテナを追加
        let container1 = create_test_container("test-1", ContainerStatus::Running).await;
        let container2 = create_test_container("test-2", ContainerStatus::Stopped).await;

        repo.add_container(container1.clone()).await;
        repo.add_container(container2.clone()).await;

        // Verify listing
        // 一覧の確認
        let containers = repo.list_containers().await.unwrap();
        assert_eq!(containers.len(), 2);
        assert_eq!(repo.container_count().await, 2);
    }

    #[tokio::test]
    async fn test_mock_repository_filtered_listing() {
        // Test filtered container listing
        // フィルタ付きコンテナ一覧のテスト
        let repo = MockDockerRepository::new();

        let running_container = create_test_container("running-1", ContainerStatus::Running).await;
        let stopped_container = create_test_container("stopped-1", ContainerStatus::Stopped).await;

        repo.add_container(running_container).await;
        repo.add_container(stopped_container).await;

        // Filter for running containers only
        // 実行中のコンテナのみフィルタ
        let filter = crate::domain::entities::ContainerFilter::running_only();
        let running_containers = repo.list_containers_filtered(&filter).await.unwrap();

        assert_eq!(running_containers.len(), 1);
        assert_eq!(running_containers[0].status, ContainerStatus::Running);
    }

    #[tokio::test]
    async fn test_mock_repository_get_container() {
        // Test getting specific container
        // 特定コンテナ取得のテスト
        let repo = MockDockerRepository::new();
        let container = create_test_container("get-test", ContainerStatus::Running).await;
        let container_id = container.id.clone();

        repo.add_container(container.clone()).await;

        // Get existing container
        // 存在するコンテナの取得
        let retrieved = repo.get_container(&container_id).await.unwrap();
        assert_eq!(retrieved.id, container_id);
        assert_eq!(retrieved.status, ContainerStatus::Running);

        // Get non-existing container
        // 存在しないコンテナの取得
        let non_existing_id = ContainerId::new("non-existing").unwrap();
        let result = repo.get_container(&non_existing_id).await;
        assert!(result.is_err());

        if let Err(crate::error::DockaError::ContainerNotFound { name }) = result {
            assert_eq!(name, "non-existing");
        } else {
            panic!("Expected ContainerNotFound error");
        }
    }

    #[tokio::test]
    async fn test_mock_repository_start_container() {
        // Test container start operation with detailed debugging
        // 詳細デバッグ付きコンテナ開始操作のテスト
        let repo = MockDockerRepository::new();

        // Create container with explicit Stopped status
        // 明示的にStopped状態でコンテナを作成
        let container = create_test_container("start-test", ContainerStatus::Stopped).await;
        let container_id = container.id.clone();

        // Verify initial container state
        // 初期コンテナ状態の確認
        assert_eq!(container.status, ContainerStatus::Stopped);
        assert!(
            container.can_start(),
            "Container should be able to start from Stopped state"
        );

        // Add container to repository
        // リポジトリにコンテナを追加
        repo.add_container(container).await;

        // Verify container was added correctly
        // コンテナが正しく追加されたことを確認
        let stored_container = repo.get_container(&container_id).await.unwrap();
        assert_eq!(
            stored_container.status,
            ContainerStatus::Stopped,
            "Container should be Stopped after being added to repository"
        );

        // Start stopped container
        // 停止中のコンテナを開始
        let result = repo.start_container(&container_id).await;
        assert!(
            result.is_ok(),
            "Start operation should succeed: {:?}",
            result
        );

        // Verify status changed to Running
        // ステータスがRunningに変更されたことを確認
        let updated_container = repo.get_container(&container_id).await.unwrap();
        assert_eq!(
            updated_container.status,
            ContainerStatus::Running,
            "Container status should be Running after start operation"
        );

        // Try to start already running container (should fail)
        // 既に実行中のコンテナを開始しようとする（失敗すべき）
        let result = repo.start_container(&container_id).await;
        assert!(
            result.is_err(),
            "Starting an already running container should fail"
        );

        if let Err(crate::error::DockaError::InvalidInput { message }) = result {
            assert!(
                message.contains("cannot be started"),
                "Error message should indicate cannot be started"
            );
        } else {
            panic!("Expected InvalidInput error for starting running container");
        }
    }

    #[tokio::test]
    async fn test_mock_repository_stop_container() {
        // Test container stop operation with detailed debugging
        // 詳細デバッグ付きコンテナ停止操作のテスト
        let repo = MockDockerRepository::new();

        // Create container with explicit Running status
        // 明示的にRunning状態でコンテナを作成
        let container = create_test_container("stop-test", ContainerStatus::Running).await;
        let container_id = container.id.clone();

        // Verify initial container state
        // 初期コンテナ状態の確認
        assert_eq!(container.status, ContainerStatus::Running);
        assert!(
            container.can_stop(),
            "Container should be able to stop from Running state"
        );

        // Add container to repository
        // リポジトリにコンテナを追加
        repo.add_container(container).await;

        // Verify container was added correctly
        // コンテナが正しく追加されたことを確認
        let stored_container = repo.get_container(&container_id).await.unwrap();
        assert_eq!(
            stored_container.status,
            ContainerStatus::Running,
            "Container should be Running after being added to repository"
        );

        // Stop running container
        // 実行中のコンテナを停止
        let result = repo.stop_container(&container_id).await;
        assert!(
            result.is_ok(),
            "Stop operation should succeed: {:?}",
            result
        );

        // Verify status changed to Stopped
        // ステータスがStoppedに変更されたことを確認
        let updated_container = repo.get_container(&container_id).await.unwrap();
        assert_eq!(
            updated_container.status,
            ContainerStatus::Stopped,
            "Container status should be Stopped after stop operation"
        );

        // Try to stop already stopped container (should fail)
        // 既に停止中のコンテナを停止しようとする（失敗すべき）
        let result = repo.stop_container(&container_id).await;
        assert!(
            result.is_err(),
            "Stopping an already stopped container should fail"
        );

        if let Err(crate::error::DockaError::InvalidInput { message }) = result {
            assert!(
                message.contains("cannot be stopped"),
                "Error message should indicate cannot be stopped"
            );
        } else {
            panic!("Expected InvalidInput error for stopping stopped container");
        }
    }

    #[tokio::test]
    async fn test_mock_repository_remove_container() {
        // Test container removal
        // コンテナ削除のテスト
        let repo = MockDockerRepository::new();
        let stopped_container =
            create_test_container("remove-stopped", ContainerStatus::Stopped).await;
        let running_container =
            create_test_container("remove-running", ContainerStatus::Running).await;

        let stopped_id = stopped_container.id.clone();
        let running_id = running_container.id.clone();

        repo.add_container(stopped_container).await;
        repo.add_container(running_container).await;

        assert_eq!(repo.container_count().await, 2);

        // Remove stopped container (should succeed)
        // 停止中のコンテナを削除（成功すべき）
        let result = repo.remove_container(&stopped_id, false).await;
        assert!(result.is_ok());
        assert_eq!(repo.container_count().await, 1);

        // Try to remove running container without force (should fail)
        // forceなしで実行中のコンテナを削除しようとする（失敗すべき）
        let result = repo.remove_container(&running_id, false).await;
        assert!(result.is_err());
        assert_eq!(repo.container_count().await, 1);

        // Force remove running container (should succeed)
        // 実行中のコンテナを強制削除（成功すべき）
        let result = repo.remove_container(&running_id, true).await;
        assert!(result.is_ok());
        assert_eq!(repo.container_count().await, 0);
    }

    #[tokio::test]
    async fn test_mock_repository_pause_unpause() {
        // Test container pause/unpause operations
        // コンテナ一時停止/再開操作のテスト
        let repo = MockDockerRepository::new();
        let container = create_test_container("pause-test", ContainerStatus::Running).await;
        let container_id = container.id.clone();

        repo.add_container(container).await;

        // Pause running container
        // 実行中のコンテナを一時停止
        let result = repo.pause_container(&container_id).await;
        assert!(result.is_ok());

        let paused = repo.get_container(&container_id).await.unwrap();
        assert_eq!(paused.status, ContainerStatus::Paused);

        // Unpause paused container
        // 一時停止中のコンテナを再開
        let result = repo.unpause_container(&container_id).await;
        assert!(result.is_ok());

        let unpaused = repo.get_container(&container_id).await.unwrap();
        assert_eq!(unpaused.status, ContainerStatus::Running);
    }

    #[tokio::test]
    async fn test_mock_repository_restart_container() {
        // Test container restart operation
        // コンテナ再起動操作のテスト
        let repo = MockDockerRepository::new();
        let container = create_test_container("restart-test", ContainerStatus::Stopped).await;
        let container_id = container.id.clone();

        repo.add_container(container).await;

        // Restart container
        // コンテナを再起動
        let result = repo.restart_container(&container_id).await;
        assert!(result.is_ok());

        // Verify status is running
        // ステータスが実行中であることを確認
        let restarted = repo.get_container(&container_id).await.unwrap();
        assert_eq!(restarted.status, ContainerStatus::Running);
    }

    #[tokio::test]
    async fn test_mock_repository_error_cases() {
        // Test various error scenarios
        // 様々なエラーシナリオのテスト
        let repo = MockDockerRepository::new();
        let non_existing_id = ContainerId::new("non-existing").unwrap();

        // Operations on non-existing container should return ContainerNotFound
        // 存在しないコンテナに対する操作はContainerNotFoundを返すべき
        let operations = vec![
            repo.start_container(&non_existing_id),
            repo.stop_container(&non_existing_id),
            repo.restart_container(&non_existing_id),
            repo.pause_container(&non_existing_id),
            repo.unpause_container(&non_existing_id),
        ];

        for operation in operations {
            let result = operation.await;
            assert!(result.is_err());
            assert!(matches!(
                result.unwrap_err(),
                crate::error::DockaError::ContainerNotFound { .. }
            ));
        }

        // Remove non-existing container should also fail
        // 存在しないコンテナの削除も失敗すべき
        let result = repo.remove_container(&non_existing_id, false).await;
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            crate::error::DockaError::ContainerNotFound { .. }
        ));
    }

    #[tokio::test]
    async fn test_mock_repository_state_validation() {
        // Test that operations respect container state rules
        // 操作がコンテナ状態ルールを尊重することをテスト
        let repo = MockDockerRepository::new();

        // Test all invalid state transitions
        // 全ての無効な状態遷移をテスト
        let test_cases = vec![
            ("running-cannot-start", ContainerStatus::Running, "start"),
            ("stopped-cannot-stop", ContainerStatus::Stopped, "stop"),
            ("stopped-cannot-pause", ContainerStatus::Stopped, "pause"),
            (
                "running-cannot-unpause",
                ContainerStatus::Running,
                "unpause",
            ),
        ];

        for (id, initial_status, operation) in test_cases {
            let container = create_test_container(id, initial_status.clone()).await;
            let container_id = container.id.clone();
            repo.add_container(container).await;

            let result = match operation {
                "start" => repo.start_container(&container_id).await,
                "stop" => repo.stop_container(&container_id).await,
                "pause" => repo.pause_container(&container_id).await,
                "unpause" => repo.unpause_container(&container_id).await,
                _ => panic!("Unknown operation: {}", operation),
            };

            assert!(
                result.is_err(),
                "Operation {} should fail for container in {} state",
                operation,
                initial_status
            );

            if let Err(crate::error::DockaError::InvalidInput { message }) = result {
                assert!(message.contains("cannot be"));
            } else {
                panic!("Expected InvalidInput error for operation {}", operation);
            }

            // Clean up for next test
            // 次のテストのためにクリーンアップ
            repo.clear_containers().await;
        }
    }

    #[tokio::test]
    async fn test_repository_trait_send_sync() {
        // Test that the trait and implementation are Send + Sync
        // traitと実装がSend + Syncであることをテスト
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<MockDockerRepository>();

        // Test that we can use the repository across async boundaries
        // 非同期境界を越えてリポジトリを使用できることをテスト
        let repo = std::sync::Arc::new(MockDockerRepository::new());
        let repo_clone = repo.clone();

        let handle = tokio::spawn(async move {
            let container = create_test_container("async-test", ContainerStatus::Running).await;
            repo_clone.add_container(container).await;
            repo_clone.container_count().await
        });

        let count = handle.await.unwrap();
        assert_eq!(count, 1);
        assert_eq!(repo.container_count().await, 1);
    }

    #[tokio::test]
    async fn test_mock_repository_timeout_operations() {
        // Test timeout-specific operations
        // タイムアウト固有操作のテスト
        let repo = MockDockerRepository::new();
        let container = create_test_container("timeout-test", ContainerStatus::Running).await;
        let container_id = container.id.clone();

        repo.add_container(container).await;

        // Test stop with timeout
        // タイムアウト付き停止のテスト
        let result = repo.stop_container_with_timeout(&container_id, 30).await;
        assert!(result.is_ok());

        let stopped = repo.get_container(&container_id).await.unwrap();
        assert_eq!(stopped.status, ContainerStatus::Stopped);
    }

    #[tokio::test]
    async fn test_mock_repository_comprehensive_workflow() {
        // Test a complete container lifecycle workflow
        // 完全なコンテナライフサイクルワークフローのテスト
        let repo = MockDockerRepository::new();
        let container = create_test_container("workflow-test", ContainerStatus::Stopped).await;
        let container_id = container.id.clone();

        repo.add_container(container).await;
        assert_eq!(repo.container_count().await, 1);

        // 1. Start container
        // 1. コンテナ開始
        let result = repo.start_container(&container_id).await;
        assert!(result.is_ok());
        let running = repo.get_container(&container_id).await.unwrap();
        assert_eq!(running.status, ContainerStatus::Running);

        // 2. Pause container
        // 2. コンテナ一時停止
        let result = repo.pause_container(&container_id).await;
        assert!(result.is_ok());
        let paused = repo.get_container(&container_id).await.unwrap();
        assert_eq!(paused.status, ContainerStatus::Paused);

        // 3. Unpause container
        // 3. コンテナ一時停止解除
        let result = repo.unpause_container(&container_id).await;
        assert!(result.is_ok());
        let unpaused = repo.get_container(&container_id).await.unwrap();
        assert_eq!(unpaused.status, ContainerStatus::Running);

        // 4. Stop container
        // 4. コンテナ停止
        let result = repo.stop_container(&container_id).await;
        assert!(result.is_ok());
        let stopped = repo.get_container(&container_id).await.unwrap();
        assert_eq!(stopped.status, ContainerStatus::Stopped);

        // 5. Remove container
        // 5. コンテナ削除
        let result = repo.remove_container(&container_id, false).await;
        assert!(result.is_ok());
        assert_eq!(repo.container_count().await, 0);

        // 6. Verify container is gone
        // 6. コンテナが削除されたことを確認
        let result = repo.get_container(&container_id).await;
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            crate::error::DockaError::ContainerNotFound { .. }
        ));
    }

    #[tokio::test]
    async fn test_container_status_transition_logic() {
        // Test the underlying status transition logic
        // 根底の状態遷移ロジックのテスト

        // Test that Stopped containers can be started
        // Stoppedコンテナが開始可能であることをテスト
        let stopped_status = ContainerStatus::Stopped;
        assert!(
            stopped_status.can_start(),
            "Stopped status should allow start"
        );

        // Test that Running containers can be stopped
        // Runningコンテナが停止可能であることをテスト
        let running_status = ContainerStatus::Running;
        assert!(
            running_status.can_stop(),
            "Running status should allow stop"
        );

        // Test state transitions
        // 状態遷移のテスト
        assert!(stopped_status.can_transition_to(&ContainerStatus::Starting));
        assert!(ContainerStatus::Starting.can_transition_to(&ContainerStatus::Running));
        assert!(running_status.can_transition_to(&ContainerStatus::Stopping));
        assert!(ContainerStatus::Stopping.can_transition_to(&ContainerStatus::Stopped));
    }
}
