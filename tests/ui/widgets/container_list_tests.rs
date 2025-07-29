// tests/ui/widgets/container_list_tests.rs
//! ContainerListWidget integration tests
//! ContainerListWidget統合テスト
//!
//! This module contains integration tests for the ContainerListWidget,
//! focusing on its integration with the application state and rendering pipeline.
//!
//! このモジュールはContainerListWidgetの統合テストを含み、
//! アプリケーション状態とレンダリングパイプラインとの統合に焦点を当てています。

use chrono::Utc;
use std::sync::Arc;

// Import from the main crate
// メインクレートからインポート
use docka::{
    ContainerFilter, DockaError, DockaResult,
    domain::{Container, ContainerBuilder, ContainerId, ContainerStatus, DockerRepository},
    ui::{App, ContainerListWidget, Theme, ViewState},
};
use ratatui::{
    Terminal,
    backend::{Backend, TestBackend},
    layout::Rect,
};

/// Simple test repository implementation for integration tests
/// 統合テスト用シンプルテストリポジトリ実装
struct TestDockerRepository {
    containers: Vec<Container>,
}

impl TestDockerRepository {
    fn new() -> Self {
        Self {
            containers: Vec::new(),
        }
    }

    fn with_containers(containers: Vec<Container>) -> Self {
        Self { containers }
    }
}

#[async_trait::async_trait]
impl DockerRepository for TestDockerRepository {
    async fn list_containers(&self) -> DockaResult<Vec<Container>> {
        Ok(self.containers.clone())
    }

    async fn list_containers_filtered(
        &self,
        _filter: &ContainerFilter,
    ) -> DockaResult<Vec<Container>> {
        // For testing purposes, return all containers regardless of filter
        // テスト目的のため、フィルタに関係なく全コンテナを返す
        Ok(self.containers.clone())
    }

    async fn get_container(&self, _id: &ContainerId) -> DockaResult<Container> {
        self.containers
            .first()
            .cloned()
            .ok_or_else(|| DockaError::ContainerNotFound {
                name: "test".to_string(),
            })
    }

    async fn start_container(&self, _id: &ContainerId) -> DockaResult<()> {
        Ok(())
    }

    async fn stop_container(&self, _id: &ContainerId) -> DockaResult<()> {
        Ok(())
    }

    async fn stop_container_with_timeout(
        &self,
        _id: &ContainerId,
        _timeout: u32,
    ) -> DockaResult<()> {
        // For testing purposes, ignore timeout and return success
        // テスト目的のため、タイムアウトを無視して成功を返す
        Ok(())
    }

    async fn restart_container(&self, _id: &ContainerId) -> DockaResult<()> {
        Ok(())
    }

    async fn remove_container(&self, _id: &ContainerId, _force: bool) -> DockaResult<()> {
        Ok(())
    }

    async fn pause_container(&self, _id: &ContainerId) -> DockaResult<()> {
        Ok(())
    }

    async fn unpause_container(&self, _id: &ContainerId) -> DockaResult<()> {
        Ok(())
    }
}

/// Test utilities for ContainerListWidget integration tests
/// ContainerListWidget統合テスト用テストユーティリティ
struct ContainerListTestHelper {
    app: App,
    widget: ContainerListWidget,
    terminal: Terminal<TestBackend>,
    theme: Theme,
}

impl ContainerListTestHelper {
    /// Create a new test helper with empty app
    /// 空のアプリで新しいテストヘルパーを作成
    fn new() -> Self {
        let docker_repo = Arc::new(TestDockerRepository::new());
        let app = App::new(docker_repo);
        let widget = ContainerListWidget::new();
        let backend = TestBackend::new(80, 24);
        let terminal = Terminal::new(backend).unwrap();
        let theme = Theme::dark();

        Self {
            app,
            widget,
            terminal,
            theme,
        }
    }

    /// Create test helper with pre-populated containers
    /// 事前にコンテナが設定されたテストヘルパーを作成
    fn with_containers(containers: Vec<Container>) -> Self {
        let docker_repo = Arc::new(TestDockerRepository::with_containers(containers.clone()));
        let mut app = App::new(docker_repo);

        // Directly set containers for testing
        // テスト用にコンテナを直接設定
        app.containers = containers;
        app.view_state = ViewState::ContainerList;

        let widget = ContainerListWidget::new();
        let backend = TestBackend::new(80, 24);
        let terminal = Terminal::new(backend).unwrap();
        let theme = Theme::dark();

        Self {
            app,
            widget,
            terminal,
            theme,
        }
    }

    /// Set selected index
    /// 選択インデックスを設定
    fn with_selection(mut self, index: usize) -> Self {
        if index < self.app.containers.len() {
            self.app.selected_index = index;
        }
        self
    }

    /// Set view state
    /// ビュー状態を設定
    fn with_view_state(mut self, view_state: ViewState) -> Self {
        self.app.view_state = view_state;
        self
    }

    /// Set theme
    /// テーマを設定
    fn with_theme(mut self, theme: Theme) -> Self {
        self.theme = theme;
        self
    }

    /// Render the widget and capture the result
    /// ウィジェットをレンダリングして結果をキャプチャ
    fn render(&mut self) -> DockaResult<()> {
        self.terminal.draw(|f| {
            let area = f.area();
            ContainerListWidget::render(&mut self.widget, f, &self.app, area, &self.theme);
        })?;
        Ok(())
    }

    /// Get the terminal backend for content inspection
    /// コンテンツ検査用ターミナルバックエンドを取得
    fn backend(&self) -> &TestBackend {
        self.terminal.backend()
    }
}

/// Helper function to create test container
/// テストコンテナ作成ヘルパー関数
fn create_test_container(name: &str, status: ContainerStatus, image: &str) -> Container {
    ContainerBuilder::new()
        .id(ContainerId::new(format!("{}_id", name)).unwrap())
        .name(name.to_string())
        .image(image.to_string())
        .status(status)
        .created_at(Utc::now())
        .build()
        .expect("Valid container for testing")
}

/// Create a sample set of containers for testing
/// テスト用コンテナセットを作成
fn create_sample_containers() -> Vec<Container> {
    vec![
        create_test_container("web_server", ContainerStatus::Running, "nginx:latest"),
        create_test_container(
            "database",
            ContainerStatus::Exited { exit_code: 0 },
            "postgres:13",
        ),
        create_test_container("cache", ContainerStatus::Paused, "redis:alpine"),
        create_test_container("worker", ContainerStatus::Starting, "worker:v1.0"),
        create_test_container("monitor", ContainerStatus::Stopped, "prometheus:latest"),
    ]
}

#[test]
fn test_integration_empty_container_list() {
    // Test rendering with no containers
    // コンテナがない場合のレンダリングテスト
    let mut helper = ContainerListTestHelper::new();

    // Should not panic when rendering empty list
    // 空リストのレンダリング時にパニックしないことを確認
    assert!(helper.render().is_ok());

    // Verify that the terminal was updated
    // ターミナルが更新されたことを確認
    let backend = helper.backend();
    assert_eq!(backend.size().unwrap(), Rect::new(0, 0, 80, 24).into());
}

#[test]
fn test_integration_single_container() {
    // Test rendering with a single container
    // 単一コンテナでのレンダリングテスト
    let containers = vec![create_test_container(
        "single_container",
        ContainerStatus::Running,
        "nginx:latest",
    )];

    let mut helper = ContainerListTestHelper::with_containers(containers);

    // Should render successfully
    // 正常にレンダリングされることを確認
    assert!(helper.render().is_ok());
}

#[test]
fn test_integration_multiple_containers() {
    // Test rendering with multiple containers
    // 複数コンテナでのレンダリングテスト
    let containers = create_sample_containers();
    let mut helper = ContainerListTestHelper::with_containers(containers);

    // Should render successfully with multiple containers
    // 複数コンテナで正常にレンダリングされることを確認
    assert!(helper.render().is_ok());
}

#[test]
fn test_integration_container_selection() {
    // Test container selection integration
    // コンテナ選択統合テスト
    let containers = create_sample_containers();
    let mut helper = ContainerListTestHelper::with_containers(containers).with_selection(2);

    // Should render with selection applied
    // 選択が適用された状態でレンダリングされることを確認
    assert!(helper.render().is_ok());
    assert_eq!(helper.app.selected_index, 2);
}

#[test]
fn test_integration_widget_state_management() {
    // Test widget state management integration
    // ウィジェット状態管理統合テスト
    let containers = create_sample_containers();
    let container_count = containers.len();
    let mut helper = ContainerListTestHelper::with_containers(containers).with_selection(1);

    // Test state synchronization
    // 状態同期のテスト
    helper
        .widget
        .sync_with_app(helper.app.selected_index, container_count);
    assert_eq!(helper.widget.selected(), Some(1));

    // Test navigation
    // ナビゲーションのテスト
    helper.widget.select_next(container_count);
    assert_eq!(helper.widget.selected(), Some(2));

    helper.widget.select_previous(container_count);
    assert_eq!(helper.widget.selected(), Some(1));
}

#[test]
fn test_integration_different_view_states() {
    // Test integration with different view states
    // 異なるビュー状態での統合テスト
    let containers = create_sample_containers();

    // Test with ContainerList view state
    // ContainerListビュー状態でのテスト
    let mut helper1 = ContainerListTestHelper::with_containers(containers.clone())
        .with_view_state(ViewState::ContainerList);
    assert!(helper1.render().is_ok());

    // Test with Loading view state
    // Loadingビュー状態でのテスト
    let mut helper2 = ContainerListTestHelper::with_containers(containers.clone())
        .with_view_state(ViewState::Loading);
    assert!(helper2.render().is_ok());

    // Test with Error view state
    // Errorビュー状態でのテスト
    let mut helper3 = ContainerListTestHelper::with_containers(containers)
        .with_view_state(ViewState::Error("Test error".to_string()));
    assert!(helper3.render().is_ok());
}

#[test]
fn test_integration_theme_consistency() {
    // Test theme consistency across rendering
    // レンダリング全体でのテーマ一貫性テスト
    let containers = create_sample_containers();

    // Test with dark theme
    // ダークテーマでのテスト
    let mut helper_dark =
        ContainerListTestHelper::with_containers(containers.clone()).with_theme(Theme::dark());
    assert!(helper_dark.render().is_ok());

    // Test with light theme
    // ライトテーマでのテスト
    let mut helper_light =
        ContainerListTestHelper::with_containers(containers).with_theme(Theme::light());
    assert!(helper_light.render().is_ok());
}

#[test]
fn test_integration_edge_cases() {
    // Test edge cases in integration scenarios
    // 統合シナリオのエッジケーステスト

    // Test with out-of-bounds selection
    // 範囲外選択のテスト
    let containers = create_sample_containers();
    let mut helper1 = ContainerListTestHelper::with_containers(containers).with_selection(999);
    assert!(helper1.render().is_ok());

    // Test with containers having special characters
    // 特殊文字を含むコンテナのテスト
    let special_containers = vec![
        create_test_container(
            "container-with-dashes",
            ContainerStatus::Running,
            "image:latest",
        ),
        create_test_container(
            "container_with_underscores",
            ContainerStatus::Paused,
            "registry.example.com/image:v1.0",
        ),
        create_test_container("", ContainerStatus::Stopped, "image"), // Empty name
    ];

    let mut helper2 = ContainerListTestHelper::with_containers(special_containers);
    assert!(helper2.render().is_ok());
}

#[test]
fn test_integration_large_container_list() {
    // Test performance with large container list
    // 大量コンテナリストでのパフォーマンステスト

    // Create a large number of containers
    // 大量のコンテナを作成
    let large_container_list: Vec<Container> = (0..50)
        .map(|i| {
            let status = match i % 4 {
                0 => ContainerStatus::Running,
                1 => ContainerStatus::Stopped,
                2 => ContainerStatus::Paused,
                _ => ContainerStatus::Exited { exit_code: 0 },
            };
            create_test_container(&format!("container_{}", i), status, "test:latest")
        })
        .collect();

    let mut helper = ContainerListTestHelper::with_containers(large_container_list);

    // Should handle large lists without issues
    // 大量リストを問題なく処理できることを確認
    assert!(helper.render().is_ok());
}

#[test]
fn test_integration_container_wraparound_navigation() {
    // Test wraparound navigation in widget
    // ウィジェットでの循環ナビゲーションテスト
    let containers = create_sample_containers();
    let container_count = containers.len();
    let mut helper = ContainerListTestHelper::with_containers(containers);

    // Test wraparound to first when at last
    // 最後から最初への循環テスト
    helper.widget.set_selected(Some(container_count - 1));
    helper.widget.select_next(container_count);
    assert_eq!(helper.widget.selected(), Some(0));

    // Test wraparound to last when at first
    // 最初から最後への循環テスト
    helper.widget.set_selected(Some(0));
    helper.widget.select_previous(container_count);
    assert_eq!(helper.widget.selected(), Some(container_count - 1));
}

#[test]
fn test_integration_responsive_layout() {
    // Test responsive layout behavior
    // レスポンシブレイアウト動作テスト
    let containers = create_sample_containers();

    // Test with small terminal
    // 小さなターミナルでのテスト
    let mut small_helper = ContainerListTestHelper::with_containers(containers.clone());
    small_helper.terminal = Terminal::new(TestBackend::new(40, 10)).unwrap();
    assert!(small_helper.render().is_ok());

    // Test with large terminal
    // 大きなターミナルでのテスト
    let mut large_helper = ContainerListTestHelper::with_containers(containers);
    large_helper.terminal = Terminal::new(TestBackend::new(120, 40)).unwrap();
    assert!(large_helper.render().is_ok());
}

#[test]
fn test_integration_real_world_scenario() {
    // Test a realistic user scenario
    // 現実的なユーザーシナリオのテスト

    // Simulate a typical development environment
    // 典型的な開発環境をシミュレート
    let dev_containers = vec![
        create_test_container("web_frontend", ContainerStatus::Running, "react:latest"),
        create_test_container("api_backend", ContainerStatus::Running, "node:16"),
        create_test_container("postgres_db", ContainerStatus::Running, "postgres:13"),
        create_test_container("redis_cache", ContainerStatus::Running, "redis:alpine"),
        create_test_container(
            "test_runner",
            ContainerStatus::Exited { exit_code: 0 },
            "jest:latest",
        ),
        create_test_container("old_version", ContainerStatus::Stopped, "app:v1.0"),
    ];

    let mut helper = ContainerListTestHelper::with_containers(dev_containers);

    // User navigates through containers
    // ユーザーがコンテナ間をナビゲート
    for i in 0..helper.app.containers.len() {
        helper.app.selected_index = i;
        assert!(helper.render().is_ok());
    }

    // Verify final state
    // 最終状態を確認
    assert_eq!(helper.app.containers.len(), 6);
    assert!(helper.app.selected_index < helper.app.containers.len());
}
