// src/ui/widgets/container_list.rs
// Container list widget implementation for TUI
// TUI用コンテナリストウィジェット実装

use ratatui::{
    Frame,
    layout::Rect,
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState},
};

use crate::domain::{Container, ContainerStatus};
use crate::ui::{app::App, styles::Theme};

/// Container list widget for displaying Docker containers
/// Dockerコンテナ表示用コンテナリストウィジェット
///
/// This widget renders a list of Docker containers with status-based color coding,
/// selection highlighting, and keyboard navigation support.
///
/// このウィジェットはDockerコンテナのリストを、ステータスベースの色分け、
/// 選択ハイライト、キーボードナビゲーションサポートと共にレンダリングします。
///
/// # Features
/// - Status-based color coding (Running: Green, Stopped: Red, etc.)
/// - Selection highlighting with background color
/// - Formatted display: "[Name] | [Status] | [Image]"
/// - Empty list messaging
/// - Scrollable list for large container counts
///
/// # Usage
///
/// ```rust,no_run
/// use docka::ui::{ContainerListWidget, App, Theme};
/// use ratatui::{Frame, layout::Rect};
///
/// fn render_container_list(f: &mut Frame, app: &App, area: Rect, theme: &Theme) {
///     ContainerListWidget::render(f, app, area, theme);
/// }
/// ```
pub struct ContainerListWidget {
    /// Internal list state for ratatui List widget
    /// ratatui Listウィジェット用内部リスト状態
    list_state: ListState,
}

impl ContainerListWidget {
    /// Creates a new `ContainerListWidget` instance
    /// `新しいContainerListWidgetインスタンスを作成`
    ///
    /// # Returns
    ///
    /// A new widget instance with default list state
    /// デフォルトリスト状態を持つ新しいウィジェットインスタンス
    #[must_use] pub fn new() -> Self {
        Self {
            list_state: ListState::default(),
        }
    }

    /// Update selection to next item
    /// 次のアイテムに選択を更新
    pub fn select_next(&mut self, container_count: usize) {
        if container_count == 0 {
            return;
        }

        let selected = self.list_state.selected().unwrap_or(0);
        let next = if selected + 1 >= container_count {
            0 // Wrap around to first item
        } else {
            selected + 1
        };
        self.list_state.select(Some(next));
    }

    /// Update selection to previous item
    /// 前のアイテムに選択を更新
    pub fn select_previous(&mut self, container_count: usize) {
        if container_count == 0 {
            return;
        }

        let selected = self.list_state.selected().unwrap_or(0);
        let previous = if selected == 0 {
            container_count - 1 // Wrap around to last item
        } else {
            selected - 1
        };
        self.list_state.select(Some(previous));
    }

    /// Get currently selected index
    /// 現在選択されているインデックスを取得
    #[must_use] pub const fn selected(&self) -> Option<usize> {
        self.list_state.selected()
    }

    /// Set selected index
    /// 選択インデックスを設定
    pub fn set_selected(&mut self, index: Option<usize>) {
        self.list_state.select(index);
    }

    /// Synchronize with app selection state
    /// アプリの選択状態と同期
    pub fn sync_with_app(&mut self, app_selected_index: usize, container_count: usize) {
        if container_count > 0 && app_selected_index < container_count {
            self.list_state.select(Some(app_selected_index));
        } else {
            self.list_state.select(None);
        }
    }

    /// Renders the container list widget to the terminal
    /// コンテナリストウィジェットをターミナルにレンダリング
    pub fn render(
        widget: &mut Self,
        f: &mut Frame,
        app: &App,
        area: Rect,
        theme: &Theme,
    ) {
        // Synchronize widget state with app state
        // ウィジェット状態をアプリ状態と同期
        widget.sync_with_app(app.selected_index, app.containers.len());

        // Create list items from containers
        // コンテナからリストアイテムを作成
        let items: Vec<ListItem> = if app.containers.is_empty() {
            vec![ListItem::new(Line::from(Span::styled(
                "No containers found. Press 'r' to refresh.",
                theme.styles.muted_style(),
            )))]
        } else {
            app.containers
                .iter()
                .enumerate()
                .map(|(index, container)| {
                    let is_selected = Some(index) == widget.list_state.selected();
                    Self::format_container_item(container, is_selected, theme)
                })
                .collect()
        };

        // Create list widget with border and title
        let list = List::new(items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Docker Containers")
                    .border_style(theme.blocks.normal_border_style)
                    .title_style(theme.styles.normal_style()),
            )
            .style(theme.styles.normal_style());

        // Render the stateful widget with maintained state
        f.render_stateful_widget(list, area, &mut widget.list_state);
    }

    /// Formats a single container into a `ListItem` with appropriate styling
    /// `単一コンテナを適切なスタイリングでListItemにフォーマット`
    ///
    /// # Arguments
    ///
    /// * `container` - Container to format
    /// * `is_selected` - Whether this container is currently selected
    /// * `theme` - Theme for styling
    ///
    /// # Returns
    ///
    /// A styled `ListItem` representing the container
    ///
    /// # Format
    ///
    /// The format follows: "[Name] | [Status] | [Image]"
    /// フォーマットは: "[名前] | [ステータス] | [イメージ]" に従います
    fn format_container_item<'a>(
        container: &'a Container,
        is_selected: bool,
        theme: &'a Theme,
    ) -> ListItem<'a> {
        // Determine base style based on container status
        // コンテナステータスに基づいてベーススタイルを決定
        let status_style = match container.status {
            ContainerStatus::Running => theme.styles.success_style(),
            ContainerStatus::Exited { .. } => theme.styles.error_style(),
            ContainerStatus::Paused => theme.styles.loading_style(),
            ContainerStatus::Restarting => theme.styles.loading_style(),
            ContainerStatus::Dead => theme.styles.error_style(),
            ContainerStatus::Created => theme.styles.muted_style(),
            ContainerStatus::Removing => theme.styles.loading_style(),
            ContainerStatus::Stopped => theme.styles.muted_style(),
            ContainerStatus::Starting => theme.styles.loading_style(),
            ContainerStatus::Stopping => theme.styles.loading_style(),
        };

        // Apply selection highlighting if selected
        // 選択されている場合は選択ハイライトを適用
        let final_style = if is_selected {
            theme.styles.selected_style()
        } else {
            status_style
        };

        // Format container information
        // コンテナ情報をフォーマット
        let display_name = container.display_name();
        let status_text = Self::format_status(&container.status);
        let image_name = container
            .image
            .split(':')
            .next()
            .unwrap_or(&container.image);

        // Create formatted line with spans
        // スパンを使用してフォーマット済みラインを作成
        let line = Line::from(vec![
            Span::styled(display_name, final_style),
            Span::styled(" | ", theme.styles.muted_style()),
            Span::styled(status_text, final_style),
            Span::styled(" | ", theme.styles.muted_style()),
            Span::styled(image_name, theme.styles.muted_style()),
        ]);

        ListItem::new(line)
    }

    /// Formats container status for display
    /// 表示用コンテナステータスをフォーマット
    ///
    /// # Arguments
    ///
    /// * `status` - Container status to format
    ///
    /// # Returns
    ///
    /// Human-readable status string
    /// 人間が読める形式のステータス文字列
    fn format_status(status: &ContainerStatus) -> String {
        match status {
            ContainerStatus::Running => "Running".to_string(),
            ContainerStatus::Exited { exit_code } => {
                format!("Exited ({exit_code})")
            }
            ContainerStatus::Paused => "Paused".to_string(),
            ContainerStatus::Restarting => "Restarting".to_string(),
            ContainerStatus::Dead => "Dead".to_string(),
            ContainerStatus::Created => "Created".to_string(),
            ContainerStatus::Removing => "Removing".to_string(),
            ContainerStatus::Stopped => "Stopped".to_string(),
            ContainerStatus::Starting => "Starting".to_string(),
            ContainerStatus::Stopping => "Stopping".to_string(),
        }
    }
}

impl Default for ContainerListWidget {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{ContainerBuilder, ContainerId};
    use chrono::Utc;

    fn create_test_container(name: &str, status: ContainerStatus, image: &str) -> Container {
        ContainerBuilder::new()
            .id(ContainerId::new(format!("{name}_id")).unwrap())
            .name(name.to_string())
            .image(image.to_string())
            .status(status)
            .created_at(Utc::now()) // 必須フィールド追加
            .build()
            .expect("Valid container for testing") // Result型を適切に処理
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
    fn test_format_status_exited_without_code() {
        let status = ContainerStatus::Exited { exit_code: 0 };
        let formatted = ContainerListWidget::format_status(&status);
        assert_eq!(formatted, "Exited");
    }

    #[test]
    fn test_format_status_all_variants() {
        let test_cases = vec![
            (ContainerStatus::Paused, "Paused"),
            (ContainerStatus::Restarting, "Restarting"),
            (ContainerStatus::Dead, "Dead"),
            (ContainerStatus::Created, "Created"),
            (ContainerStatus::Removing, "Removing"),
        ];

        for (status, expected) in test_cases {
            let formatted = ContainerListWidget::format_status(&status);
            assert_eq!(formatted, expected);
        }
    }

    #[test]
    fn test_format_container_item_not_selected() {
        let container =
            create_test_container("test_container", ContainerStatus::Running, "nginx:latest");
        let theme = Theme::dark();

        // Test that the function executes without panic and returns a ListItem
        // 関数がパニックせずに実行され、ListItemを返すことをテスト
        let item = ContainerListWidget::format_container_item(&container, false, &theme);

        // Verify the item was created successfully (no panic occurred)
        // アイテムが正常に作成されたことを確認（パニックが発生しなかった）
        // We can't easily inspect ListItem contents in ratatui 0.29.0, so we verify creation success
        // ratatui 0.29.0ではListItemの内容を簡単に検査できないため、作成成功を確認
        drop(item); // Explicit drop to show the item was created and is valid
    }

    #[test]
    fn test_format_container_item_selected() {
        let container =
            create_test_container("test_container", ContainerStatus::Running, "nginx:latest");
        let theme = Theme::dark();

        // Test that the function executes without panic for selected state
        // 選択状態で関数がパニックせずに実行されることをテスト
        let item = ContainerListWidget::format_container_item(&container, true, &theme);

        // Verify the item was created with selection styling (no panic occurred)
        // 選択スタイリングでアイテムが作成されたことを確認（パニックが発生しなかった）
        drop(item); // Explicit drop to show the item was created and is valid
    }

    #[test]
    fn test_widget_creation() {
        let widget = ContainerListWidget::new();

        // Verify widget was created with default state
        // デフォルト状態でウィジェットが作成されたことを確認
        assert!(widget.list_state.selected().is_none());
    }

    #[test]
    fn test_widget_default() {
        let widget = ContainerListWidget::default();

        // Verify default creation works
        // デフォルト作成が動作することを確認
        assert!(widget.list_state.selected().is_none());
    }
}
