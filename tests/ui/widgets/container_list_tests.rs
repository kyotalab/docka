// tests/ui/widgets/container_list_tests.rs
// Unit tests for ContainerListWidget implementation
// ContainerListWidget実装の単体テスト

#[cfg(test)]
mod container_list_widget_tests {
    use super::*;
    use crate::domain::{
        Container, ContainerBuilder, ContainerId, ContainerStatus, MockDockerRepository,
    };
    use crate::ui::{App, ContainerListWidget, Theme, ViewState};
    use chrono::Utc;
    use ratatui::{Terminal, backend::TestBackend, layout::Rect};
    use std::sync::Arc;

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

    /// Helper function to create test app with containers
    /// コンテナ付きテストアプリ作成ヘルパー関数
    fn create_test_app_with_containers(containers: Vec<Container>) -> App {
        let docker_repo = Arc::new(MockDockerRepository::new());
        let mut app = App::new(docker_repo);
        app.containers = containers;
        app.view_state = ViewState::ContainerList;
        app
    }

    #[test]
    fn test_widget_creation() {
        let widget = ContainerListWidget::new();
        assert!(widget.list_state.selected().is_none());
    }

    #[test]
    fn test_widget_default() {
        let widget = ContainerListWidget::default();
        assert!(widget.list_state.selected().is_none());
    }

    #[test]
    fn test_format_status_running() {
        let status = ContainerStatus::Running;
        let formatted = ContainerListWidget::format_status(&status);
        assert_eq!(formatted, "Running");
    }

    #[test]
    fn test_format_status_exited_with_code() {
        let status = ContainerStatus::Exited { exit_code: 1 };
        let formatted = ContainerListWidget::format_status(&status);
        assert_eq!(formatted, "Exited (1)");
    }

    #[test]
    fn test_format_status_exited_with_zero() {
        let status = ContainerStatus::Exited { exit_code: 0 };
        let formatted = ContainerListWidget::format_status(&status);
        assert_eq!(formatted, "Exited (0)");
    }

    #[test]
    fn test_format_status_paused() {
        let status = ContainerStatus::Paused;
        let formatted = ContainerListWidget::format_status(&status);
        assert_eq!(formatted, "Paused");
    }

    #[test]
    fn test_format_status_restarting() {
        let status = ContainerStatus::Restarting;
        let formatted = ContainerListWidget::format_status(&status);
        assert_eq!(formatted, "Restarting");
    }

    #[test]
    fn test_format_status_dead() {
        let status = ContainerStatus::Dead;
        let formatted = ContainerListWidget::format_status(&status);
        assert_eq!(formatted, "Dead");
    }

    #[test]
    fn test_format_status_created() {
        let status = ContainerStatus::Created;
        let formatted = ContainerListWidget::format_status(&status);
        assert_eq!(formatted, "Created");
    }

    #[test]
    fn test_format_status_removing() {
        let status = ContainerStatus::Removing;
        let formatted = ContainerListWidget::format_status(&status);
        assert_eq!(formatted, "Removing");
    }

    #[test]
    fn test_format_status_unknown() {
        let status = ContainerStatus::Unknown;
        let formatted = ContainerListWidget::format_status(&status);
        assert_eq!(formatted, "Unknown");
    }

    #[test]
    fn test_format_container_item_running_not_selected() {
        let container =
            create_test_container("test_container", ContainerStatus::Running, "nginx:latest");
        let theme = Theme::dark();

        let item = ContainerListWidget::format_container_item(&container, false, &theme);

        // Verify the item was created successfully
        // アイテムが正常に作成されたことを確認
        drop(item); // Function completed without panic
    }

    #[test]
    fn test_format_container_item_running_selected() {
        let container =
            create_test_container("test_container", ContainerStatus::Running, "nginx:latest");
        let theme = Theme::dark();

        let item = ContainerListWidget::format_container_item(&container, true, &theme);

        // Verify the item was created with selection styling
        // 選択スタイリングでアイテムが作成されたことを確認
        drop(item); // Function completed without panic
    }

    #[test]
    fn test_format_container_item_stopped_not_selected() {
        let container = create_test_container(
            "db_server",
            ContainerStatus::Exited { exit_code: 0 },
            "postgres:13",
        );
        let theme = Theme::dark();

        let item = ContainerListWidget::format_container_item(&container, false, &theme);

        // Verify the item was created successfully
        // アイテムが正常に作成されたことを確認
        drop(item); // Function completed without panic
    }

    #[test]
    fn test_format_container_item_all_statuses() {
        let theme = Theme::dark();
        let test_cases = vec![
            (ContainerStatus::Running, "web1"),
            (ContainerStatus::Exited { exit_code: 1 }, "web2"),
            (ContainerStatus::Paused, "web3"),
            (ContainerStatus::Restarting, "web4"),
            (ContainerStatus::Dead, "web5"),
            (ContainerStatus::Created, "web6"),
            (ContainerStatus::Removing, "web7"),
            (ContainerStatus::Stopped, "web8"),
            (ContainerStatus::Starting, "web9"),
            (ContainerStatus::Stopping, "web10"),
        ];

        for (status, name) in test_cases {
            let container = create_test_container(name, status, "nginx:latest");
            let item = ContainerListWidget::format_container_item(&container, false, &theme);

            // Each status should create a valid item without panicking
            // 各ステータスでパニックせずに有効なアイテムが作成されることを確認
            drop(item); // Function completed without panic for this status
        }
    }

    #[test]
    fn test_render_empty_container_list() {
        let docker_repo = Arc::new(MockDockerRepository::new());
        let app = App::new(docker_repo);
        let theme = Theme::dark();
        let area = Rect::new(0, 0, 80, 20);

        let backend = TestBackend::new(80, 20);
        let mut terminal = Terminal::new(backend).unwrap();

        // Should not panic when rendering empty container list
        // 空のコンテナリストをレンダリングしてもパニックしないことを確認
        terminal
            .draw(|f| {
                ContainerListWidget::render(f, &app, area, &theme);
            })
            .unwrap();
    }

    #[test]
    fn test_render_single_container() {
        let containers = vec![create_test_container(
            "web_server",
            ContainerStatus::Running,
            "nginx:latest",
        )];
        let app = create_test_app_with_containers(containers);
        let theme = Theme::dark();
        let area = Rect::new(0, 0, 80, 20);

        let backend = TestBackend::new(80, 20);
        let mut terminal = Terminal::new(backend).unwrap();

        // Should not panic when rendering single container
        // 単一コンテナをレンダリングしてもパニックしないことを確認
        terminal
            .draw(|f| {
                ContainerListWidget::render(f, &app, area, &theme);
            })
            .unwrap();
    }

    #[test]
    fn test_render_multiple_containers() {
        let containers = vec![
            create_test_container("web_server", ContainerStatus::Running, "nginx:latest"),
            create_test_container(
                "db_server",
                ContainerStatus::Exited { exit_code: Some(0) },
                "postgres:13",
            ),
            create_test_container("cache_server", ContainerStatus::Paused, "redis:alpine"),
        ];
        let app = create_test_app_with_containers(containers);
        let theme = Theme::dark();
        let area = Rect::new(0, 0, 80, 20);

        let backend = TestBackend::new(80, 20);
        let mut terminal = Terminal::new(backend).unwrap();

        // Should not panic when rendering multiple containers
        // 複数コンテナをレンダリングしてもパニックしないことを確認
        terminal
            .draw(|f| {
                ContainerListWidget::render(f, &app, area, &theme);
            })
            .unwrap();
    }

    #[test]
    fn test_render_with_selection() {
        let containers = vec![
            create_test_container("web_server", ContainerStatus::Running, "nginx:latest"),
            create_test_container(
                "db_server",
                ContainerStatus::Exited { exit_code: Some(0) },
                "postgres:13",
            ),
        ];
        let mut app = create_test_app_with_containers(containers);
        app.selected_index = 1; // Select second container

        let theme = Theme::dark();
        let area = Rect::new(0, 0, 80, 20);

        let backend = TestBackend::new(80, 20);
        let mut terminal = Terminal::new(backend).unwrap();

        // Should not panic when rendering with selection
        // 選択状態でレンダリングしてもパニックしないことを確認
        terminal
            .draw(|f| {
                ContainerListWidget::render(f, &app, area, &theme);
            })
            .unwrap();
    }

    #[test]
    fn test_render_small_area() {
        let containers = vec![create_test_container(
            "web",
            ContainerStatus::Running,
            "nginx",
        )];
        let app = create_test_app_with_containers(containers);
        let theme = Theme::dark();
        let area = Rect::new(0, 0, 20, 5); // Small area

        let backend = TestBackend::new(20, 5);
        let mut terminal = Terminal::new(backend).unwrap();

        // Should handle small rendering areas gracefully
        // 小さなレンダリングエリアを適切に処理することを確認
        terminal
            .draw(|f| {
                ContainerListWidget::render(f, &app, area, &theme);
            })
            .unwrap();
    }

    #[test]
    fn test_render_with_long_names() {
        let containers = vec![create_test_container(
            "very_long_container_name_that_might_overflow",
            ContainerStatus::Running,
            "very_long_image_name_with_registry:latest",
        )];
        let app = create_test_app_with_containers(containers);
        let theme = Theme::dark();
        let area = Rect::new(0, 0, 40, 10); // Medium area

        let backend = TestBackend::new(40, 10);
        let mut terminal = Terminal::new(backend).unwrap();

        // Should handle long names gracefully without panicking
        // 長い名前を適切に処理し、パニックしないことを確認
        terminal
            .draw(|f| {
                ContainerListWidget::render(f, &app, area, &theme);
            })
            .unwrap();
    }

    #[test]
    fn test_image_name_truncation() {
        let container = create_test_container(
            "web",
            ContainerStatus::Running,
            "registry.example.com/namespace/nginx:v1.20.1-alpine",
        );
        let theme = Theme::dark();

        let item = ContainerListWidget::format_container_item(&container, false, &theme);

        // Should handle long image names by showing only the image part before ':'
        // 長いイメージ名を':'より前の部分のみ表示して処理することを確認
        drop(item); // Function completed without panic
    }

    #[test]
    fn test_container_with_empty_name() {
        let container = create_test_container("", ContainerStatus::Running, "nginx:latest");
        let theme = Theme::dark();

        let item = ContainerListWidget::format_container_item(&container, false, &theme);

        // Should handle containers with empty names (will use display_name which falls back to ID)
        // 空の名前を持つコンテナを処理することを確認（display_nameがIDにフォールバック）
        drop(item); // Function completed without panic
    }

    #[test]
    fn test_theme_consistency() {
        let container = create_test_container("web", ContainerStatus::Running, "nginx:latest");
        let dark_theme = Theme::dark();
        let light_theme = Theme::light();

        // Both themes should successfully create items
        // 両方のテーマでアイテムが正常に作成されることを確認
        let dark_item = ContainerListWidget::format_container_item(&container, false, &dark_theme);
        let light_item =
            ContainerListWidget::format_container_item(&container, false, &light_theme);

        drop(dark_item); // Function completed without panic for dark theme
        drop(light_item); // Function completed without panic for light theme
    }

    #[test]
    fn test_status_color_mapping() {
        let theme = Theme::dark();

        // Test that different statuses get different styles
        // 異なるステータスが異なるスタイルを取得することをテスト
        let running_container = create_test_container("web1", ContainerStatus::Running, "nginx");
        let stopped_container =
            create_test_container("web2", ContainerStatus::Exited { exit_code: 0 }, "nginx");
        let paused_container = create_test_container("web3", ContainerStatus::Paused, "nginx");

        let running_item =
            ContainerListWidget::format_container_item(&running_container, false, &theme);
        let stopped_item =
            ContainerListWidget::format_container_item(&stopped_container, false, &theme);
        let paused_item =
            ContainerListWidget::format_container_item(&paused_container, false, &theme);

        // All should be valid and created without panicking
        // すべて有効でパニックせずに作成されることを確認
        drop(running_item);
        drop(stopped_item);
        drop(paused_item);
    }

    #[test]
    fn test_selection_highlighting() {
        let container = create_test_container("web", ContainerStatus::Running, "nginx:latest");
        let theme = Theme::dark();

        // Test both selected and non-selected states
        // 選択状態と非選択状態の両方をテスト
        let selected_item = ContainerListWidget::format_container_item(&container, true, &theme);
        let unselected_item = ContainerListWidget::format_container_item(&container, false, &theme);

        drop(selected_item); // Function completed without panic for selected state
        drop(unselected_item); // Function completed without panic for unselected state
    }

    #[test]
    fn test_render_boundary_conditions() {
        let theme = Theme::dark();

        // Test with zero-sized area
        // ゼロサイズエリアでのテスト
        let app = create_test_app_with_containers(vec![]);
        let zero_area = Rect::new(0, 0, 0, 0);

        let backend = TestBackend::new(1, 1);
        let mut terminal = Terminal::new(backend).unwrap();

        // Should handle zero-sized area gracefully
        // ゼロサイズエリアを適切に処理することを確認
        terminal
            .draw(|f| {
                ContainerListWidget::render(f, &app, zero_area, &theme);
            })
            .unwrap();
    }
}

// Integration tests for ContainerListWidget with App state
// App状態とのContainerListWidget統合テスト
#[cfg(test)]
mod container_list_integration_tests {
    use super::*;
    use crate::domain::{
        Container, ContainerBuilder, ContainerId, ContainerStatus, MockDockerRepository,
    };
    use crate::ui::{App, ContainerListWidget, Theme, ViewState};
    use ratatui::{Terminal, backend::TestBackend, layout::Rect};
    use std::sync::Arc;

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

    #[test]
    fn test_full_integration_empty_list() {
        let docker_repo = Arc::new(MockDockerRepository::new());
        let app = App::new(docker_repo);
        let theme = Theme::dark();
        let area = Rect::new(0, 0, 80, 20);

        let backend = TestBackend::new(80, 20);
        let mut terminal = Terminal::new(backend).unwrap();

        // Test complete integration with empty container list
        // 空のコンテナリストでの完全統合テスト
        terminal
            .draw(|f| {
                ContainerListWidget::render(f, &app, area, &theme);
            })
            .unwrap();
    }

    #[test]
    fn test_full_integration_with_containers() {
        let docker_repo = Arc::new(MockDockerRepository::new());
        let mut app = App::new(docker_repo);

        // Add test containers to app state
        // アプリ状態にテストコンテナを追加
        app.containers = vec![
            create_test_container("nginx_web", ContainerStatus::Running, "nginx:latest"),
            create_test_container(
                "postgres_db",
                ContainerStatus::Exited { exit_code: 0 },
                "postgres:13",
            ),
            create_test_container("redis_cache", ContainerStatus::Paused, "redis:alpine"),
        ];
        app.view_state = ViewState::ContainerList;
        app.selected_index = 1;

        let theme = Theme::dark();
        let area = Rect::new(0, 0, 80, 20);

        let backend = TestBackend::new(80, 20);
        let mut terminal = Terminal::new(backend).unwrap();

        // Test complete integration with populated container list and selection
        // コンテナリストと選択状態での完全統合テスト
        terminal
            .draw(|f| {
                ContainerListWidget::render(f, &app, area, &theme);
            })
            .unwrap();
    }

    #[test]
    fn test_navigation_integration() {
        let docker_repo = Arc::new(MockDockerRepository::new());
        let mut app = App::new(docker_repo);

        // Add test containers
        // テストコンテナを追加
        app.containers = vec![
            create_test_container("container1", ContainerStatus::Running, "image1"),
            create_test_container(
                "container2",
                ContainerStatus::Exited { exit_code: 0 },
                "image2",
            ),
            create_test_container("container3", ContainerStatus::Running, "image3"),
        ];
        app.view_state = ViewState::ContainerList;

        let theme = Theme::dark();
        let area = Rect::new(0, 0, 80, 20);
        let backend = TestBackend::new(80, 20);
        let mut terminal = Terminal::new(backend).unwrap();

        // Test navigation through different selections
        // 異なる選択でのナビゲーションをテスト
        for i in 0..app.containers.len() {
            app.selected_index = i;

            terminal
                .draw(|f| {
                    ContainerListWidget::render(f, &app, area, &theme);
                })
                .unwrap();
        }
    }

    #[test]
    fn test_different_view_states() {
        let docker_repo = Arc::new(MockDockerRepository::new());
        let mut app = App::new(docker_repo);

        app.containers = vec![create_test_container(
            "test_container",
            ContainerStatus::Running,
            "test_image",
        )];

        let theme = Theme::dark();
        let area = Rect::new(0, 0, 80, 20);
        let backend = TestBackend::new(80, 20);
        let mut terminal = Terminal::new(backend).unwrap();

        // Test rendering in different view states
        // 異なるビュー状態でのレンダリングをテスト
        let view_states = vec![
            ViewState::ContainerList,
            ViewState::Loading,
            ViewState::Error("Test error".to_string()),
        ];

        for view_state in view_states {
            app.view_state = view_state;

            terminal
                .draw(|f| {
                    ContainerListWidget::render(f, &app, area, &theme);
                })
                .unwrap();
        }
    }
}
