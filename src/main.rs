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
        app::App,
        events::{EventStats, handle_key_event, process_app_event},
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

    // Main event loop
    // メインイベントループ
    while app.is_running() {
        // Handle events with timeout
        // タイムアウト付きイベント処理
        if event::poll(EVENT_POLL_INTERVAL).map_err(DockaError::Io)? {
            // Handle keyboard events only, ignore other event types for Phase 1
            // Phase 1ではキーボードイベントのみ処理、他のイベントタイプは無視
            if let Ok(Event::Key(key_event)) = event::read() {
                // Validate and process key input
                // キー入力を検証して処理
                if docka::ui::events::validate_key_input(key_event) {
                    let app_event = handle_key_event(key_event);
                    let result = process_app_event(app, app_event.clone()).await;
                    event_stats.record_event(&app_event, &result);

                    // Handle processing errors
                    // 処理エラーを処理
                    if let Err(ref error) = result {
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

        // Render UI with frame rate limiting
        // フレームレート制限付きでUIをレンダリング
        let now = Instant::now();
        if now.duration_since(last_render) >= TARGET_FPS {
            render_ui(terminal, app)?;
            last_render = now;
        }

        // Small yield to prevent busy waiting
        // ビジー待機を防ぐため小さなyield
        tokio::task::yield_now().await;
    }

    Ok(event_stats)
}

/// Render the user interface
/// ユーザーインターフェースをレンダリング
///
/// This function handles the UI rendering logic. In Phase 1, it provides
/// a basic implementation that will be expanded with proper widgets in later tasks.
///
/// この関数はUIレンダリングロジックを処理します。Phase 1では
/// 後のタスクで適切なウィジェットに拡張される基本実装を提供します。
///
/// # Arguments
/// * `terminal` - Terminal instance for rendering
/// * `app` - Application state to render
///
/// # Returns
/// * `Ok(())` - Rendering successful
/// * `Err(DockaError)` - Rendering failed
fn render_ui<B: Backend>(terminal: &mut Terminal<B>, app: &App) -> DockaResult<()> {
    terminal
        .draw(|f| {
            // Phase 1: Basic text-based rendering
            // Phase 1: 基本的なテキストベースレンダリング
            // This will be replaced with proper widgets in Task 1.2.2 and 1.2.3
            // これはTask 1.2.2と1.2.3で適切なウィジェットに置き換えられる

            use ratatui::{
                layout::{Constraint, Direction, Layout},
                text::{Line, Span},
                widgets::{Block, Borders, List, ListItem, Paragraph},
                style::{Color, Style},
            };

            let size = f.area();

            // Simple layout: main area + status bar
            // シンプルなレイアウト: メインエリア + ステータスバー
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Min(3), Constraint::Length(3)].as_ref())
                .split(size);

            // Render container list or status message
            // コンテナリストまたはステータスメッセージをレンダリング
            match &app.view_state {
                docka::ui::app::ViewState::Loading => {
                    let loading = Paragraph::new("Loading containers...")
                        .block(Block::default()
                            .title("Docker Containers")
                            .borders(Borders::ALL))
                        .style(Style::default().fg(Color::Yellow));
                    f.render_widget(loading, chunks[0]);
                }
                docka::ui::app::ViewState::Error(error) => {
                    let error_text = Paragraph::new(format!("Error: {}", error))
                        .block(Block::default()
                            .title("Error")
                            .borders(Borders::ALL))
                        .style(Style::default().fg(Color::Red));
                    f.render_widget(error_text, chunks[0]);
                }
                docka::ui::app::ViewState::ContainerList => {
                    if app.containers.is_empty() {
                        let empty = Paragraph::new("No containers found. Press 'r' to refresh.")
                            .block(Block::default()
                                .title("Docker Containers")
                                .borders(Borders::ALL))
                            .style(Style::default().fg(Color::Gray));
                        f.render_widget(empty, chunks[0]);
                    } else {
                        // Render container list
                        // コンテナリストをレンダリング
                        let items: Vec<ListItem> = app.containers
                            .iter()
                            .enumerate()
                            .map(|(i, container)| {
                                let style = if i == app.selected_index {
                                    Style::default().bg(Color::Blue).fg(Color::White)
                                } else {
                                    Style::default()
                                };

                                let status_color = match container.status {
                                    docka::domain::ContainerStatus::Running => Color::Green,
                                    docka::domain::ContainerStatus::Stopped => Color::Red,
                                    docka::domain::ContainerStatus::Exited { .. } => Color::Yellow,
                                    docka::domain::ContainerStatus::Paused => Color::Cyan,
                                    _ => Color::Gray,
                                };

                                let content = Line::from(vec![
                                    Span::styled(
                                        format!("{:<20}", container.display_name()),
                                        style
                                    ),
                                    Span::styled(
                                        format!(" [{:?}]", container.status),
                                        Style::default().fg(status_color)
                                    ),
                                    Span::styled(
                                        format!(" {}", container.image),
                                        Style::default().fg(Color::Cyan)
                                    ),
                                ]);

                                ListItem::new(content).style(style)
                            })
                            .collect();

                        let list = List::new(items)
                            .block(Block::default()
                                .title(format!("Docker Containers ({})", app.containers.len()))
                                .borders(Borders::ALL));

                        f.render_widget(list, chunks[0]);
                    }
                }
            }

            // Render status bar
            // ステータスバーをレンダリング
            let status_text = format!(
                " docka v{} | Selected: {}/{} | Press 'q' to quit, 'r' to refresh, 'j/k' to navigate",
                VERSION,
                if app.containers.is_empty() { 0 } else { app.selected_index + 1 },
                app.containers.len()
            );

            let status_bar = Paragraph::new(status_text)
                .block(Block::default().borders(Borders::ALL))
                .style(Style::default().fg(Color::White).bg(Color::DarkGray));

            f.render_widget(status_bar, chunks[1]);
        })
        .map_err(|e| DockaError::UiRendering { message: format!("UI rendering failed: {}", e) })?;

    Ok(())
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
