// src/ui/events.rs
// Event handling implementation for TUI application
// TUIアプリケーション用イベント処理実装

use crate::error::DockaResult;
use crate::ui::app::App;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

/// Application event enum representing user actions
/// ユーザーアクションを表すアプリケーションイベント列挙型
///
/// This enum defines all possible user interactions that can be processed
/// by the application, providing a clean abstraction over raw keyboard input.
///
/// この列挙型はアプリケーションで処理可能な全ユーザー交互作用を定義し、
/// 生のキーボード入力に対するクリーンな抽象化を提供します。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AppEvent {
    /// Normal quit request (q, Esc)
    /// 通常の終了要求 (q, Esc)
    Quit,

    /// Force quit request (Ctrl+C)
    /// 強制終了要求 (Ctrl+C)
    ForceQuit,

    /// Refresh containers list (r, F5)
    /// コンテナリスト更新 (r, F5)
    Refresh,

    /// Select next container (j, Down)
    /// 次のコンテナ選択 (j, Down)
    SelectNext,

    /// Select previous container (k, Up)
    /// 前のコンテナ選択 (k, Up)
    SelectPrevious,

    /// Enter/activate current selection (Enter)
    /// 現在の選択をアクティベート (Enter)
    Enter,

    /// Unknown or unhandled key
    /// 不明または未処理のキー
    Unknown,
}

/// Convert `KeyEvent` to `AppEvent` based on key bindings
/// `キーバインドに基づいてKeyEventをAppEventに変換`
///
/// This function implements vim-style key bindings with standard navigation keys.
/// It provides a consistent mapping from raw keyboard input to application events.
///
/// この関数は標準ナビゲーションキー付きのvimスタイルキーバインドを実装します。
/// 生のキーボード入力からアプリケーションイベントへの一貫したマッピングを提供します。
///
/// # Key Bindings
/// - `j`, `Down` - Select next container
/// - `k`, `Up` - Select previous container
/// - `q`, `Esc` - Normal quit
/// - `Ctrl+C` - Force quit
/// - `r`, `F5` - Refresh containers
/// - `Enter` - Activate selection
///
/// # Arguments
/// * `key_event` - Raw keyboard event from crossterm
///
/// # Returns
/// * `AppEvent` - Corresponding application event
///
/// # Examples
///
/// ```rust
/// use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
/// use docka::ui::events::{handle_key_event, AppEvent};
///
/// let key_j = KeyEvent::new(KeyCode::Char('j'), KeyModifiers::NONE);
/// assert_eq!(handle_key_event(key_j), AppEvent::SelectNext);
///
/// let key_quit = KeyEvent::new(KeyCode::Char('q'), KeyModifiers::NONE);
/// assert_eq!(handle_key_event(key_quit), AppEvent::Quit);
///
/// let key_force_quit = KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL);
/// assert_eq!(handle_key_event(key_force_quit), AppEvent::ForceQuit);
/// ```
#[must_use]
pub const fn handle_key_event(key_event: KeyEvent) -> AppEvent {
    match key_event.code {
        // Navigation - vim-style bindings
        // ナビゲーション - vimスタイルバインド
        KeyCode::Char('j') | KeyCode::Down => AppEvent::SelectNext,
        KeyCode::Char('k') | KeyCode::Up => AppEvent::SelectPrevious,

        // Action keys
        // アクションキー
        KeyCode::Enter => AppEvent::Enter,

        // Refresh operations
        // 更新操作
        KeyCode::Char('r') | KeyCode::F(5) => AppEvent::Refresh,

        // Quit operations with modifier check
        // モディファイア確認付き終了操作
        KeyCode::Char('q') | KeyCode::Esc => AppEvent::Quit,
        KeyCode::Char('c') if key_event.modifiers.contains(KeyModifiers::CONTROL) => {
            AppEvent::ForceQuit
        }

        // Unknown key
        // 不明なキー
        _ => AppEvent::Unknown,
    }
}

/// Process application event and update app state
/// アプリケーションイベントを処理してアプリケーション状態を更新
///
/// This function takes an application event and updates the app state accordingly.
/// It handles both synchronous state changes and triggers for asynchronous operations.
///
/// この関数はアプリケーションイベントを受け取り、それに応じてアプリケーション状態を更新します。
/// 同期状態変更と非同期操作のトリガーの両方を処理します。
///
/// # Arguments
/// * `app` - Mutable reference to application state
/// * `event` - Application event to process
///
/// # Returns
/// * `Ok(())` - Event processed successfully
/// * `Err(DockaError)` - Error occurred during event processing
///
/// # Errors
///
/// This function will return an error if:
/// * `AppEvent::Refresh` fails to fetch containers from Docker daemon
/// * Docker daemon is not accessible during refresh operation
/// * Network or permission errors occur during Docker API calls
///
/// この関数は以下の場合にエラーを返します：
/// * `AppEvent::Refresh`でDockerデーモンからのコンテナ取得が失敗した場合
/// * リフレッシュ操作中にDockerデーモンにアクセスできない場合
/// * Docker API呼び出し中にネットワークまたは権限エラーが発生した場合
///
/// # Examples
///
/// ```rust,no_run
/// use std::sync::Arc;
/// use docka::infrastructure::BollardDockerRepository;
/// use docka::ui::app::App;
/// use docka::ui::events::{process_app_event, AppEvent};
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let docker_repo = Arc::new(BollardDockerRepository::new().await?);
///     let mut app = App::new(docker_repo);
///
///     // Process navigation event
///     process_app_event(&mut app, AppEvent::SelectNext).await?;
///
///     // Process refresh event
///     process_app_event(&mut app, AppEvent::Refresh).await?;
///
///     Ok(())
/// }
/// ```
pub async fn process_app_event(app: &mut App, event: AppEvent) -> DockaResult<()> {
    match event {
        AppEvent::Quit => {
            app.quit();
            Ok(())
        }

        AppEvent::ForceQuit => {
            app.force_quit();
            Ok(())
        }

        AppEvent::Refresh => {
            // Trigger async container refresh
            // 非同期コンテナ更新をトリガー
            app.refresh_containers().await
        }

        AppEvent::SelectNext => {
            app.select_next();
            Ok(())
        }

        AppEvent::SelectPrevious => {
            app.select_previous();
            Ok(())
        }

        AppEvent::Enter => {
            // Phase 1: No action implementation yet
            // Phase 1: アクション実装はまだなし
            // This will be expanded in Phase 1.4 for container operations
            // これはPhase 1.4でコンテナ操作用に拡張される予定
            Ok(())
        }

        AppEvent::Unknown => {
            // Unknown events are silently ignored
            // 不明なイベントは無視される
            Ok(())
        }
    }
}

/// Validate key input to filter out control characters
/// 制御文字をフィルタリングするためのキー入力検証
///
/// This function validates keyboard input to ensure only appropriate
/// keys are processed by the application using an allowlist approach.
///
/// この関数はアローリスト方式を使用して、アプリケーションで適切なキーのみが
/// 処理されることを保証するためにキーボード入力を検証します。
///
/// # Arguments
/// * `key_event` - Keyboard event to validate
///
/// # Returns
/// * `true` - Key input is valid and should be processed
/// * `false` - Key input should be ignored
///
/// # Examples
///
/// ```rust
/// use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
/// use docka::ui::events::validate_key_input;
///
/// let normal_key = KeyEvent::new(KeyCode::Char('j'), KeyModifiers::NONE);
/// assert!(validate_key_input(normal_key));
///
/// let ctrl_key = KeyEvent::new(KeyCode::Char('a'), KeyModifiers::CONTROL);
/// assert!(validate_key_input(ctrl_key)); // Ctrl combinations are allowed
///
/// let function_key = KeyEvent::new(KeyCode::F(1), KeyModifiers::NONE);
/// assert!(validate_key_input(function_key));
/// ```
#[must_use]
pub const fn validate_key_input(key_event: KeyEvent) -> bool {
    // Allowlist approach - all allowed keys in single match arm
    // アローリスト方式 - 許可される全キーを単一のmatchアームで処理
    match key_event.code {
        // All allowed keys: characters, navigation, actions, function keys, and editing keys
        // 許可される全キー: 文字、ナビゲーション、アクション、ファンクション、編集キー
        KeyCode::Char(_)                                        // Printable characters / 印刷可能文字
        | KeyCode::Up | KeyCode::Down | KeyCode::Left | KeyCode::Right  // Navigation / ナビゲーション
        | KeyCode::Enter | KeyCode::Esc | KeyCode::Tab | KeyCode::BackTab  // Core actions / コアアクション
        | KeyCode::F(_)                                         // Function keys / ファンクションキー
        | KeyCode::Backspace | KeyCode::Delete                  // Basic editing / 基本編集
        | KeyCode::Home | KeyCode::End                          // Line navigation / 行ナビゲーション
        | KeyCode::PageUp | KeyCode::PageDown => true,         // Page navigation / ページナビゲーション

        // Reject all other keys (Insert, Pause, ScrollLock, etc.)
        // その他全てのキーを拒否 (Insert, Pause, ScrollLock等)
        _ => false,
    }
}

/// Event processing statistics for monitoring
/// 監視用イベント処理統計
///
/// This struct tracks event processing metrics for performance monitoring
/// and debugging purposes.
///
/// この構造体はパフォーマンス監視とデバッグ目的で
/// イベント処理メトリクスを追跡します。
#[derive(Debug, Clone, Default)]
pub struct EventStats {
    /// Total number of events processed
    /// 処理されたイベントの総数
    pub total_events: u64,

    /// Number of navigation events
    /// ナビゲーションイベントの数
    pub navigation_events: u64,

    /// Number of action events
    /// アクションイベントの数
    pub action_events: u64,

    /// Number of unknown events (should be low)
    /// 不明なイベントの数（少ないはず）
    pub unknown_events: u64,

    /// Number of processing errors
    /// 処理エラーの数
    pub error_count: u64,
}

impl EventStats {
    /// Create new event statistics tracker
    /// 新しいイベント統計トラッカーを作成
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Record an event in statistics
    /// 統計にイベントを記録
    ///
    /// # Arguments
    /// * `event` - Application event to record
    /// * `result` - Result of event processing
    pub const fn record_event(&mut self, event: &AppEvent, result: &DockaResult<()>) {
        self.total_events += 1;

        // Count by event type
        // イベントタイプ別カウント
        match event {
            AppEvent::SelectNext | AppEvent::SelectPrevious => {
                self.navigation_events += 1;
            }
            AppEvent::Enter | AppEvent::Refresh | AppEvent::Quit | AppEvent::ForceQuit => {
                self.action_events += 1;
            }
            AppEvent::Unknown => {
                self.unknown_events += 1;
            }
        }

        // Count errors
        // エラーカウント
        if result.is_err() {
            self.error_count += 1;
        }
    }

    /// Get error rate as percentage
    /// エラー率をパーセンテージで取得
    #[allow(clippy::cast_precision_loss)] // Acceptable for statistics calculation
    #[must_use]
    pub fn error_rate(&self) -> f64 {
        if self.total_events == 0 {
            0.0
        } else {
            (self.error_count as f64 / self.total_events as f64) * 100.0
        }
    }

    /// Reset all statistics
    /// 全統計をリセット
    pub fn reset(&mut self) {
        *self = Self::default();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::MockDockerRepository;
    use crate::error::{DockaError, DockaResult};
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
    use std::sync::Arc;

    fn create_key_event(code: KeyCode, modifiers: KeyModifiers) -> KeyEvent {
        KeyEvent::new(code, modifiers)
    }

    fn create_test_app() -> App {
        let mock_repo = Arc::new(MockDockerRepository::new());
        App::new(mock_repo)
    }

    #[test]
    fn test_handle_key_event_navigation() {
        // Test vim-style navigation
        assert_eq!(
            handle_key_event(create_key_event(KeyCode::Char('j'), KeyModifiers::NONE)),
            AppEvent::SelectNext
        );
        assert_eq!(
            handle_key_event(create_key_event(KeyCode::Char('k'), KeyModifiers::NONE)),
            AppEvent::SelectPrevious
        );

        // Test arrow key navigation
        assert_eq!(
            handle_key_event(create_key_event(KeyCode::Down, KeyModifiers::NONE)),
            AppEvent::SelectNext
        );
        assert_eq!(
            handle_key_event(create_key_event(KeyCode::Up, KeyModifiers::NONE)),
            AppEvent::SelectPrevious
        );
    }

    #[test]
    fn test_handle_key_event_actions() {
        // Test action keys
        assert_eq!(
            handle_key_event(create_key_event(KeyCode::Enter, KeyModifiers::NONE)),
            AppEvent::Enter
        );
        assert_eq!(
            handle_key_event(create_key_event(KeyCode::Char('r'), KeyModifiers::NONE)),
            AppEvent::Refresh
        );
        assert_eq!(
            handle_key_event(create_key_event(KeyCode::F(5), KeyModifiers::NONE)),
            AppEvent::Refresh
        );
    }

    #[test]
    fn test_handle_key_event_quit() {
        // Test normal quit
        assert_eq!(
            handle_key_event(create_key_event(KeyCode::Char('q'), KeyModifiers::NONE)),
            AppEvent::Quit
        );
        assert_eq!(
            handle_key_event(create_key_event(KeyCode::Esc, KeyModifiers::NONE)),
            AppEvent::Quit
        );

        // Test force quit
        assert_eq!(
            handle_key_event(create_key_event(KeyCode::Char('c'), KeyModifiers::CONTROL)),
            AppEvent::ForceQuit
        );
    }

    #[test]
    fn test_handle_key_event_unknown() {
        // Test unknown keys
        assert_eq!(
            handle_key_event(create_key_event(KeyCode::Char('x'), KeyModifiers::NONE)),
            AppEvent::Unknown
        );
        assert_eq!(
            handle_key_event(create_key_event(KeyCode::F(12), KeyModifiers::NONE)),
            AppEvent::Unknown
        );
    }

    #[tokio::test]
    async fn test_process_app_event_navigation() {
        let mut app = create_test_app();

        // Add some test containers
        app.containers = vec![
            crate::domain::ContainerBuilder::new()
                .id(crate::domain::ContainerId::new("1").unwrap())
                .name("test1".to_string())
                .image("image1".to_string())
                .status(crate::domain::ContainerStatus::Running)
                .build()
                .unwrap(),
            crate::domain::ContainerBuilder::new()
                .id(crate::domain::ContainerId::new("2").unwrap())
                .name("test2".to_string())
                .image("image2".to_string())
                .status(crate::domain::ContainerStatus::Stopped)
                .build()
                .unwrap(),
        ];
        app.selected_index = 0;

        // Test navigation events
        assert!(
            process_app_event(&mut app, AppEvent::SelectNext)
                .await
                .is_ok()
        );
        assert_eq!(app.selected_index, 1);

        assert!(
            process_app_event(&mut app, AppEvent::SelectPrevious)
                .await
                .is_ok()
        );
        assert_eq!(app.selected_index, 0);
    }

    #[tokio::test]
    async fn test_process_app_event_quit() {
        let mut app = create_test_app();

        // Test normal quit
        assert!(process_app_event(&mut app, AppEvent::Quit).await.is_ok());
        assert!(!app.is_running());

        // Reset app
        app.should_quit = false;

        // Test force quit
        assert!(
            process_app_event(&mut app, AppEvent::ForceQuit)
                .await
                .is_ok()
        );
        assert!(!app.is_running());
        assert!(!app.running);
    }

    #[tokio::test]
    async fn test_process_app_event_unknown() {
        let mut app = create_test_app();

        // Unknown events should not cause errors
        assert!(process_app_event(&mut app, AppEvent::Unknown).await.is_ok());
        assert!(process_app_event(&mut app, AppEvent::Enter).await.is_ok());
    }

    #[test]
    fn test_validate_key_input() {
        // Valid keys
        assert!(validate_key_input(create_key_event(
            KeyCode::Char('a'),
            KeyModifiers::NONE
        )));
        assert!(validate_key_input(create_key_event(
            KeyCode::Enter,
            KeyModifiers::NONE
        )));
        assert!(validate_key_input(create_key_event(
            KeyCode::Up,
            KeyModifiers::NONE
        )));
        assert!(validate_key_input(create_key_event(
            KeyCode::F(1),
            KeyModifiers::NONE
        )));

        // Invalid keys - these should return false as our function is restrictive
        // 無効なキー - この関数は制限的なので false を返すべき
        assert!(!validate_key_input(create_key_event(
            KeyCode::Insert,
            KeyModifiers::NONE
        )));

        // Valid editing keys
        // 有効な編集キー
        assert!(validate_key_input(create_key_event(
            KeyCode::Backspace,
            KeyModifiers::NONE
        )));
        assert!(validate_key_input(create_key_event(
            KeyCode::Delete,
            KeyModifiers::NONE
        )));
    }

    #[test]
    fn test_event_stats() {
        let mut stats = EventStats::new();

        // Record some events
        let ok_result = Ok(());
        let err_result: DockaResult<()> = Err(DockaError::not_implemented("test"));

        stats.record_event(&AppEvent::SelectNext, &ok_result);
        stats.record_event(&AppEvent::Refresh, &ok_result);
        stats.record_event(&AppEvent::Unknown, &err_result);

        assert_eq!(stats.total_events, 3);
        assert_eq!(stats.navigation_events, 1);
        assert_eq!(stats.action_events, 1);
        assert_eq!(stats.unknown_events, 1);
        assert_eq!(stats.error_count, 1);

        // Test error rate calculation
        assert!((stats.error_rate() - 33.333333333333336).abs() < 0.01);

        // Test reset
        stats.reset();
        assert_eq!(stats.total_events, 0);
        assert_eq!(stats.error_rate(), 0.0);
    }
}
