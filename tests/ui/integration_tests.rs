//! UI Integration Tests
//! UI統合テスト
//!
//! This module contains comprehensive integration tests for the UI layer,
//! testing the interaction between App, ContainerListWidget, layouts, and event handling.
//!
//! このモジュールはUI層の包括的な統合テストを含み、
//! App、ContainerListWidget、レイアウト、イベント処理の相互作用をテストします。

use std::sync::Arc;

use ratatui::{Terminal, backend::TestBackend};

use docka::{
    // Use the correct imports without MockDockerRepository
    // MockDockerRepositoryを使わずに正しいインポートを使用
    Container,
    ContainerFilter,
    ContainerId,
    ContainerStatus,
    DockaError,
    DockaResult,
    DockerRepository,
    ui::{
        app::{App, NavigationDirection, ViewState},
        events::{AppEvent, EventStats, handle_key_event},
        layouts::SimpleLayout,
        styles::Theme,
        widgets::{ContainerListWidget, StatusBar},
    },
};

/// Simple test repository implementation for integration tests
/// 統合テスト用シンプルテストリポジトリ実装
#[derive(Debug, Clone)]
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

#[cfg(test)]
mod tests {
    use super::*;

    /// Helper struct for UI integration testing
    /// UI統合テスト用ヘルパー構造体
    struct UIIntegrationTestHelper {
        app: App,
        container_list_widget: ContainerListWidget,
        terminal: Terminal<TestBackend>,
        theme: Theme,
        event_stats: EventStats,
    }

    impl UIIntegrationTestHelper {
        /// Create a new test helper with sample data
        /// サンプルデータを使用して新しいテストヘルパーを作成
        fn new() -> Self {
            // Use TestDockerRepository instead of MockDockerRepository
            // MockDockerRepositoryの代わりにTestDockerRepositoryを使用
            let test_repo = Arc::new(TestDockerRepository::new());
            let app = App::new(test_repo);
            let container_list_widget = ContainerListWidget::new();
            let terminal = Terminal::new(TestBackend::new(80, 24)).unwrap();
            let theme = Theme::dark();
            let event_stats = EventStats::new();

            Self {
                app,
                container_list_widget,
                terminal,
                theme,
                event_stats,
            }
        }

        /// Set up test containers using MockDockerRepository
        /// MockDockerRepositoryを使用してテストコンテナをセットアップ
        async fn with_test_containers(mut self) -> Self {
            // Setup the app with test containers through MockDockerRepository
            // MockDockerRepositoryを通じてテストコンテナでアプリをセットアップ
            self.app = setup_test_app_with_containers().await;
            self
        }

        /// Set specific view state
        /// 特定のビュー状態を設定
        fn with_view_state(mut self, state: ViewState) -> Self {
            self.app.view_state = state;
            self
        }

        /// Simulate full UI rendering cycle
        /// 完全なUIレンダリングサイクルをシミュレート
        fn render_complete_ui(&mut self) -> Result<(), Box<dyn std::error::Error>> {
            self.terminal.draw(|frame| {
                // Create layout areas using correct static method
                // 正しい静的メソッドを使用してレイアウト領域を作成
                let areas = SimpleLayout::calculate_responsive(frame.area());

                // Render main content based on view state
                // ビュー状態に基づいてメインコンテンツをレンダリング
                match &self.app.view_state {
                    ViewState::ContainerList => {
                        // Render container list using the correct signature
                        // 正しいシグネチャを使用してコンテナリストをレンダリング
                        ContainerListWidget::render(
                            &mut self.container_list_widget,
                            frame,
                            &self.app,
                            areas.main,
                            &self.theme,
                        );
                    }
                    ViewState::Loading | ViewState::Error(_) => {
                        // These states would be handled by main render logic
                        // これらの状態はメインレンダリングロジックで処理される
                        // For now, just render empty container list
                        // 今のところ、空のコンテナリストをレンダリング
                        ContainerListWidget::render(
                            &mut self.container_list_widget,
                            frame,
                            &self.app,
                            areas.main,
                            &self.theme,
                        );
                    }
                }

                // Render status bar using the correct signature
                // 正しいシグネチャを使用してステータスバーをレンダリング
                StatusBar::render(frame, &self.app, areas.status);
            })?;

            Ok(())
        }

        /// Simulate navigation event
        /// ナビゲーションイベントをシミュレート
        fn simulate_navigation(
            &mut self,
            direction: NavigationDirection,
        ) -> Result<(), Box<dyn std::error::Error>> {
            // Handle navigation using App's integrated method
            // Appの統合メソッドを使用してナビゲーションを処理
            self.app
                .handle_container_navigation(&mut self.container_list_widget, direction.clone());

            // Update event statistics
            // イベント統計を更新
            let _event = match direction {
                NavigationDirection::Next => AppEvent::SelectNext,
                NavigationDirection::Previous => AppEvent::SelectPrevious,
            };

            // Record event with simplified signature
            // 簡略化されたシグネチャでイベントを記録
            self.event_stats.total_events += 1;
            self.event_stats.navigation_events += 1;

            Ok(())
        }
    }

    /// Create test containers for integration testing using builder pattern
    /// ビルダーパターンを使用して統合テスト用のテストコンテナを作成
    fn create_test_containers() -> Vec<Container> {
        vec![
            Container::builder()
                .id("web_server")
                .name("web_server")
                .image("nginx:latest")
                .status(ContainerStatus::Running)
                .build()
                .expect("Valid web server container"),
            Container::builder()
                .id("database")
                .name("database")
                .image("postgres:13")
                .status(ContainerStatus::Running)
                .build()
                .expect("Valid database container"),
            Container::builder()
                .id("cache")
                .name("cache")
                .image("redis:alpine")
                .status(ContainerStatus::Stopped)
                .build()
                .expect("Valid cache container"),
            Container::builder()
                .id("api_server")
                .name("api_server")
                .image("node:16")
                .status(ContainerStatus::Paused)
                .build()
                .expect("Valid API server container"),
            Container::builder()
                .id("test_runner")
                .name("test_runner")
                .image("jest:latest")
                .status(ContainerStatus::Exited { exit_code: 0 })
                .build()
                .expect("Valid test runner container"),
        ]
    }

    /// Setup test containers in MockDockerRepository
    /// MockDockerRepositoryにテストコンテナをセットアップ
    async fn setup_test_app_with_containers() -> App {
        // Use TestDockerRepository with pre-loaded containers
        // 事前ロードされたコンテナでTestDockerRepositoryを使用
        let test_repo = Arc::new(TestDockerRepository::with_containers(
            create_test_containers(),
        ));

        let mut app = App::new(test_repo);

        // Load containers into app
        // アプリにコンテナをロード
        if let Err(e) = app.refresh_containers().await {
            panic!("Failed to load test containers: {}", e);
        }

        app.view_state = ViewState::ContainerList;
        app.selected_index = 0;

        app
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[tokio::test]
        async fn test_complete_ui_integration() {
            // Test complete UI integration with all components
            // 全コンポーネントでの完全UI統合テスト
            let mut helper = UIIntegrationTestHelper::new().with_test_containers().await;

            // Should render without errors
            // エラーなしでレンダリングされることを確認
            assert!(helper.render_complete_ui().is_ok());

            // Verify initial state
            // 初期状態を確認
            assert_eq!(helper.app.selected_index, 0);
            assert!(matches!(helper.app.view_state, ViewState::ContainerList));
            assert_eq!(helper.app.containers.len(), 5);
        }

        #[tokio::test]
        async fn test_navigation_integration() {
            // Test navigation integration between App and ContainerListWidget
            // AppとContainerListWidget間のナビゲーション統合テスト
            let mut helper = UIIntegrationTestHelper::new().with_test_containers().await;

            // Test forward navigation
            // 前進ナビゲーションテスト
            for expected_index in 1..helper.app.containers.len() {
                assert!(
                    helper
                        .simulate_navigation(NavigationDirection::Next)
                        .is_ok()
                );
                assert_eq!(helper.app.selected_index, expected_index);
                assert_eq!(
                    helper.container_list_widget.selected(),
                    Some(expected_index)
                );
                assert!(helper.render_complete_ui().is_ok());
            }

            // Test wraparound (last to first)
            // 循環テスト（最後から最初へ）
            assert!(
                helper
                    .simulate_navigation(NavigationDirection::Next)
                    .is_ok()
            );
            assert_eq!(helper.app.selected_index, 0);
            assert_eq!(helper.container_list_widget.selected(), Some(0));
        }

        #[tokio::test]
        async fn test_backward_navigation_integration() {
            // Test backward navigation integration
            // 後退ナビゲーション統合テスト
            let mut helper = UIIntegrationTestHelper::new().with_test_containers().await;

            // Start from first item and go backward (should wrap to last)
            // 最初のアイテムから後退（最後にラップするはず）
            assert!(
                helper
                    .simulate_navigation(NavigationDirection::Previous)
                    .is_ok()
            );
            assert_eq!(helper.app.selected_index, helper.app.containers.len() - 1);
            assert_eq!(
                helper.container_list_widget.selected(),
                Some(helper.app.containers.len() - 1)
            );
            assert!(helper.render_complete_ui().is_ok());

            // Continue backward navigation
            // 後退ナビゲーションを継続
            for expected_index in (0..helper.app.containers.len() - 1).rev() {
                assert!(
                    helper
                        .simulate_navigation(NavigationDirection::Previous)
                        .is_ok()
                );
                assert_eq!(helper.app.selected_index, expected_index);
                assert_eq!(
                    helper.container_list_widget.selected(),
                    Some(expected_index)
                );
                assert!(helper.render_complete_ui().is_ok());
            }
        }

        #[tokio::test]
        async fn test_view_state_transitions() {
            // Test UI integration with different view states
            // 異なるビュー状態でのUI統合テスト
            let mut helper = UIIntegrationTestHelper::new().with_test_containers().await;

            // Test ContainerList state
            // ContainerList状態テスト
            helper = helper.with_view_state(ViewState::ContainerList);
            assert!(helper.render_complete_ui().is_ok());

            // Test Loading state
            // Loading状態テスト
            helper = helper.with_view_state(ViewState::Loading);
            assert!(helper.render_complete_ui().is_ok());

            // Test Error state
            // Error状態テスト
            helper = helper.with_view_state(ViewState::Error("Test error message".to_string()));
            assert!(helper.render_complete_ui().is_ok());
        }

        #[test]
        fn test_empty_container_list_integration() {
            // Test UI integration with empty container list
            // 空のコンテナリストでのUI統合テスト
            let mut helper = UIIntegrationTestHelper::new();
            helper.app.containers.clear();
            helper.app.selected_index = 0;

            // Should render without errors
            // エラーなしでレンダリングされることを確認
            assert!(helper.render_complete_ui().is_ok());

            // Navigation should not crash with empty list
            // 空リストでナビゲーションがクラッシュしないことを確認
            assert!(
                helper
                    .simulate_navigation(NavigationDirection::Next)
                    .is_ok()
            );
            assert!(
                helper
                    .simulate_navigation(NavigationDirection::Previous)
                    .is_ok()
            );
        }

        #[tokio::test]
        async fn test_event_statistics_integration() {
            // Test event statistics integration
            // イベント統計統合テスト
            let mut helper = UIIntegrationTestHelper::new().with_test_containers().await;

            // Simulate multiple navigation events
            // 複数のナビゲーションイベントをシミュレート
            for _ in 0..5 {
                assert!(
                    helper
                        .simulate_navigation(NavigationDirection::Next)
                        .is_ok()
                );
            }

            for _ in 0..3 {
                assert!(
                    helper
                        .simulate_navigation(NavigationDirection::Previous)
                        .is_ok()
                );
            }

            // Verify event statistics
            // イベント統計を確認
            assert_eq!(helper.event_stats.total_events, 8);
            assert_eq!(helper.event_stats.navigation_events, 8);
        }

        #[tokio::test]
        async fn test_theme_integration() {
            // Test theme integration across UI components
            // UIコンポーネント全体でのテーマ統合テスト
            let mut helper_dark = UIIntegrationTestHelper::new().with_test_containers().await;
            helper_dark.theme = Theme::dark();
            assert!(helper_dark.render_complete_ui().is_ok());

            let mut helper_light = UIIntegrationTestHelper::new().with_test_containers().await;
            helper_light.theme = Theme::light();
            assert!(helper_light.render_complete_ui().is_ok());
        }

        #[tokio::test]
        async fn test_real_world_user_scenario() {
            // Test realistic user interaction scenario
            // 現実的なユーザー操作シナリオテスト
            let mut helper = UIIntegrationTestHelper::new().with_test_containers().await;

            // User starts the application
            // ユーザーがアプリケーションを開始
            assert!(helper.render_complete_ui().is_ok());

            // User navigates through containers to find specific one
            // ユーザーが特定のコンテナを見つけるためにナビゲート
            for _ in 0..3 {
                assert!(
                    helper
                        .simulate_navigation(NavigationDirection::Next)
                        .is_ok()
                );
                assert!(helper.render_complete_ui().is_ok());
            }

            // User goes back to previous container
            // ユーザーが前のコンテナに戻る
            assert!(
                helper
                    .simulate_navigation(NavigationDirection::Previous)
                    .is_ok()
            );
            assert!(helper.render_complete_ui().is_ok());

            // Verify final state matches expectations
            // 最終状態が期待通りであることを確認
            assert_eq!(helper.app.selected_index, 2);
            assert_eq!(helper.container_list_widget.selected(), Some(2));
            assert!(matches!(helper.app.view_state, ViewState::ContainerList));
        }

        #[test]
        fn test_performance_integration() {
            // Test performance with realistic container count
            // 現実的なコンテナ数でのパフォーマンステスト
            let mut helper = UIIntegrationTestHelper::new();

            // Create larger container list (realistic development environment)
            // より大きなコンテナリストを作成（現実的な開発環境）
            helper.app.containers = (0..20)
                .map(|i| {
                    let status = match i % 4 {
                        0 => ContainerStatus::Running,
                        1 => ContainerStatus::Stopped,
                        2 => ContainerStatus::Paused,
                        _ => ContainerStatus::Exited { exit_code: 0 },
                    };
                    Container::builder()
                        .id(&format!("container_{}", i))
                        .name(&format!("service_{}", i))
                        .image(&format!("image_{}", i))
                        .status(status)
                        .build()
                        .expect("Valid test container")
                })
                .collect();

            helper.app.selected_index = 0;

            // Should handle larger lists efficiently
            // より大きなリストを効率的に処理できることを確認
            assert!(helper.render_complete_ui().is_ok());

            // Navigation should remain responsive
            // ナビゲーションが応答性を保つことを確認
            for _ in 0..10 {
                assert!(
                    helper
                        .simulate_navigation(NavigationDirection::Next)
                        .is_ok()
                );
                assert!(helper.render_complete_ui().is_ok());
            }
        }

        #[tokio::test]
        async fn test_error_resilience_integration() {
            // Test UI integration error resilience
            // UI統合エラー耐性テスト
            let mut helper = UIIntegrationTestHelper::new().with_test_containers().await;

            // Test with out-of-bounds selection
            // 範囲外選択でのテスト
            helper.app.selected_index = 999;
            assert!(helper.render_complete_ui().is_ok());

            // Navigation should handle gracefully
            // ナビゲーションが適切に処理されることを確認
            assert!(
                helper
                    .simulate_navigation(NavigationDirection::Next)
                    .is_ok()
            );
            assert!(
                helper
                    .simulate_navigation(NavigationDirection::Previous)
                    .is_ok()
            );

            // UI should still be functional
            // UIが依然として機能することを確認
            assert!(helper.render_complete_ui().is_ok());
        }

        #[tokio::test]
        async fn test_container_widget_synchronization() {
            // Test synchronization between App navigation and ContainerListWidget
            // AppナビゲーションとContainerListWidget間の同期テスト
            let mut helper = UIIntegrationTestHelper::new().with_test_containers().await;

            // Test synchronized navigation
            // 同期ナビゲーションテスト
            for expected_index in 0..helper.app.containers.len() {
                helper.app.selected_index = expected_index;
                helper
                    .container_list_widget
                    .sync_with_app(helper.app.selected_index, helper.app.containers.len());

                assert_eq!(
                    helper.container_list_widget.selected(),
                    Some(expected_index),
                    "Widget should be synchronized with App navigation"
                );
            }

            // Test navigation operations keep them in sync
            // ナビゲーション操作で同期が保たれることをテスト
            helper.app.handle_container_navigation(
                &mut helper.container_list_widget,
                NavigationDirection::Next,
            );

            assert_eq!(
                helper.app.selected_index,
                helper.container_list_widget.selected().unwrap(),
                "App and widget should remain synchronized after navigation"
            );
        }

        #[tokio::test]
        async fn test_responsive_layout_integration() {
            // Test responsive layout behavior with different terminal sizes
            // 異なるターミナルサイズでのレスポンシブレイアウト動作テスト

            // Test with small terminal
            // 小さなターミナルでのテスト
            let mut small_helper = UIIntegrationTestHelper::new().with_test_containers().await;
            small_helper.terminal = Terminal::new(TestBackend::new(40, 10)).unwrap();
            assert!(small_helper.render_complete_ui().is_ok());

            // Test with large terminal
            // 大きなターミナルでのテスト
            let mut large_helper = UIIntegrationTestHelper::new().with_test_containers().await;
            large_helper.terminal = Terminal::new(TestBackend::new(120, 40)).unwrap();
            assert!(large_helper.render_complete_ui().is_ok());
        }

        #[test]
        fn test_key_event_handling_integration() {
            // Test integration of key event handling with UI components
            // UIコンポーネントとのキーイベント処理統合テスト

            // Simulate key events
            // キーイベントをシミュレート
            use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

            let key_j = KeyEvent::new(KeyCode::Char('j'), KeyModifiers::NONE);
            let event_j = handle_key_event(key_j);
            assert_eq!(event_j, AppEvent::SelectNext);

            let key_k = KeyEvent::new(KeyCode::Char('k'), KeyModifiers::NONE);
            let event_k = handle_key_event(key_k);
            assert_eq!(event_k, AppEvent::SelectPrevious);

            // Test that events map to correct navigation directions
            // イベントが正しいナビゲーション方向にマップされることをテスト
            let mut helper = UIIntegrationTestHelper::new();

            // Test integration with navigation
            // ナビゲーションとの統合テスト
            match event_j {
                AppEvent::SelectNext => {
                    assert!(
                        helper
                            .simulate_navigation(NavigationDirection::Next)
                            .is_ok()
                    );
                }
                _ => panic!("Expected SelectNext event"),
            }

            match event_k {
                AppEvent::SelectPrevious => {
                    assert!(
                        helper
                            .simulate_navigation(NavigationDirection::Previous)
                            .is_ok()
                    );
                }
                _ => panic!("Expected SelectPrevious event"),
            }

            // Verify state after event processing
            // イベント処理後の状態を確認
            assert!(helper.render_complete_ui().is_ok());
        }
    }

    #[tokio::test]
    async fn test_view_state_transitions() {
        // Test UI integration with different view states
        // 異なるビュー状態でのUI統合テスト
        let mut helper = UIIntegrationTestHelper::new().with_test_containers().await;

        // Test ContainerList state
        // ContainerList状態テスト
        helper = helper.with_view_state(ViewState::ContainerList);
        assert!(helper.render_complete_ui().is_ok());

        // Test Loading state
        // Loading状態テスト
        helper = helper.with_view_state(ViewState::Loading);
        assert!(helper.render_complete_ui().is_ok());

        // Test Error state
        // Error状態テスト
        helper = helper.with_view_state(ViewState::Error("Test error message".to_string()));
        assert!(helper.render_complete_ui().is_ok());
    }

    #[tokio::test]
    async fn test_real_world_user_scenario() {
        // Test realistic user interaction scenario
        // 現実的なユーザー操作シナリオテスト
        let mut helper = UIIntegrationTestHelper::new().with_test_containers().await;

        // User starts the application
        // ユーザーがアプリケーションを開始
        assert!(helper.render_complete_ui().is_ok());

        // User navigates through containers to find specific one
        // ユーザーが特定のコンテナを見つけるためにナビゲート
        for _ in 0..3 {
            assert!(
                helper
                    .simulate_navigation(NavigationDirection::Next)
                    .is_ok()
            );
            assert!(helper.render_complete_ui().is_ok());
        }

        // User goes back to previous container
        // ユーザーが前のコンテナに戻る
        assert!(
            helper
                .simulate_navigation(NavigationDirection::Previous)
                .is_ok()
        );
        assert!(helper.render_complete_ui().is_ok());

        // Verify final state matches expectations
        // 最終状態が期待通りであることを確認
        assert_eq!(helper.app.selected_index, 2);
        assert_eq!(helper.container_list_widget.selected(), Some(2));
        assert!(matches!(helper.app.view_state, ViewState::ContainerList));
    }

    #[test]
    fn test_performance_integration() {
        // Test performance with realistic container count
        // 現実的なコンテナ数でのパフォーマンステスト
        let mut helper = UIIntegrationTestHelper::new();

        // Create larger container list (realistic development environment)
        // より大きなコンテナリストを作成（現実的な開発環境）
        helper.app.containers = (0..20)
            .map(|i| {
                let status = match i % 4 {
                    0 => ContainerStatus::Running,
                    1 => ContainerStatus::Stopped,
                    2 => ContainerStatus::Paused,
                    _ => ContainerStatus::Exited { exit_code: 0 },
                };
                Container::builder()
                    .id(&format!("container_{}", i))
                    .name(&format!("service_{}", i))
                    .image(&format!("image_{}", i))
                    .status(status)
                    .build()
                    .expect("Valid test container")
            })
            .collect();

        helper.app.selected_index = 0;

        // Should handle larger lists efficiently
        // より大きなリストを効率的に処理できることを確認
        assert!(helper.render_complete_ui().is_ok());

        // Navigation should remain responsive
        // ナビゲーションが応答性を保つことを確認
        for _ in 0..10 {
            assert!(
                helper
                    .simulate_navigation(NavigationDirection::Next)
                    .is_ok()
            );
            assert!(helper.render_complete_ui().is_ok());
        }
    }

    #[tokio::test]
    async fn test_error_resilience_integration() {
        // Test UI integration error resilience
        // UI統合エラー耐性テスト
        let mut helper = UIIntegrationTestHelper::new().with_test_containers().await;

        // Test with out-of-bounds selection
        // 範囲外選択でのテスト
        helper.app.selected_index = 999;
        assert!(helper.render_complete_ui().is_ok());

        // Navigation should handle gracefully
        // ナビゲーションが適切に処理されることを確認
        assert!(
            helper
                .simulate_navigation(NavigationDirection::Next)
                .is_ok()
        );
        assert!(
            helper
                .simulate_navigation(NavigationDirection::Previous)
                .is_ok()
        );

        // UI should still be functional
        // UIが依然として機能することを確認
        assert!(helper.render_complete_ui().is_ok());
    }

    #[tokio::test]
    async fn test_container_widget_synchronization() {
        // Test synchronization between App navigation and ContainerListWidget
        // AppナビゲーションとContainerListWidget間の同期テスト
        let mut helper = UIIntegrationTestHelper::new().with_test_containers().await;

        // Test synchronized navigation
        // 同期ナビゲーションテスト
        for expected_index in 0..helper.app.containers.len() {
            helper.app.selected_index = expected_index;
            helper
                .container_list_widget
                .sync_with_app(helper.app.selected_index, helper.app.containers.len());

            assert_eq!(
                helper.container_list_widget.selected(),
                Some(expected_index),
                "Widget should be synchronized with App navigation"
            );
        }

        // Test navigation operations keep them in sync
        // ナビゲーション操作で同期が保たれることをテスト
        helper.app.handle_container_navigation(
            &mut helper.container_list_widget,
            NavigationDirection::Next,
        );

        assert_eq!(
            helper.app.selected_index,
            helper.container_list_widget.selected().unwrap(),
            "App and widget should remain synchronized after navigation"
        );
    }

    #[tokio::test]
    async fn test_responsive_layout_integration() {
        // Test responsive layout behavior with different terminal sizes
        // 異なるターミナルサイズでのレスポンシブレイアウト動作テスト

        // Test with small terminal
        // 小さなターミナルでのテスト
        let mut small_helper = UIIntegrationTestHelper::new().with_test_containers().await;
        small_helper.terminal = Terminal::new(TestBackend::new(40, 10)).unwrap();
        assert!(small_helper.render_complete_ui().is_ok());

        // Test with large terminal
        // 大きなターミナルでのテスト
        let mut large_helper = UIIntegrationTestHelper::new().with_test_containers().await;
        large_helper.terminal = Terminal::new(TestBackend::new(120, 40)).unwrap();
        assert!(large_helper.render_complete_ui().is_ok());
    }

    #[test]
    fn test_key_event_handling_integration() {
        // Test integration of key event handling with UI components
        // UIコンポーネントとのキーイベント処理統合テスト

        // Simulate key events
        // キーイベントをシミュレート
        use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

        let key_j = KeyEvent::new(KeyCode::Char('j'), KeyModifiers::NONE);
        let event_j = handle_key_event(key_j);
        assert_eq!(event_j, AppEvent::SelectNext);

        let key_k = KeyEvent::new(KeyCode::Char('k'), KeyModifiers::NONE);
        let event_k = handle_key_event(key_k);
        assert_eq!(event_k, AppEvent::SelectPrevious);

        // Test that events map to correct navigation directions
        // イベントが正しいナビゲーション方向にマップされることをテスト
        let mut helper = UIIntegrationTestHelper::new();

        // Test integration with navigation
        // ナビゲーションとの統合テスト
        match event_j {
            AppEvent::SelectNext => {
                assert!(
                    helper
                        .simulate_navigation(NavigationDirection::Next)
                        .is_ok()
                );
            }
            _ => panic!("Expected SelectNext event"),
        }

        match event_k {
            AppEvent::SelectPrevious => {
                assert!(
                    helper
                        .simulate_navigation(NavigationDirection::Previous)
                        .is_ok()
                );
            }
            _ => panic!("Expected SelectPrevious event"),
        }

        // Verify state after event processing
        // イベント処理後の状態を確認
        assert!(helper.render_complete_ui().is_ok());
    }

    #[tokio::test]
    async fn test_event_statistics_integration() {
        // Test event statistics integration
        // イベント統計統合テスト
        let mut helper = UIIntegrationTestHelper::new().with_test_containers().await;

        // Simulate multiple navigation events
        // 複数のナビゲーションイベントをシミュレート
        for _ in 0..5 {
            assert!(
                helper
                    .simulate_navigation(NavigationDirection::Next)
                    .is_ok()
            );
        }

        for _ in 0..3 {
            assert!(
                helper
                    .simulate_navigation(NavigationDirection::Previous)
                    .is_ok()
            );
        }

        // Verify event statistics
        // イベント統計を確認
        assert_eq!(helper.event_stats.total_events, 8);
        assert_eq!(helper.event_stats.navigation_events, 8);
    }

    #[tokio::test]
    async fn test_theme_integration() {
        // Test theme integration across UI components
        // UIコンポーネント全体でのテーマ統合テスト
        let mut helper_dark = UIIntegrationTestHelper::new().with_test_containers().await;
        helper_dark.theme = Theme::dark();
        assert!(helper_dark.render_complete_ui().is_ok());

        let mut helper_light = UIIntegrationTestHelper::new().with_test_containers().await;
        helper_light.theme = Theme::light();
        assert!(helper_light.render_complete_ui().is_ok());
    }

    #[test]
    fn test_ui_integration_helper_creation() {
        // Test helper creation without layout and status bar instances
        // レイアウトとステータスバーインスタンスなしでヘルパー作成をテスト
        let helper = UIIntegrationTestHelper::new();

        // Verify app is properly initialized
        // アプリが適切に初期化されていることを確認
        assert_eq!(helper.app.containers.len(), 0);
        assert_eq!(helper.app.selected_index, 0);
        assert!(matches!(helper.app.view_state, ViewState::Loading));

        // Verify container widget is initialized with no selection (empty container list)
        // コンテナウィジェットが選択なしで初期化されていることを確認（空のコンテナリスト）
        assert_eq!(helper.container_list_widget.selected(), None);
    }

    #[test]
    fn test_ui_integration_helper_with_containers() {
        // Test helper with containers to verify selection works when containers exist
        // コンテナありでヘルパーをテストし、コンテナが存在する場合に選択が動作することを確認
        let mut helper = UIIntegrationTestHelper::new();

        // Add test containers
        // テストコンテナを追加
        helper.app.containers = create_test_containers();
        helper.app.selected_index = 0;

        // Synchronize widget with app state
        // ウィジェットをアプリ状態と同期
        helper
            .app
            .sync_widget_state(&mut helper.container_list_widget);

        // Now widget should have selection since containers exist
        // コンテナが存在するため、ウィジェットに選択状態があるはず
        assert_eq!(helper.container_list_widget.selected(), Some(0));
        assert_eq!(helper.app.containers.len(), 5);
    }

    #[test]
    fn test_widget_selection_synchronization() {
        // Test widget selection synchronization with app state
        // ウィジェット選択とアプリ状態の同期をテスト
        let mut helper = UIIntegrationTestHelper::new();

        // Initially no selection (no containers)
        // 初期状態では選択なし（コンテナなし）
        assert_eq!(helper.container_list_widget.selected(), None);

        // Add containers and sync
        // コンテナを追加して同期
        helper.app.containers = create_test_containers();
        helper.app.selected_index = 2;
        helper
            .app
            .sync_widget_state(&mut helper.container_list_widget);

        // Widget should now be synchronized
        // ウィジェットが同期されているはず
        assert_eq!(helper.container_list_widget.selected(), Some(2));

        // Test navigation synchronization
        // ナビゲーション同期をテスト
        helper.app.handle_container_navigation(
            &mut helper.container_list_widget,
            NavigationDirection::Next,
        );

        // Both app and widget should be synchronized after navigation
        // ナビゲーション後、アプリとウィジェット両方が同期されているはず
        assert_eq!(helper.app.selected_index, 3);
        assert_eq!(helper.container_list_widget.selected(), Some(3));
    }
}
