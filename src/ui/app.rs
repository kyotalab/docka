// src/ui/app.rs
// Basic App structure implementation for TUI application
// TUIアプリケーション用基本App構造実装

use crate::domain::entities::Container;
use crate::domain::repositories::DockerRepository;
use crate::error::DockaResult;
use std::sync::Arc;
use std::time::Instant;

/// View state enum representing current application UI state
/// `現在のアプリケーションUI状態を表すViewState列挙型`
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ViewState {
    /// Container list view - normal operation state
    /// コンテナリストビュー - 通常操作状態
    ContainerList,
    /// Loading state during async operations
    /// 非同期操作中のローディング状態
    Loading,
    /// Error state with error message
    /// エラーメッセージ付きエラー状態
    Error(String),
}

/// Navigation direction for container selection
/// コンテナ選択のナビゲーション方向
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NavigationDirection {
    /// Move to next container (j, Down key)
    /// 次のコンテナに移動 (j, Down キー)
    Next,

    /// Move to previous container (k, Up key)
    /// 前のコンテナに移動 (k, Up キー)
    Previous,
}

/// Main application state struct managing TUI application
/// TUIアプリケーションを管理するメインアプリケーション状態構造体
///
/// This struct maintains all application state including containers list,
/// UI state, selected index, and Docker repository integration.
///
/// この構造体はコンテナリスト、UI状態、選択インデックス、
/// Dockerリポジトリ統合を含む全アプリケーション状態を維持します。
///
/// # Examples
///
/// ```rust,no_run
/// use std::sync::Arc;
/// use docka::infrastructure::BollardDockerRepository;
/// use docka::ui::app::App;
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let docker_repo = Arc::new(BollardDockerRepository::new().await?);
///     let mut app = App::new(docker_repo);
///
///     // Initialize container data
///     app.refresh_containers().await?;
///
///     Ok(())
/// }
/// ```
pub struct App {
    /// Application running state flag
    /// アプリケーション実行状態フラグ
    pub running: bool,

    /// Should quit flag for graceful shutdown
    /// グレースフルシャットダウン用終了フラグ
    pub should_quit: bool,

    /// Current list of containers
    /// 現在のコンテナリスト
    pub containers: Vec<Container>,

    /// Currently selected container index
    /// 現在選択されているコンテナのインデックス
    pub selected_index: usize,

    /// Current view state
    /// 現在のビュー状態
    pub view_state: ViewState,

    /// Docker repository for API operations
    /// `API操作用Dockerリポジトリ`
    docker_repository: Arc<dyn DockerRepository>,

    /// Last error message for display purposes
    /// 表示用の最後のエラーメッセージ
    pub last_error: Option<String>,

    /// Last activity timestamp for rendering optimization
    /// レンダリング最適化用最後のアクティビティタイムスタンプ
    pub last_activity: Instant,

    /// Help area visibility toggle state
    /// ヘルプエリア表示切り替え状態
    ///
    /// Controls whether the help area is displayed in the TUI interface.
    /// When enabled, shows keyboard shortcuts and command help at the bottom
    /// of the screen. The help area is responsive and automatically hidden
    /// on smaller terminals to preserve main content visibility.
    ///
    /// TUIインターフェースでヘルプエリアを表示するかを制御します。
    /// 有効時は、画面下部にキーボードショートカットとコマンドヘルプを表示します。
    /// ヘルプエリアはレスポンシブで、小さなターミナルでは自動的に非表示になり、
    /// メインコンテンツの可視性を保持します。
    ///
    /// # Default Value
    /// `false` - Help is initially hidden to maximize container list space
    /// `false` - コンテナリストスペース最大化のため初期状態では非表示
    ///
    /// # Usage
    /// - Toggle with '?' key or explicit toggle_help() method call
    /// - Automatically respected by layout system (SimpleLayout)
    /// - Only shown when terminal has sufficient height (≥10 rows)
    ///
    /// - '?'キーまたは明示的なtoggle_help()メソッド呼び出しで切り替え
    /// - レイアウトシステム（SimpleLayout）により自動的に考慮される
    /// - ターミナルに十分な高さ（≥10行）がある場合のみ表示
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use std::sync::Arc;
    /// # use docka::{ui::app::App, infrastructure::BollardDockerRepository};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let docker_repo = Arc::new(BollardDockerRepository::new().await?);
    /// let mut app = App::new(docker_repo);
    ///
    /// // Initially hidden
    /// assert!(!app.show_help());
    ///
    /// // Toggle to show help
    /// app.toggle_help();
    /// assert!(app.show_help());
    ///
    /// // Toggle to hide help
    /// app.toggle_help();
    /// assert!(!app.show_help());
    /// # Ok(())
    /// # }
    /// ```
    pub show_help: bool,
}

impl App {
    /// Create new App instance with Docker repository
    /// `Dockerリポジトリを使用して新しいAppインスタンスを作成`
    ///
    /// # Arguments
    /// * `docker_repository` - Docker API repository implementation
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use std::sync::Arc;
    /// use docka::infrastructure::BollardDockerRepository;
    /// use docka::ui::app::App;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let docker_repo = Arc::new(BollardDockerRepository::new().await?);
    ///     let app = App::new(docker_repo);
    ///     assert_eq!(app.containers.len(), 0);
    ///     Ok(())
    /// }
    /// ```
    pub fn new(docker_repository: Arc<dyn DockerRepository>) -> Self {
        Self {
            running: true,
            should_quit: false,
            containers: Vec::new(),
            selected_index: 0,
            view_state: ViewState::Loading,
            docker_repository,
            last_error: None,
            last_activity: Instant::now(), // 初期化を追加
            show_help: false,              // <- 新規追加
        }
    }

    /// Refresh containers from Docker API
    /// Docker APIからコンテナを更新
    ///
    /// This method fetches the latest container list from Docker daemon
    /// and updates the application state accordingly.
    ///
    /// このメソッドはDockerデーモンから最新のコンテナリストを取得し、
    /// それに応じてアプリケーション状態を更新します。
    ///
    /// # Returns
    /// * `Ok(())` - Successfully refreshed containers
    /// * `Err(DockaError)` - Failed to fetch containers
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// * Docker daemon is not running (`DockerDaemonNotRunning`)
    /// * Network connection to Docker daemon fails (`DockerApi`)
    /// * Docker API returns invalid data (`Serialization`)
    /// * Permission denied when accessing Docker daemon (`PermissionDenied`)
    ///
    /// この関数は以下の場合にエラーを返します：
    /// * Dockerデーモンが動作していない場合
    /// * Dockerデーモンへのネットワーク接続が失敗した場合
    /// * Docker APIが無効なデータを返した場合
    /// * Dockerデーモンへのアクセス権限が拒否された場合
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use std::sync::Arc;
    /// # use docka::infrastructure::BollardDockerRepository;
    /// # use docka::ui::app::App;
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let docker_repo = Arc::new(BollardDockerRepository::new().await?);
    /// let mut app = App::new(docker_repo);
    ///
    /// match app.refresh_containers().await {
    ///     Ok(()) => println!("Successfully refreshed {} containers", app.containers.len()),
    ///     Err(e) => eprintln!("Failed to refresh containers: {}", e),
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn refresh_containers(&mut self) -> DockaResult<()> {
        // Set loading state
        self.view_state = ViewState::Loading;
        self.last_error = None;
        self.last_activity = Instant::now(); // アクティビティ更新を追加

        match self.docker_repository.list_containers().await {
            Ok(containers) => {
                self.containers = containers;
                // Reset selected index if out of bounds
                if self.selected_index >= self.containers.len() && !self.containers.is_empty() {
                    self.selected_index = self.containers.len() - 1;
                } else if self.containers.is_empty() {
                    self.selected_index = 0;
                }
                self.view_state = ViewState::ContainerList;
                self.last_error = None; // Clear previous error
                Ok(())
            }
            Err(error) => {
                let error_message = error.to_string();
                self.view_state = ViewState::Error(error_message.clone());
                self.last_error = Some(error_message);
                Err(error)
            }
        }
    }

    /// Select next container in the list (循環ナビゲーション - 下方向)
    /// リスト内の次のコンテナを選択（循環ナビゲーション - 下方向）
    ///
    /// This method implements circular navigation, wrapping to the first
    /// container when at the end of the list.
    ///
    /// このメソッドは循環ナビゲーションを実装し、リストの最後にいる時は
    /// 最初のコンテナにラップします。
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use std::sync::Arc;
    /// # use docka::infrastructure::BollardDockerRepository;
    /// # use docka::ui::app::App;
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let docker_repo = Arc::new(BollardDockerRepository::new().await?);
    /// let mut app = App::new(docker_repo);
    ///
    /// // Load some containers first
    /// app.refresh_containers().await?;
    ///
    /// // Navigate through containers
    /// app.select_next();  // Move to next container
    /// app.select_next();  // Move to next container again
    /// # Ok(())
    /// # }
    /// ```
    pub fn select_next(&mut self) {
        if self.containers.is_empty() {
            return;
        }

        self.selected_index = (self.selected_index + 1) % self.containers.len();
        self.last_activity = Instant::now(); // アクティビティ更新を追加
    }

    /// Select previous container in the list (循環ナビゲーション - 上方向)
    /// リスト内の前のコンテナを選択（循環ナビゲーション - 上方向）
    ///
    /// This method implements circular navigation, wrapping to the last
    /// container when at the beginning of the list.
    ///
    /// このメソッドは循環ナビゲーションを実装し、リストの最初にいる時は
    /// 最後のコンテナにラップします。
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use std::sync::Arc;
    /// # use docka::infrastructure::BollardDockerRepository;
    /// # use docka::ui::app::App;
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let docker_repo = Arc::new(BollardDockerRepository::new().await?);
    /// let mut app = App::new(docker_repo);
    ///
    /// // Load some containers first
    /// app.refresh_containers().await?;
    ///
    /// // Navigate backwards through containers
    /// app.select_previous();  // Move to previous container (wraps to last)
    /// app.select_previous();  // Move to previous container
    /// # Ok(())
    /// # }
    /// ```
    pub fn select_previous(&mut self) {
        if self.containers.is_empty() {
            return;
        }

        if self.selected_index == 0 {
            self.selected_index = self.containers.len() - 1;
        } else {
            self.selected_index -= 1;
        }
        self.last_activity = Instant::now(); // アクティビティ更新を追加
    }

    /// Get currently selected container if any
    /// 現在選択されているコンテナを取得（存在する場合）
    ///
    /// # Returns
    /// * `Some(&Container)` - Currently selected container
    /// * `None` - No container selected or list is empty
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use std::sync::Arc;
    /// # use docka::infrastructure::BollardDockerRepository;
    /// # use docka::ui::app::App;
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let docker_repo = Arc::new(BollardDockerRepository::new().await?);
    /// let mut app = App::new(docker_repo);
    ///
    /// // Initially no container is selected (empty list)
    /// assert!(app.selected_container().is_none());
    ///
    /// // After loading containers, first one is selected by default
    /// app.refresh_containers().await?;
    /// if !app.containers.is_empty() {
    ///     assert!(app.selected_container().is_some());
    ///     let selected = app.selected_container().unwrap();
    ///     println!("Selected container: {}", selected.display_name());
    /// }
    /// # Ok(())
    /// # }
    /// ```
    #[must_use]
    pub fn selected_container(&self) -> Option<&Container> {
        self.containers.get(self.selected_index)
    }

    /// Check if the application should continue running
    /// アプリケーションが実行を継続すべきかチェック
    ///
    /// # Returns
    /// * `true` - Application should continue running
    /// * `false` - Application should quit
    #[must_use]
    pub const fn is_running(&self) -> bool {
        self.running && !self.should_quit
    }

    /// Request application quit
    /// アプリケーション終了を要求
    ///
    /// This sets the `should_quit` flag for graceful shutdown.
    /// `これはグレースフルシャットダウンのためのshould_quitフラグを設定します`。
    pub fn quit(&mut self) {
        self.should_quit = true;
        self.last_activity = Instant::now(); // アクティビティ更新を追加
    }

    /// Force application stop
    /// アプリケーション強制停止
    ///
    /// This immediately stops the application by setting running to false.
    /// これはrunningをfalseに設定してアプリケーションを即座に停止します。
    pub fn force_quit(&mut self) {
        self.running = false;
        self.should_quit = true;
        self.last_activity = Instant::now(); // アクティビティ更新を追加
    }

    /// Handle container navigation with widget state synchronization
    /// ウィジェット状態同期付きコンテナナビゲーション処理
    ///
    /// This method provides bidirectional synchronization between App state
    /// and ContainerListWidget state during navigation operations.
    ///
    /// このメソッドはナビゲーション操作中にApp状態と
    /// ContainerListWidget状態の双方向同期を提供します。
    ///
    /// # Arguments
    ///
    /// * `widget` - Mutable reference to ContainerListWidget
    /// * `direction` - Navigation direction (Next/Previous)
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use std::sync::Arc;
    /// use docka::{
    ///     ui::{App, ContainerListWidget, NavigationDirection},
    ///     infrastructure::BollardDockerRepository,
    /// };
    ///
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let docker_repo = Arc::new(BollardDockerRepository::new().await?);
    /// let mut app = App::new(docker_repo);
    /// let mut widget = ContainerListWidget::new();
    ///
    /// // Navigate to next container
    /// app.handle_container_navigation(&mut widget, NavigationDirection::Next);
    ///
    /// // Navigate to previous container
    /// app.handle_container_navigation(&mut widget, NavigationDirection::Previous);
    /// # Ok(())
    /// # }
    /// ```
    pub fn handle_container_navigation(
        &mut self,
        widget: &mut crate::ui::widgets::ContainerListWidget,
        direction: NavigationDirection,
    ) {
        // Skip navigation if no containers available
        // コンテナが利用できない場合はナビゲーションをスキップ
        if self.containers.is_empty() {
            return;
        }

        match direction {
            NavigationDirection::Next => {
                // Update widget selection first
                // まずウィジェット選択を更新
                widget.select_next(self.containers.len());

                // Synchronize app state with widget selection
                // アプリ状態をウィジェット選択と同期
                if let Some(selected) = widget.selected() {
                    self.selected_index = selected;
                }
            }
            NavigationDirection::Previous => {
                // Update widget selection first
                // まずウィジェット選択を更新
                widget.select_previous(self.containers.len());

                // Synchronize app state with widget selection
                // アプリ状態をウィジェット選択と同期
                if let Some(selected) = widget.selected() {
                    self.selected_index = selected;
                }
            }
        }

        // Update activity timestamp for rendering optimization
        // レンダリング最適化のためアクティビティタイムスタンプを更新
        self.last_activity = Instant::now();
    }

    /// Synchronize widget state with current app state
    /// 現在のアプリ状態とウィジェット状態を同期
    ///
    /// This method ensures that the widget's selection state matches
    /// the app's selected_index when needed.
    ///
    /// このメソッドは必要に応じてウィジェットの選択状態が
    /// アプリのselected_indexと一致することを保証します。
    ///
    /// # Arguments
    ///
    /// * `widget` - Mutable reference to ContainerListWidget
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use std::sync::Arc;
    /// use docka::{
    ///     ui::{App, ContainerListWidget},
    ///     infrastructure::BollardDockerRepository,
    /// };
    ///
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let docker_repo = Arc::new(BollardDockerRepository::new().await?);
    /// let mut app = App::new(docker_repo);
    /// let mut widget = ContainerListWidget::new();
    ///
    /// // Manually set app selection index
    /// // app.selected_index = 2;
    ///
    /// // Synchronize widget to match app state
    /// app.sync_widget_state(&mut widget);
    /// # Ok(())
    /// # }
    /// ```
    pub fn sync_widget_state(&self, widget: &mut crate::ui::widgets::ContainerListWidget) {
        if !self.containers.is_empty() && self.selected_index < self.containers.len() {
            widget.set_selected(Some(self.selected_index));
        } else {
            widget.set_selected(None);
        }
    }

    /// Check if rendering is needed based on recent activity
    /// 最近のアクティビティに基づいてレンダリングが必要かチェック
    ///
    /// This method helps optimize rendering by determining if a redraw
    /// is necessary based on recent user activity.
    ///
    /// このメソッドは最近のユーザーアクティビティに基づいて
    /// 再描画が必要かを判定することでレンダリングを最適化します。
    ///
    /// # Returns
    ///
    /// * `bool` - True if redraw is needed, false otherwise
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use std::sync::Arc;
    /// use docka::{ui::App, infrastructure::BollardDockerRepository};
    ///
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let docker_repo = Arc::new(BollardDockerRepository::new().await?);
    /// let app = App::new(docker_repo);
    ///
    /// if app.needs_redraw() {
    ///     // Perform rendering
    ///     println!("Redraw needed");
    /// }
    /// # Ok(())
    /// # }
    /// ```
    #[must_use]
    pub fn needs_redraw(&self) -> bool {
        // Consider redraw needed if activity within last 100ms
        // 最後の100ms以内のアクティビティがあれば再描画が必要と判定
        self.last_activity.elapsed().as_millis() < 100
    }

    /// Get the currently selected container reference
    /// 現在選択されているコンテナの参照を取得
    ///
    /// This method provides safe access to the currently selected container,
    /// returning None if no containers exist or if the selection is out of bounds.
    ///
    /// このメソッドは現在選択されているコンテナへの安全なアクセスを提供し、
    /// コンテナが存在しないか選択が範囲外の場合はNoneを返します。
    ///
    /// # Returns
    ///
    /// * `Option<&Container>` - Reference to selected container, if valid
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use std::sync::Arc;
    /// use docka::{ui::App, infrastructure::BollardDockerRepository};
    ///
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let docker_repo = Arc::new(BollardDockerRepository::new().await?);
    /// let mut app = App::new(docker_repo);
    ///
    /// // Load containers first
    /// app.refresh_containers().await?;
    ///
    /// if let Some(container) = app.get_selected_container() {
    ///     println!("Selected: {}", container.name());
    /// }
    /// # Ok(())
    /// # }
    /// ```
    #[must_use]
    pub fn get_selected_container(&self) -> Option<&crate::domain::entities::Container> {
        if self.selected_index < self.containers.len() {
            Some(&self.containers[self.selected_index])
        } else {
            None
        }
    }

    /// Check if the container list is empty
    /// コンテナリストが空かどうかをチェック
    ///
    /// # Returns
    ///
    /// * `bool` - True if no containers are loaded
    #[must_use]
    pub fn is_container_list_empty(&self) -> bool {
        self.containers.is_empty()
    }

    /// Validate that the current selected index is within bounds
    /// 現在の選択インデックスが範囲内にあることを検証
    ///
    /// # Returns
    ///
    /// * `bool` - True if selected_index is valid for current container list
    #[must_use]
    pub fn is_selected_index_valid(&self) -> bool {
        !self.containers.is_empty() && self.selected_index < self.containers.len()
    }

    /// Debug information for development and testing
    /// 開発とテスト用のデバッグ情報
    ///
    /// This method provides detailed state information for debugging purposes.
    /// Only available in debug builds.
    ///
    /// このメソッドはデバッグ目的で詳細な状態情報を提供します。
    /// デバッグビルドでのみ利用可能です。
    #[cfg(debug_assertions)]
    #[must_use]
    pub fn debug_info(&self) -> String {
        format!(
            "App Debug: containers={}, selected={}, view_state={:?}, needs_redraw={}, show_help={}",
            self.containers.len(),
            self.selected_index,
            self.view_state,
            self.needs_redraw(),
            self.show_help
        )
    }

    /// Toggle help area visibility
    /// ヘルプエリア表示の切り替え
    ///
    /// Toggles the visibility of the help area at the bottom of the TUI interface.
    /// The help area displays keyboard shortcuts and command information.
    /// Layout system automatically respects this setting and terminal size constraints.
    ///
    /// TUIインターフェース下部のヘルプエリアの表示を切り替えます。
    /// ヘルプエリアはキーボードショートカットとコマンド情報を表示します。
    /// レイアウトシステムはこの設定とターミナルサイズ制約を自動的に考慮します。
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use std::sync::Arc;
    /// # use docka::{ui::app::App, infrastructure::BollardDockerRepository};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let docker_repo = Arc::new(BollardDockerRepository::new().await?);
    /// let mut app = App::new(docker_repo);
    ///
    /// assert!(!app.show_help());
    /// app.toggle_help();
    /// assert!(app.show_help());
    /// app.toggle_help();
    /// assert!(!app.show_help());
    /// # Ok(())
    /// # }
    /// ```
    pub fn toggle_help(&mut self) {
        self.show_help = !self.show_help;
        self.last_activity = std::time::Instant::now();
    }

    /// Get current help area visibility state
    /// 現在のヘルプエリア表示状態を取得
    ///
    /// Returns whether the help area is currently set to be visible.
    /// Note that the actual visibility also depends on terminal size constraints
    /// handled by the layout system.
    ///
    /// ヘルプエリアが現在表示に設定されているかを返します。
    /// 実際の表示はレイアウトシステムで処理されるターミナルサイズ制約にも依存することに注意してください。
    ///
    /// # Returns
    /// * `true` - Help area is enabled for display
    /// * `false` - Help area is hidden
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use std::sync::Arc;
    /// # use docka::{ui::app::App, infrastructure::BollardDockerRepository};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let docker_repo = Arc::new(BollardDockerRepository::new().await?);
    /// let app = App::new(docker_repo);
    /// assert!(!app.show_help());
    /// # Ok(())
    /// # }
    /// ```
    #[must_use]
    pub const fn show_help(&self) -> bool {
        self.show_help
    }

    /// エラー状態を設定
    pub fn set_error_state(&mut self, error_message: String) {
        use crate::ui::app::ViewState;
        self.view_state = ViewState::Error(error_message);
        self.last_activity = std::time::Instant::now();
    }

    /// 成功状態を設定
    pub fn set_success_state(&mut self) {
        use crate::ui::app::ViewState;
        self.view_state = ViewState::ContainerList;
        self.last_activity = std::time::Instant::now();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::MockDockerRepository;
    use crate::domain::{ContainerBuilder, ContainerId, ContainerStatus};
    use std::sync::Arc;

    fn create_test_app() -> App {
        let mock_repo = Arc::new(MockDockerRepository::new());
        App::new(mock_repo)
    }

    fn create_test_container(id: &str, name: &str) -> Container {
        ContainerBuilder::new()
            .id(ContainerId::new(id).unwrap())
            .name(name.to_string())
            .image("test:latest".to_string())
            .status(ContainerStatus::Running)
            .build()
            .unwrap()
    }

    #[test]
    fn test_app_new() {
        let app = create_test_app();
        assert!(app.running);
        assert!(!app.should_quit);
        assert_eq!(app.containers.len(), 0);
        assert_eq!(app.selected_index, 0);
        assert_eq!(app.view_state, ViewState::Loading);
        assert!(app.last_error.is_none());
        assert!(app.last_activity.elapsed().as_secs() < 1);
    }

    #[test]
    fn test_needs_redraw() {
        let mut app = create_test_app();

        // 初期状態では再描画不要（100ms経過済み）
        std::thread::sleep(std::time::Duration::from_millis(110));
        assert!(!app.needs_redraw());

        // アクティビティ更新後は再描画必要
        app.last_activity = Instant::now();
        assert!(app.needs_redraw());
    }

    #[test]
    fn test_navigation_updates_activity() {
        let mut app = create_test_app();
        app.containers = vec![create_test_container("1", "test1")];

        let initial_activity = app.last_activity;

        // 小さな遅延で確実にタイムスタンプが変わることを保証
        std::thread::sleep(std::time::Duration::from_millis(10));

        app.select_next();

        // アクティビティタイムスタンプが更新されているべき
        assert!(app.last_activity > initial_activity);
    }

    #[test]
    fn test_navigation_empty_list() {
        let mut app = create_test_app();

        // Should not panic with empty list
        app.select_next();
        assert_eq!(app.selected_index, 0);

        app.select_previous();
        assert_eq!(app.selected_index, 0);
    }

    #[test]
    fn test_navigation_single_item() {
        let mut app = create_test_app();
        app.containers = vec![create_test_container("1", "test1")];
        app.selected_index = 0;

        // Should stay at 0 for single item
        app.select_next();
        assert_eq!(app.selected_index, 0);

        app.select_previous();
        assert_eq!(app.selected_index, 0);
    }

    #[test]
    fn test_navigation_multiple_items() {
        let mut app = create_test_app();
        app.containers = vec![
            create_test_container("1", "test1"),
            create_test_container("2", "test2"),
            create_test_container("3", "test3"),
        ];
        app.selected_index = 0;

        // Test forward navigation
        app.select_next();
        assert_eq!(app.selected_index, 1);

        app.select_next();
        assert_eq!(app.selected_index, 2);

        // Test wrap around forward
        app.select_next();
        assert_eq!(app.selected_index, 0);

        // Test backward navigation
        app.select_previous();
        assert_eq!(app.selected_index, 2);

        app.select_previous();
        assert_eq!(app.selected_index, 1);

        app.select_previous();
        assert_eq!(app.selected_index, 0);
    }

    #[test]
    fn test_selected_container() {
        let mut app = create_test_app();

        // Empty list
        assert!(app.selected_container().is_none());

        // With containers
        let container1 = create_test_container("1", "test1");
        let container2 = create_test_container("2", "test2");
        app.containers = vec![container1.clone(), container2.clone()];

        app.selected_index = 0;
        assert_eq!(app.selected_container().unwrap().id, container1.id);

        app.selected_index = 1;
        assert_eq!(app.selected_container().unwrap().id, container2.id);

        // Out of bounds
        app.selected_index = 999;
        assert!(app.selected_container().is_none());
    }

    #[test]
    fn test_app_lifecycle() {
        let mut app = create_test_app();

        assert!(app.is_running());

        app.quit();
        assert!(!app.is_running());

        // Reset for force quit test
        app.should_quit = false;
        assert!(app.is_running());

        app.force_quit();
        assert!(!app.is_running());
        assert!(!app.running);
    }

    #[test]
    fn test_view_state_transitions() {
        let mut app = create_test_app();

        // Initial state
        assert_eq!(app.view_state, ViewState::Loading);

        // Can change states
        app.view_state = ViewState::ContainerList;
        assert_eq!(app.view_state, ViewState::ContainerList);

        let error_msg = "Test error".to_string();
        app.view_state = ViewState::Error(error_msg.clone());
        if let ViewState::Error(msg) = &app.view_state {
            assert_eq!(msg, &error_msg);
        } else {
            panic!("Expected Error state");
        }
    }
}
