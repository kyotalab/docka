// src/main.rs
// Main entry point and TUI event loop for docka application
// dockaアプリケーションのメインエントリーポイントとTUIイベントループ

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{
    Terminal,
    backend::{Backend, CrosstermBackend},
};
use std::{
    io::{self, Stdout},
    sync::Arc,
    time::{Duration, Instant},
};
use tokio::time::timeout;

// Import docka modules
use docka::{
    error::{DockaError, DockaResult},
    infrastructure::BollardDockerRepository,
    ui::{
        app::{App, NavigationDirection},
        events::{AppEvent, EventStats, handle_key_event, process_app_event},
        layouts::SimpleLayout,
        styles::Theme,
        validate_key_input,
        widgets::{ContainerListWidget, StatusBar}, // ContainerListWidget, StatusBar を追加
    },
};

/// Application configuration constants
/// アプリケーション設定定数
const APP_NAME: &str = "docka";
const VERSION: &str = env!("CARGO_PKG_VERSION");
const EVENT_POLL_INTERVAL: Duration = Duration::from_millis(100);
const STARTUP_TIMEOUT: Duration = Duration::from_secs(3);

/// Main application entry point
/// メインアプリケーションエントリーポイント
///
/// This function initializes the Docker repository, sets up the terminal,
/// creates the application state, and runs the main event loop.
///
/// この関数はDockerリポジトリを初期化し、ターミナルを設定し、
/// アプリケーション状態を作成し、メインイベントループを実行します。
///
/// # Returns
/// * `Ok(())` - Application exited successfully
/// * `Err(Box<dyn std::error::Error>)` - Application failed with error
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing for debugging in development
    // 開発時のデバッグ用トレース初期化
    #[cfg(debug_assertions)]
    init_tracing()?;

    // Display startup banner
    // 起動バナーを表示
    println!("{} v{} - TUI Docker Management Tool", APP_NAME, VERSION);
    println!("Initializing Docker connection...");

    // Initialize Docker repository with timeout
    // タイムアウト付きでDockerリポジトリを初期化
    let docker_repo = timeout(STARTUP_TIMEOUT, BollardDockerRepository::new())
        .await
        .map_err(|_| DockaError::Internal {
            message: "Docker initialization timed out after 3 seconds".to_string(),
        })?
        .map_err(|_| DockaError::DockerDaemonNotRunning)?;

    let docker_repo = Arc::new(docker_repo);

    // Test Docker connection
    // Docker接続をテスト
    println!("Testing Docker connection...");
    test_docker_connection(&docker_repo).await?;
    println!("Docker connection successful!");

    // Setup terminal
    // ターミナル設定
    let mut terminal = setup_terminal()?;

    // Create application state
    // アプリケーション状態を作成
    let mut app = App::new(docker_repo);

    // Initial container load
    // 初期コンテナロード
    println!("Loading initial container data...");
    if let Err(e) = app.refresh_containers().await {
        eprintln!("Warning: Failed to load containers: {}", e);
        // Continue anyway - user can manually refresh
        // とりあえず継続 - ユーザーが手動でリフレッシュ可能
    }

    // Run the application
    // アプリケーション実行
    let result = run_app(&mut terminal, &mut app).await;

    // Cleanup terminal
    // ターミナルクリーンアップ
    cleanup_terminal(&mut terminal)?;

    // Handle application result
    // アプリケーション結果を処理
    match result {
        Ok(stats) => {
            println!("Application exited successfully.");
            println!("Event Statistics:");
            println!("  Total events: {}", stats.total_events);
            println!("  Navigation events: {}", stats.navigation_events);
            println!("  Action events: {}", stats.action_events);
            println!("  Error rate: {:.2}%", stats.error_rate());
            Ok(())
        }
        Err(e) => {
            eprintln!("Application error: {}", e);
            Err(e.into())
        }
    }
}

/// Setup terminal for TUI mode
/// TUIモード用ターミナル設定
///
/// This function enables raw mode, enters alternate screen, and enables mouse capture
/// for the terminal interface.
///
/// この関数は生モードを有効にし、代替画面に入り、
/// ターミナルインターフェース用にマウスキャプチャを有効にします。
///
/// # Returns
/// * `Ok(Terminal)` - Successfully configured terminal
/// * `Err(DockaError)` - Terminal setup failed
fn setup_terminal() -> DockaResult<Terminal<CrosstermBackend<Stdout>>> {
    // Enable raw mode for direct key input
    // 直接キー入力用に生モードを有効化
    enable_raw_mode().map_err(DockaError::Io)?;

    let mut stdout = io::stdout();

    // Enter alternate screen and enable mouse capture
    // 代替画面に入りマウスキャプチャを有効化
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture).map_err(DockaError::Io)?;

    // Create terminal backend
    // ターミナルバックエンドを作成
    let backend = CrosstermBackend::new(stdout);
    let terminal = Terminal::new(backend).map_err(|e| DockaError::UiRendering {
        message: format!("Failed to create terminal: {}", e),
    })?;

    Ok(terminal)
}

/// Cleanup terminal and restore normal mode
/// ターミナルクリーンアップと通常モード復元
///
/// This function restores the terminal to its normal state by disabling raw mode,
/// leaving alternate screen, and disabling mouse capture.
///
/// この関数は生モードを無効にし、代替画面を離れ、マウスキャプチャを無効にして
/// ターミナルを通常状態に復元します。
///
/// # Arguments
/// * `terminal` - Terminal instance to cleanup
///
/// # Returns
/// * `Ok(())` - Terminal cleaned up successfully
/// * `Err(DockaError)` - Cleanup failed
fn cleanup_terminal<B: Backend>(terminal: &mut Terminal<B>) -> DockaResult<()> {
    // Disable raw mode
    // 生モードを無効化
    disable_raw_mode().map_err(DockaError::Io)?;

    // Leave alternate screen and disable mouse capture using stdout directly
    // stdoutを直接使用して代替画面を離れマウスキャプチャを無効化
    execute!(io::stdout(), LeaveAlternateScreen, DisableMouseCapture).map_err(DockaError::Io)?;

    // Show cursor
    // カーソルを表示
    terminal
        .show_cursor()
        .map_err(|e| DockaError::UiRendering {
            message: format!("Failed to show cursor: {}", e),
        })?;

    Ok(())
}

/// Test Docker connection with basic API call
/// 基本API呼び出しでDocker接続をテスト
///
/// # Arguments
/// * `docker_repo` - Docker repository to test
///
/// # Returns
/// * `Ok(())` - Connection test successful
/// * `Err(DockaError)` - Connection test failed
async fn test_docker_connection<T>(docker_repo: &Arc<T>) -> DockaResult<()>
where
    T: docka::domain::DockerRepository + ?Sized,
{
    // Try to list containers as a connection test
    // 接続テストとしてコンテナリストを試行
    docker_repo.list_containers().await.map(|containers| {
        println!("Found {} containers", containers.len());
    })
}

/// Main application event loop
/// メインアプリケーションイベントループ
///
/// This function runs the main TUI event loop, handling keyboard input,
/// updating application state, and rendering the UI.
///
/// この関数はメインTUIイベントループを実行し、キーボード入力を処理し、
/// アプリケーション状態を更新し、UIをレンダリングします。
///
/// # Arguments
/// * `terminal` - Terminal instance for rendering
/// * `app` - Application state to manage
///
/// # Returns
/// * `Ok(EventStats)` - Application exited successfully with statistics
/// * `Err(DockaError)` - Event loop failed
async fn run_app<B: Backend>(terminal: &mut Terminal<B>, app: &mut App) -> DockaResult<EventStats> {
    let mut event_stats = EventStats::new();
    let mut last_render = Instant::now();
    const TARGET_FPS: Duration = Duration::from_millis(16); // ~60 FPS

    // === Widget統合のための初期化 ===
    let mut container_widget = ContainerListWidget::new();
    let theme = Theme::dark();

    // 初期状態同期
    app.sync_widget_state(&mut container_widget);

    // Main event loop
    // メインイベントループ
    while app.is_running() {
        // Handle events with timeout
        // タイムアウト付きイベント処理
        if event::poll(EVENT_POLL_INTERVAL).map_err(DockaError::Io)? {
            if let Event::Key(key_event) = event::read().map_err(DockaError::Io)? {
                // Validate and process key input
                // キー入力を検証して処理
                if validate_key_input(key_event) {
                    let app_event = handle_key_event(key_event);

                    // 統合されたイベント処理
                    let event_result = match app_event {
                        AppEvent::SelectNext => {
                            app.handle_container_navigation(
                                &mut container_widget,
                                NavigationDirection::Next,
                            );
                            Ok(())
                        }
                        AppEvent::SelectPrevious => {
                            app.handle_container_navigation(
                                &mut container_widget,
                                NavigationDirection::Previous,
                            );
                            Ok(())
                        }
                        AppEvent::Refresh => {
                            match app.refresh_containers().await {
                                Ok(()) => {
                                    // 更新成功後にウィジェット状態を同期
                                    app.sync_widget_state(&mut container_widget);
                                    Ok(())
                                }
                                Err(e) => {
                                    // エラー状態は自動的にrefresh_containers内で設定される
                                    Err(e)
                                }
                            }
                        }
                        // 他のイベントは既存のprocess_app_event関数を使用
                        _ => process_app_event(app, app_event.clone()).await,
                    };

                    // Record event statistics
                    // イベント統計を記録
                    event_stats.record_event(&app_event, &event_result);

                    // Handle processing errors
                    // 処理エラーを処理
                    if let Err(ref error) = event_result {
                        // Log error but continue running
                        // エラーをログするが実行を継続
                        #[cfg(debug_assertions)]
                        eprintln!("Event processing error: {}", error);
                    }
                }
            }
            // Note: Other events (resize, mouse, etc.) are implicitly ignored
            // 注意: その他のイベント（リサイズ、マウス等）は暗黙的に無視される
        }

        // === 統合レンダリング（修正箇所） ===
        // Render UI with frame rate limiting
        // フレームレート制限付きでUIをレンダリング
        let now = Instant::now();
        if now.duration_since(last_render) >= TARGET_FPS || app.needs_redraw() {
            // 統合されたrender_ui関数を使用
            render_ui(terminal, app, &mut container_widget, &theme)?;
            last_render = now;
        }

        // Small yield to prevent busy waiting
        // ビジー待機を防ぐため小さなyield
        tokio::task::yield_now().await;
    }

    Ok(event_stats)
}

// /// Render UI with full widget integration
// /// 完全なウィジェット統合でUIをレンダリング
// fn render_ui_integrated(
//     f: &mut ratatui::Frame,
//     app: &App,
//     container_widget: &mut ContainerListWidget,
//     theme: &Theme,
// ) {
//     // レスポンシブレイアウトを計算
//     let layout = SimpleLayout::calculate_responsive(f.area());

//     // メインエリア: ContainerListWidget
//     ContainerListWidget::render(container_widget, f, app, layout.main, theme);

//     // ステータスバーエリア: StatusBar
//     StatusBar::render(f, app, layout.status);

//     // ヘルプエリア（利用可能な場合）
//     if layout.help.height > 0 && layout.help.width > 0 {
//         render_help_area(f, layout.help, theme);
//     }
// }

/// Render help area with key bindings
/// キーバインド付きヘルプエリアレンダリング
fn render_help_area(f: &mut ratatui::Frame, area: ratatui::layout::Rect, theme: &Theme) {
    use ratatui::{
        text::{Line, Span, Text},
        widgets::{Block, Borders, Paragraph},
    };

    // ヘルプテキストの作成
    let help_spans = vec![
        Span::styled("j/k", theme.styles.success_style()),
        Span::styled(": navigate | ", theme.styles.muted_style()),
        Span::styled("r", theme.styles.success_style()),
        Span::styled(": refresh | ", theme.styles.muted_style()),
        Span::styled("Enter", theme.styles.success_style()),
        Span::styled(": select | ", theme.styles.muted_style()),
        Span::styled("q", theme.styles.error_style()),
        Span::styled(": quit", theme.styles.muted_style()),
    ];

    let help_text = Text::from(Line::from(help_spans));

    let help_paragraph = Paragraph::new(help_text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Help")
                .border_style(theme.blocks.normal_border_style)
                .title_style(theme.styles.normal_style()),
        )
        .style(theme.styles.normal_style());

    f.render_widget(help_paragraph, area);
}

/// Render the user interface with full widget integration
/// 完全なウィジェット統合でユーザーインターフェースをレンダリング
///
/// This function handles the UI rendering logic using the full widget system
/// implemented in Task 1.2.2 and 1.2.3. It provides advanced layout management,
/// status bar integration, and responsive design.
///
/// この関数はTask 1.2.2と1.2.3で実装された完全なウィジェットシステムを使用して
/// UIレンダリングロジックを処理します。高度なレイアウト管理、ステータスバー統合、
/// レスポンシブデザインを提供します。
///
/// # Arguments
/// * `terminal` - Terminal instance for rendering
/// * `app` - Application state to render
/// * `container_widget` - Container list widget state
/// * `theme` - Theme configuration for styling
///
/// # Returns
/// * `Ok(())` - Rendering successful
/// * `Err(DockaError)` - Rendering failed
///
/// # Phase Migration
/// This function replaces the basic `render_ui` from Phase 1 with the full
/// widget-based implementation planned for Task 1.2.2 and 1.2.3.
///
/// この関数はPhase 1の基本的な`render_ui`を、Task 1.2.2と1.2.3で計画された
/// 完全なウィジェットベース実装に置き換えます。
fn render_ui<B: Backend>(
    terminal: &mut Terminal<B>,
    app: &App,
    container_widget: &mut ContainerListWidget,
    theme: &Theme,
) -> DockaResult<()> {
    terminal
        .draw(|f| {
            // レスポンシブレイアウトを計算
            let layout = SimpleLayout::calculate_responsive(f.area());

            // メインエリア: ContainerListWidget
            ContainerListWidget::render(container_widget, f, app, layout.main, theme);

            // ステータスバーエリア: StatusBar
            StatusBar::render(f, app, layout.status);

            // ヘルプエリア（利用可能な場合）
            if layout.help.height > 0 && layout.help.width > 0 {
                render_help_area(f, layout.help, theme);
            }
        })
        // === 修正: CompletedFrame を () に変換 ===
        .map(|_| ()) // CompletedFrame<'_> を () に変換
        .map_err(|e| DockaError::UiRendering {
            message: format!("Failed to render UI: {}", e),
        })
}

/// Initialize tracing for development debugging
/// 開発デバッグ用トレース初期化
#[cfg(debug_assertions)]
fn init_tracing() -> Result<(), Box<dyn std::error::Error>> {
    use tracing_subscriber;

    // Simple tracing setup without env-filter
    // env-filterなしのシンプルなtracing設定
    tracing_subscriber::fmt()
        .with_target(false)
        .with_thread_ids(true)
        .with_file(true)
        .with_line_number(true)
        .with_level(true)
        .try_init()
        .map_err(|e| format!("Failed to initialize tracing: {}", e))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore] // Requires actual Docker environment
    async fn test_docker_connection_real() {
        // Note: This test requires a running Docker daemon
        // 注意: このテストは動作中のDockerデーモンが必要です
        match BollardDockerRepository::new().await {
            Ok(repo) => {
                let repo = Arc::new(repo);
                // Test actual Docker connection
                match test_docker_connection(&repo).await {
                    Ok(()) => println!("Docker connection test passed"),
                    Err(e) => println!("Docker connection failed (expected in CI): {}", e),
                }
            }
            Err(e) => {
                println!("Docker initialization failed (expected in CI): {}", e);
            }
        }
    }

    #[test]
    fn test_setup_and_cleanup_terminal() {
        // Note: This test might fail in CI environments without a proper terminal
        // 注意: このテストは適切なターミナルがないCI環境では失敗する可能性があります
        if std::env::var("CI").is_ok() {
            return; // Skip in CI
        }

        // Test terminal setup
        let terminal_result = setup_terminal();
        if let Ok(mut terminal) = terminal_result {
            // Test cleanup
            assert!(cleanup_terminal(&mut terminal).is_ok());
        }
        // If setup fails, that's also acceptable in test environments
        // セットアップが失敗した場合も、テスト環境では許容される
    }

    #[test]
    fn test_constants() {
        // Test application constants
        // アプリケーション定数のテスト
        assert!(!APP_NAME.is_empty());
        assert!(!VERSION.is_empty());
        assert!(EVENT_POLL_INTERVAL.as_millis() > 0);
        assert!(STARTUP_TIMEOUT.as_secs() > 0);
    }

    #[test]
    fn test_app_configuration() {
        // Test basic application configuration
        // 基本アプリケーション設定のテスト
        assert_eq!(APP_NAME, "docka");
        assert_eq!(EVENT_POLL_INTERVAL, Duration::from_millis(100));
        assert_eq!(STARTUP_TIMEOUT, Duration::from_secs(3));
    }
}
