// tests/integration_docker.rs
// Integration tests for Docker API functionality
// `Docker` `API`機能の統合テスト

//! Integration tests for `BollardDockerRepository`
//! `BollardDockerRepository`の統合テスト
//!
//! These tests require a running Docker daemon and will create/manipulate
//! real containers for testing. They are designed to be safe and clean up
//! after themselves.
//!
//! これらのテストは動作する`Docker` `daemon`を必要とし、テストのために
//! 実際のコンテナを作成/操作します。安全であり、自動的にクリーンアップ
//! するよう設計されています。
//!
//! # Prerequisites
//!
//! - `Docker` `daemon` must be running
//! - Current user must have `Docker` permissions
//! - Internet connection for pulling test images
//!
//! # Running the tests
//!
//! ```bash
//! # Run all integration tests
//! cargo test --test integration_docker
//!
//! # Run with output
//! cargo test --test integration_docker -- --nocapture
//!
//! # Skip integration tests in normal testing
//! cargo test --lib
//! ```

use docka::domain::repositories::DockerRepository;
use docka::domain::value_objects::{ContainerId, ContainerStatus};
use docka::infrastructure::BollardDockerRepository;
use docka::{DockaError, DockaResult};
use std::time::Duration;
use tokio::time::sleep;

// Import the new OpenAPI generated types and builders
// 新しいOpenAPI生成型とビルダーをインポート
use bollard::models::ContainerCreateBody;
use bollard::query_parameters::{CreateContainerOptions, CreateContainerOptionsBuilder};

/// Test configuration constants
/// テスト設定定数
const TEST_IMAGE: &str = "alpine:latest";
const TEST_CONTAINER_PREFIX: &str = "docka-test-";
const TEST_TIMEOUT: Duration = Duration::from_secs(30);

/// Integration test helper utilities
/// 統合テスト用ヘルパーユーティリティ
struct IntegrationTestHelper {
    repo: BollardDockerRepository,
    created_containers: Vec<ContainerId>,
}

impl IntegrationTestHelper {
    /// Create a new test helper
    /// 新しいテストヘルパーを作成
    #[must_use]
    async fn new() -> DockaResult<Self> {
        let repo = BollardDockerRepository::new().await?;
        Ok(Self {
            repo,
            created_containers: Vec::new(),
        })
    }

    /// Create `CreateContainerOptions` using the new Builder API
    /// 新しいBuilder `API`を使用して`CreateContainerOptions`を作成
    ///
    /// This helper method encapsulates the creation of `CreateContainerOptions`
    /// using the new `OpenAPI` generated Builder pattern.
    ///
    /// このヘルパーメソッドは新しい`OpenAPI`生成Builderパターンを使用した
    /// `CreateContainerOptions`の作成をカプセル化します。
    fn create_container_options(name: &str) -> CreateContainerOptions {
        CreateContainerOptionsBuilder::default().name(name).build()
    }

    /// Create `ContainerCreateBody` using the new OpenAPI model
    /// 新しい`OpenAPI`モデルを使用して`ContainerCreateBody`を作成
    ///
    /// This helper method creates the container configuration body
    /// using the new OpenAPI generated model instead of the deprecated `Config`.
    ///
    /// このヘルパーメソッドは非推奨の`Config`の代わりに、
    /// 新しい`OpenAPI`生成モデルを使用してコンテナ設定ボディを作成します。
    fn create_container_body() -> ContainerCreateBody {
        ContainerCreateBody {
            image: Some(TEST_IMAGE.to_string()),
            cmd: Some(vec!["sleep".to_string(), "300".to_string()]), // Sleep for 5 minutes
            attach_stdout: Some(false),
            attach_stderr: Some(false),
            attach_stdin: Some(false),
            tty: Some(false),
            open_stdin: Some(false),
            stdin_once: Some(false),
            ..Default::default()
        }
    }

    /// Create a test container with unique name
    /// 一意の名前でテストコンテナを作成
    async fn create_test_container(&mut self, suffix: &str) -> DockaResult<ContainerId> {
        let container_name = format!("{}{}", TEST_CONTAINER_PREFIX, suffix);

        // Use new `OpenAPI` generated types for container creation
        // コンテナ作成に新しい`OpenAPI`生成型を使用
        let config = Self::create_container_body();
        let options = Some(Self::create_container_options(&container_name));

        let create_response = self
            .repo
            .client()
            .create_container(options, config)
            .await
            .map_err(DockaError::DockerApi)?;

        let container_id = ContainerId::new(create_response.id)?;
        self.created_containers.push(container_id.clone());

        Ok(container_id)
    }

    /// Clean up all created test containers
    /// 作成した全テストコンテナをクリーンアップ
    async fn cleanup(&mut self) {
        for container_id in &self.created_containers {
            // Try to stop and remove container (ignore errors)
            // コンテナの停止と削除を試行（エラーは無視）
            let _ = self.repo.stop_container(container_id).await;
            let _ = self.repo.remove_container(container_id, true).await;
        }
        self.created_containers.clear();
    }

    /// Wait for container to reach expected status
    /// コンテナが期待されるステータスに達するまで待機
    ///
    /// This method now uses flexible status matching to handle struct variants
    /// like `ContainerStatus::Exited { exit_code }` properly.
    ///
    /// このメソッドは構造体バリアント（`ContainerStatus::Exited { exit_code }`等）を
    /// 適切に処理するため、柔軟なステータスマッチングを使用します。
    async fn wait_for_status(
        &self,
        container_id: &ContainerId,
        expected_status: ContainerStatus,
        timeout: Duration,
    ) -> DockaResult<()> {
        let start = std::time::Instant::now();

        loop {
            if start.elapsed() > timeout {
                return Err(DockaError::internal(format!(
                    "Timeout waiting for container {} to reach status {}",
                    container_id, expected_status
                )));
            }

            let container = self.repo.get_container(container_id).await?;

            // Use flexible status matching instead of direct equality
            // 直接の等価比較の代わりに柔軟なステータスマッチングを使用
            if Self::status_matches(&container.status, &expected_status) {
                return Ok(());
            }

            sleep(Duration::from_millis(500)).await;
        }
    }

    /// Wait for container to reach any stopped state (Stopped or Exited)
    /// コンテナが任意の停止状態（StoppedまたはExited）に達するまで待機
    ///
    /// This is a specialized method for waiting for containers to stop,
    /// regardless of whether they stop cleanly or exit with an error.
    ///
    /// これはコンテナが正常に停止するかエラーで終了するかに関係なく、
    /// コンテナの停止を待機する専用メソッドです。
    async fn wait_for_container_stopped(
        &self,
        container_id: &ContainerId,
        timeout: Duration,
    ) -> DockaResult<()> {
        let start = std::time::Instant::now();

        loop {
            if start.elapsed() > timeout {
                return Err(DockaError::internal(format!(
                    "Timeout waiting for container {} to stop",
                    container_id
                )));
            }

            let container = self.repo.get_container(container_id).await?;

            // Check if container is in any stopped state
            // コンテナが任意の停止状態にあるかチェック
            if container.is_stopped() {
                return Ok(());
            }

            sleep(Duration::from_millis(500)).await;
        }
    }

    /// Check if container status matches expected pattern
    /// コンテナステータスが期待されるパターンにマッチするかチェック
    ///
    /// This method provides flexible matching for container statuses,
    /// particularly useful for handling struct variants like `Exited { exit_code }`.
    ///
    /// このメソッドはコンテナステータスの柔軟なマッチングを提供し、
    /// 特に`Exited { exit_code }`のような構造体バリアントの処理に有用です。
    fn status_matches(actual: &ContainerStatus, expected: &ContainerStatus) -> bool {
        use ContainerStatus::*;

        match (actual, expected) {
            // Exact matches for simple variants
            // シンプルなバリアントの完全一致
            (Running, Running)
            | (Stopped, Stopped)
            | (Starting, Starting)
            | (Stopping, Stopping)
            | (Paused, Paused)
            | (Restarting, Restarting)
            | (Removing, Removing)
            | (Dead, Dead)
            | (Created, Created) => true,

            // Flexible matching for Exited status
            // Exitedステータスの柔軟なマッチング
            (Exited { .. }, Exited { .. }) => true,

            // Special case: treat specific exit codes if needed
            // 特別ケース: 必要に応じて特定の終了コードを処理
            // (
            //     Exited {
            //         exit_code: actual_code,
            //     },
            //     Exited {
            //         exit_code: expected_code,
            //     },
            // ) => actual_code == expected_code,

            // No match for different status types
            // 異なるステータス型はマッチしない
            _ => false,
        }
    }

    /// Check if container status matches expected exit code exactly
    /// コンテナステータスが期待される終了コードと完全に一致するかチェック
    ///
    /// This method provides strict matching for cases where specific exit codes matter.
    /// Use this when you need to verify the exact exit code of a stopped container.
    ///
    /// このメソッドは特定の終了コードが重要なケースで厳密なマッチングを提供します。
    /// 停止したコンテナの正確な終了コードを検証する必要がある場合に使用してください。
    #[allow(dead_code)] // Reserved for future use
    fn status_matches_exact(actual: &ContainerStatus, expected: &ContainerStatus) -> bool {
        // Use PartialEq for exact matching including exit codes
        // 終了コードを含む完全一致にはPartialEqを使用
        actual == expected
    }

    /// Wait for container to reach running state
    /// コンテナが実行状態に達するまで待機
    async fn wait_for_running(
        &self,
        container_id: &ContainerId,
        timeout: Duration,
    ) -> DockaResult<()> {
        self.wait_for_status(container_id, ContainerStatus::Running, timeout)
            .await
    }

    // Note: wait_for_created() method was removed as it's not currently used
    // in any test cases. Docker containers typically transition from Created
    // to Running state immediately in our test scenarios.
    // If needed in the future, it can be easily re-implemented as:
    // async fn wait_for_created(&self, container_id: &ContainerId, timeout: Duration) -> DockaResult<()> {
    //     self.wait_for_status(container_id, ContainerStatus::Created, timeout).await
    // }
    //
    // 注意: wait_for_created()メソッドは現在のテストケースで使用されていないため削除しました。
    // Dockerコンテナは通常、テストシナリオでCreated状態からRunning状態に即座に遷移します。
    // 将来必要になった場合は、上記のように簡単に再実装できます。
}

impl Drop for IntegrationTestHelper {
    fn drop(&mut self) {
        // Cleanup in a blocking context (best effort)
        // ブロッキングコンテキストでのクリーンアップ（ベストエフォート）
        if !self.created_containers.is_empty() {
            println!(
                "Warning: {} test containers may need manual cleanup",
                self.created_containers.len()
            );
        }
    }
}

/// Check if `Docker` is available for testing
/// テストで`Docker`が利用可能かチェック
async fn check_docker_available() -> bool {
    match BollardDockerRepository::new().await {
        Ok(_) => true,
        Err(_) => {
            println!("Skipping Docker integration tests: Docker daemon not available");
            false
        }
    }
}

#[tokio::test]
async fn test_docker_connection() {
    // Test basic `Docker` connection
    // 基本的な`Docker`接続のテスト
    if !check_docker_available().await {
        return;
    }

    let result = BollardDockerRepository::new().await;
    assert!(result.is_ok(), "Should connect to Docker daemon");

    let repo = result.unwrap();

    // Test that we can list containers (even if empty)
    // コンテナをリストできることをテスト（空でも）
    let containers = repo.list_containers().await;
    assert!(containers.is_ok(), "Should be able to list containers");
}

#[tokio::test]
async fn test_list_containers_integration() {
    // Test container listing with real `Docker`
    // 実際の`Docker`でのコンテナリストのテスト
    if !check_docker_available().await {
        return;
    }

    let mut helper = IntegrationTestHelper::new().await.unwrap();

    // Get initial container count
    // 初期コンテナ数を取得
    let initial_containers = helper.repo.list_containers().await.unwrap();
    let initial_count = initial_containers.len();

    // Create a test container
    // テストコンテナを作成
    let _container_id = helper.create_test_container("list-test").await.unwrap();

    // List containers again
    // 再度コンテナをリスト
    let containers = helper.repo.list_containers().await.unwrap();
    assert_eq!(
        containers.len(),
        initial_count + 1,
        "Should have one more container"
    );

    // Find our test container
    // テストコンテナを見つける
    let test_container = containers
        .iter()
        .find(|c| c.name.starts_with(TEST_CONTAINER_PREFIX))
        .expect("Should find test container");

    assert_eq!(test_container.image, TEST_IMAGE);
    assert!(matches!(
        test_container.status,
        ContainerStatus::Created | ContainerStatus::Running
    ));

    helper.cleanup().await;
}

#[tokio::test]
async fn test_container_lifecycle_integration() {
    // Test complete container lifecycle: create -> start -> stop -> remove
    // 完全なコンテナライフサイクルのテスト: 作成 -> 開始 -> 停止 -> 削除
    if !check_docker_available().await {
        return;
    }

    let mut helper = IntegrationTestHelper::new().await.unwrap();

    // Create test container
    // テストコンテナを作成
    let container_id = helper
        .create_test_container("lifecycle-test")
        .await
        .unwrap();

    // Verify container exists and is created
    // コンテナが存在し、作成されていることを確認
    let container = helper.repo.get_container(&container_id).await.unwrap();
    assert_eq!(container.id, container_id);
    assert!(container.can_start());

    // Start the container
    // コンテナを開始
    let result = helper.repo.start_container(&container_id).await;
    assert!(result.is_ok(), "Should start container successfully");

    // Wait for container to be running
    // コンテナが実行中になるまで待機
    helper
        .wait_for_running(&container_id, TEST_TIMEOUT)
        .await
        .unwrap();

    // Verify container is running
    // コンテナが実行中であることを確認
    let running_container = helper.repo.get_container(&container_id).await.unwrap();
    assert!(running_container.is_running());

    // Stop the container
    // コンテナを停止
    let result = helper.repo.stop_container(&container_id).await;
    assert!(result.is_ok(), "Should stop container successfully");

    // Wait for container to be stopped (using flexible stopped state matching)
    // コンテナが停止するまで待機（柔軟な停止状態マッチングを使用）
    helper
        .wait_for_container_stopped(&container_id, TEST_TIMEOUT)
        .await
        .unwrap();

    // Verify container is stopped (could be Stopped or Exited)
    // コンテナが停止していることを確認（StoppedまたはExitedの可能性）
    let stopped_container = helper.repo.get_container(&container_id).await.unwrap();
    assert!(
        stopped_container.is_stopped(),
        "Container should be stopped, actual status: {}",
        stopped_container.status
    );

    // Remove the container
    // コンテナを削除
    let result = helper.repo.remove_container(&container_id, false).await;
    assert!(result.is_ok(), "Should remove container successfully");

    // Verify container is removed
    // コンテナが削除されたことを確認
    let result = helper.repo.get_container(&container_id).await;
    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        DockaError::ContainerNotFound { .. }
    ));

    // Clear from helper to avoid cleanup attempt
    // ヘルパーからクリアしてクリーンアップ試行を回避
    helper.created_containers.clear();
}

#[tokio::test]
async fn test_container_restart_integration() {
    // Test container restart functionality
    // コンテナ再起動機能のテスト
    if !check_docker_available().await {
        return;
    }

    let mut helper = IntegrationTestHelper::new().await.unwrap();

    // Create and start container
    // コンテナを作成して開始
    let container_id = helper.create_test_container("restart-test").await.unwrap();
    helper.repo.start_container(&container_id).await.unwrap();

    helper
        .wait_for_running(&container_id, TEST_TIMEOUT)
        .await
        .unwrap();

    // Get container before restart
    // 再起動前のコンテナを取得
    let before_restart = helper.repo.get_container(&container_id).await.unwrap();
    assert!(before_restart.is_running());

    // Restart the container
    // コンテナを再起動
    let result = helper.repo.restart_container(&container_id).await;
    assert!(result.is_ok(), "Should restart container successfully");

    // Wait for container to be running again
    // コンテナが再び実行中になるまで待機
    helper
        .wait_for_running(&container_id, TEST_TIMEOUT)
        .await
        .unwrap();

    // Verify container is running after restart
    // 再起動後にコンテナが実行中であることを確認
    let after_restart = helper.repo.get_container(&container_id).await.unwrap();
    assert!(after_restart.is_running());

    helper.cleanup().await;
}

#[tokio::test]
async fn test_error_handling_integration() {
    // Test error handling with non-existent containers
    // 存在しないコンテナでのエラーハンドリングテスト
    if !check_docker_available().await {
        return;
    }

    let helper = IntegrationTestHelper::new().await.unwrap();
    let non_existent_id = ContainerId::new("non-existent-container-123").unwrap();

    // Test get non-existent container
    // 存在しないコンテナの取得テスト
    let result = helper.repo.get_container(&non_existent_id).await;
    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        DockaError::ContainerNotFound { .. }
    ));

    // Test start non-existent container
    // 存在しないコンテナの開始テスト
    let result = helper.repo.start_container(&non_existent_id).await;
    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        DockaError::ContainerNotFound { .. }
    ));

    // Test stop non-existent container
    // 存在しないコンテナの停止テスト
    let result = helper.repo.stop_container(&non_existent_id).await;
    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        DockaError::ContainerNotFound { .. }
    ));
}

#[tokio::test]
async fn test_invalid_operations_integration() {
    // Test invalid operations (e.g., starting already running container)
    // 無効な操作のテスト（例：既に実行中のコンテナの開始）
    if !check_docker_available().await {
        return;
    }

    let mut helper = IntegrationTestHelper::new().await.unwrap();

    // Create and start container
    // コンテナを作成して開始
    let container_id = helper
        .create_test_container("invalid-ops-test")
        .await
        .unwrap();
    helper.repo.start_container(&container_id).await.unwrap();

    helper
        .wait_for_running(&container_id, TEST_TIMEOUT)
        .await
        .unwrap();

    // Try to start already running container
    // 既に実行中のコンテナを開始しようとする
    let result = helper.repo.start_container(&container_id).await;
    assert!(
        result.is_err(),
        "Should not start already running container"
    );
    assert!(matches!(
        result.unwrap_err(),
        DockaError::InvalidInput { .. }
    ));

    helper.cleanup().await;
}

// Unit tests for helper functions and conversion logic
// ヘルパー関数と変換ロジックの単体テスト

#[cfg(test)]
mod unit_tests {
    use super::*;
    use bollard::models::ContainerSummary;
    use chrono::Utc;
    use std::collections::HashMap;

    #[test]
    fn test_status_matches() {
        // Test flexible status matching
        // 柔軟なステータスマッチングのテスト

        // Exact matches
        // 完全一致
        assert!(IntegrationTestHelper::status_matches(
            &ContainerStatus::Running,
            &ContainerStatus::Running
        ));

        assert!(IntegrationTestHelper::status_matches(
            &ContainerStatus::Stopped,
            &ContainerStatus::Stopped
        ));

        // Flexible Exited matching (any exit code matches any exit code pattern)
        // 柔軟なExitedマッチング（任意の終了コードが任意の終了コードパターンにマッチ）
        assert!(IntegrationTestHelper::status_matches(
            &ContainerStatus::Exited { exit_code: 0 },
            &ContainerStatus::Exited { exit_code: 1 }
        ));

        assert!(IntegrationTestHelper::status_matches(
            &ContainerStatus::Exited { exit_code: 127 },
            &ContainerStatus::Exited { exit_code: 0 }
        ));

        // Non-matches
        // 非マッチ
        assert!(!IntegrationTestHelper::status_matches(
            &ContainerStatus::Running,
            &ContainerStatus::Stopped
        ));

        assert!(!IntegrationTestHelper::status_matches(
            &ContainerStatus::Running,
            &ContainerStatus::Exited { exit_code: 0 }
        ));
    }

    /// Create a test `ContainerSummary` for testing
    /// テスト用の`ContainerSummary`を作成
    fn create_test_container_summary(
        id: &str,
        name: &str,
        image: &str,
        status: &str,
    ) -> ContainerSummary {
        let mut labels = HashMap::new();
        labels.insert("env".to_string(), "test".to_string());
        labels.insert("version".to_string(), "1.0".to_string());

        ContainerSummary {
            id: Some(id.to_string()),
            names: Some(vec![format!("/{}", name)]),
            image: Some(image.to_string()),
            status: Some(status.to_string()),
            created: Some(Utc::now().timestamp()),
            command: Some("/bin/bash -c sleep infinity".to_string()),
            labels: Some(labels),
            ..Default::default()
        }
    }

    #[test]
    fn test_create_container_options_builder() {
        // Test the new `CreateContainerOptions` builder pattern
        // 新しい`CreateContainerOptions`ビルダーパターンのテスト
        let options = IntegrationTestHelper::create_container_options("test-container");

        // Verify that the builder creates a valid options object
        // ビルダーが有効なオプションオブジェクトを作成することを確認
        // Note: Due to the builder pattern, we can't directly inspect internal values,
        // but we can verify it compiles and creates the expected type
        // 注意：ビルダーパターンのため内部値を直接検査できませんが、
        // コンパイルし、期待される型を作成することを確認できます
        drop(options); // Just verify it compiles and has the correct type
    }

    #[test]
    fn test_create_container_body() {
        // Test the new `ContainerCreateBody` creation
        // 新しい`ContainerCreateBody`作成のテスト
        let body = IntegrationTestHelper::create_container_body();

        // Verify key properties are set correctly
        // 主要なプロパティが正しく設定されていることを確認
        assert_eq!(body.image, Some(TEST_IMAGE.to_string()));
        assert_eq!(body.cmd, Some(vec!["sleep".to_string(), "300".to_string()]));
        assert_eq!(body.attach_stdout, Some(false));
        assert_eq!(body.attach_stderr, Some(false));
        assert_eq!(body.attach_stdin, Some(false));
        assert_eq!(body.tty, Some(false));
    }

    #[test]
    fn test_convert_container_success() {
        // Test successful container conversion
        // 成功するコンテナ変換のテスト
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
        assert!(container.command.as_ref().is_none_or(|c| c.is_empty())); // Should be empty or None
    }

    #[test]
    fn test_convert_container_missing_id() {
        // Test container conversion fails when `ID` is missing
        // `ID`が不足している場合のコンテナ変換失敗テスト
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

        // Check that `created_at` is between before and after conversion
        // `created_at`が変換前後の時刻の間にあることを確認
        assert!(
            container.created_at >= before_conversion && container.created_at <= after_conversion,
            "Missing timestamp should be filled with current time"
        );
    }

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
}
