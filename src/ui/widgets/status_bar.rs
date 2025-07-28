//! Status Bar Widget Implementation for Docka TUI
//!
//! ステータスバーウィジェット実装 - Docka TUI用
//!
//! This module provides a status bar widget that displays current application state,
//! container information, and contextual help messages.
//!
//! このモジュールは現在のアプリケーション状態、コンテナ情報、
//! コンテキストヘルプメッセージを表示するステータスバーウィジェットを提供します。

use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};

use crate::ui::app::{App, ViewState};

/// Status bar widget for displaying application state and information
/// アプリケーション状態と情報を表示するステータスバーウィジェット
pub struct StatusBar;

impl StatusBar {
    /// Render status bar widget based on current application state
    /// 現在のアプリケーション状態に基づいてステータスバーウィジェットを描画
    ///
    /// # Arguments / 引数
    /// * `f` - Frame for rendering / 描画用フレーム
    /// * `app` - Current application state / 現在のアプリケーション状態
    /// * `area` - Area to render within / 描画するエリア
    ///
    /// # Display Logic / 表示ロジック
    /// - Loading: Yellow background with loading message / 黄色背景でローディングメッセージ
    /// - Error: Red background with error message / 赤色背景でエラーメッセージ
    /// - `ContainerList`: Green accent with container count and selection / 緑色アクセントでコンテナ数と選択状態
    pub fn render(f: &mut Frame, app: &App, area: Rect) {
        let (content, style) = Self::build_status_content(app);

        let status_block = Block::default().borders(Borders::ALL).style(style);

        let status_paragraph = Paragraph::new(content).block(status_block);

        f.render_widget(status_paragraph, area);
    }

    /// Build status content and style based on application state
    /// アプリケーション状態に基づいてステータスコンテンツとスタイルを構築
    ///
    /// # Arguments / 引数
    /// * `app` - Current application state / 現在のアプリケーション状態
    ///
    /// # Returns / 戻り値
    /// * `(Line, Style)` - Content line and border style / コンテンツラインとボーダースタイル
    fn build_status_content(app: &App) -> (Line<'static>, Style) {
        match &app.view_state {
            ViewState::Loading => {
                let content = Line::from(vec![Span::styled(
                    "⏳ Loading containers...",
                    Style::default()
                        .fg(Color::Black)
                        .bg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                )]);
                let style = Style::default().fg(Color::Yellow);
                (content, style)
            }

            ViewState::Error(error_msg) => {
                let content = Line::from(vec![
                    Span::styled(
                        "❌ Error: ",
                        Style::default()
                            .fg(Color::White)
                            .bg(Color::Red)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(
                        error_msg.clone(),
                        Style::default().fg(Color::White).bg(Color::Red),
                    ),
                ]);
                let style = Style::default().fg(Color::Red);
                (content, style)
            }

            ViewState::ContainerList => {
                let container_count = app.containers.len();
                let selected_position = if container_count > 0 {
                    app.selected_index + 1
                } else {
                    0
                };

                let content = Line::from(vec![
                    Span::styled(
                        "📦 Containers: ",
                        Style::default()
                            .fg(Color::Green)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(
                        container_count.to_string(),
                        Style::default()
                            .fg(Color::White)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(" | Selected: ", Style::default().fg(Color::Gray)),
                    Span::styled(
                        format!("{selected_position}/{container_count}"),
                        Style::default()
                            .fg(Color::Cyan)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(
                        " | j/k: navigate, q: quit, r: refresh",
                        Style::default().fg(Color::DarkGray),
                    ),
                ]);
                let style = Style::default().fg(Color::Green);
                (content, style)
            }
        }
    }

    /// Get recommended height for status bar
    /// ステータスバーの推奨高さを取得
    ///
    /// # Returns / 戻り値
    /// * `u16` - Recommended height in rows / 推奨高さ（行数）
    #[must_use] pub const fn recommended_height() -> u16 {
        3 // Border (1) + Content (1) + Border (1) / ボーダー(1) + コンテンツ(1) + ボーダー(1)
    }

    /// Check if status bar can be displayed in given area
    /// 指定されたエリアでステータスバーが表示できるかチェック
    ///
    /// # Arguments / 引数
    /// * `area` - Area to check / チェックするエリア
    ///
    /// # Returns / 戻り値
    /// * `bool` - True if status bar can be displayed / ステータスバーが表示できる場合true
    #[must_use] pub const fn can_display(area: Rect) -> bool {
        area.height >= Self::recommended_height() && area.width >= 20
    }

    /// Create a minimal status bar for very small terminals
    /// 非常に小さなターミナル用の最小ステータスバーを作成
    ///
    /// # Arguments / 引数
    /// * `f` - Frame for rendering / 描画用フレーム
    /// * `app` - Current application state / 現在のアプリケーション状態
    /// * `area` - Area to render within / 描画するエリア
    pub fn render_minimal(f: &mut Frame, app: &App, area: Rect) {
        let content = match &app.view_state {
            ViewState::Loading => Line::from(Span::styled(
                "Loading...",
                Style::default().fg(Color::Yellow),
            )),
            ViewState::Error(_) => Line::from(Span::styled(
                "Error!",
                Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
            )),
            ViewState::ContainerList => {
                let count = app.containers.len();
                let selected = if count > 0 { app.selected_index + 1 } else { 0 };
                Line::from(vec![
                    Span::styled(
                        format!("{selected}/{count}"),
                        Style::default().fg(Color::Green),
                    ),
                    Span::styled(" | q:quit", Style::default().fg(Color::DarkGray)),
                ])
            }
        };

        let minimal_paragraph = Paragraph::new(content);
        f.render_widget(minimal_paragraph, area);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{Container, ContainerBuilder, ContainerId, ContainerStatus};
    use crate::ui::app::{App, ViewState};
    use ratatui::layout::Rect;
    use std::sync::Arc;

    /// Create test app with mock data
    /// テストデータでテストアプリを作成
    fn create_test_app() -> App {
        // Create a mock Docker repository for testing
        // テスト用のモックDockerリポジトリを作成
        let docker_repo = Arc::new(crate::domain::MockDockerRepository::new());
        App::new(docker_repo)
    }

    /// Create test container for testing
    /// テスト用のテストコンテナを作成
    fn create_test_container(name: &str, status: ContainerStatus) -> Container {
        ContainerBuilder::new()
            .id(ContainerId::new(format!("test-{name}")).expect("Valid container ID"))
            .name(name.to_string())
            .image("test-image:latest".to_string())
            .status(status)
            .build()
            .expect("Valid container")
    }

    #[test]
    fn test_build_status_content_loading() {
        // Test loading state status content
        // ローディング状態のステータスコンテンツをテスト
        let mut app = create_test_app();
        app.view_state = ViewState::Loading;

        let (content, style) = StatusBar::build_status_content(&app);

        // Check content contains loading message
        // コンテンツにローディングメッセージが含まれることを確認
        let content_text = content
            .spans
            .iter()
            .map(|span| span.content.as_ref())
            .collect::<String>();
        assert!(content_text.contains("Loading containers"));

        // Check style is yellow
        // スタイルが黄色であることを確認
        assert_eq!(style.fg, Some(Color::Yellow));
    }

    #[test]
    fn test_build_status_content_error() {
        // Test error state status content
        // エラー状態のステータスコンテンツをテスト
        let mut app = create_test_app();
        let error_message = "Docker connection failed".to_string();
        app.view_state = ViewState::Error(error_message.clone());

        let (content, style) = StatusBar::build_status_content(&app);

        // Check content contains error message
        // コンテンツにエラーメッセージが含まれることを確認
        let content_text = content
            .spans
            .iter()
            .map(|span| span.content.as_ref())
            .collect::<String>();
        assert!(content_text.contains("Error"));
        assert!(content_text.contains(&error_message));

        // Check style is red
        // スタイルが赤色であることを確認
        assert_eq!(style.fg, Some(Color::Red));
    }

    #[test]
    fn test_build_status_content_container_list_empty() {
        // Test container list state with no containers
        // コンテナなしでのコンテナリスト状態をテスト
        let mut app = create_test_app();
        app.view_state = ViewState::ContainerList;
        app.containers = vec![];

        let (content, style) = StatusBar::build_status_content(&app);

        // Check content shows zero containers
        // コンテンツがゼロコンテナを表示することを確認
        let content_text = content
            .spans
            .iter()
            .map(|span| span.content.as_ref())
            .collect::<String>();
        assert!(content_text.contains("Containers: 0"));
        assert!(content_text.contains("Selected: 0/0"));

        // Check style is green
        // スタイルが緑色であることを確認
        assert_eq!(style.fg, Some(Color::Green));
    }

    #[test]
    fn test_build_status_content_container_list_with_containers() {
        // Test container list state with containers
        // コンテナありでのコンテナリスト状態をテスト
        let mut app = create_test_app();
        app.view_state = ViewState::ContainerList;
        app.containers = vec![
            create_test_container("container1", ContainerStatus::Running),
            create_test_container("container2", ContainerStatus::Stopped),
            create_test_container("container3", ContainerStatus::Running),
        ];
        app.selected_index = 1; // Select second container / 2番目のコンテナを選択

        let (content, style) = StatusBar::build_status_content(&app);

        // Check content shows correct container count and selection
        // コンテンツが正しいコンテナ数と選択状態を表示することを確認
        let content_text = content
            .spans
            .iter()
            .map(|span| span.content.as_ref())
            .collect::<String>();
        assert!(content_text.contains("Containers: 3"));
        assert!(content_text.contains("Selected: 2/3"));
        assert!(content_text.contains("j/k: navigate"));

        // Check style is green
        // スタイルが緑色であることを確認
        assert_eq!(style.fg, Some(Color::Green));
    }

    #[test]
    fn test_recommended_height() {
        // Test recommended height value
        // 推奨高さの値をテスト
        assert_eq!(StatusBar::recommended_height(), 3);
    }

    #[test]
    fn test_can_display() {
        // Test display capability check
        // 表示可能性チェックをテスト

        // Large enough area
        // 十分に大きなエリア
        assert!(StatusBar::can_display(Rect::new(0, 0, 80, 24)));
        assert!(StatusBar::can_display(Rect::new(0, 0, 40, 10)));
        assert!(StatusBar::can_display(Rect::new(0, 0, 20, 3)));

        // Too small height
        // 高さが小さすぎる
        assert!(!StatusBar::can_display(Rect::new(0, 0, 80, 2)));
        assert!(!StatusBar::can_display(Rect::new(0, 0, 80, 1)));

        // Too small width
        // 幅が小さすぎる
        assert!(!StatusBar::can_display(Rect::new(0, 0, 19, 5)));
        assert!(!StatusBar::can_display(Rect::new(0, 0, 10, 5)));
    }

    #[test]
    fn test_status_content_navigation_bounds() {
        // Test status content with edge cases for navigation
        // ナビゲーションのエッジケースでステータスコンテンツをテスト
        let mut app = create_test_app();
        app.view_state = ViewState::ContainerList;
        app.containers = vec![create_test_container(
            "only-container",
            ContainerStatus::Running,
        )];

        // Test first and only container selection
        // 最初で唯一のコンテナ選択をテスト
        app.selected_index = 0;
        let (content, _) = StatusBar::build_status_content(&app);
        let content_text = content
            .spans
            .iter()
            .map(|span| span.content.as_ref())
            .collect::<String>();
        assert!(content_text.contains("Selected: 1/1"));
    }
}
