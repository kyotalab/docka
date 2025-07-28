// src/ui/app.rs
// Basic App structure implementation for TUI application
// TUIアプリケーション用基本App構造実装

use crate::domain::entities::Container;
use crate::domain::repositories::DockerRepository;
use crate::error::DockaResult;
use std::sync::Arc;

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
            self.selected_index = self.selected_index.saturating_sub(1);
        }
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
    pub const fn quit(&mut self) {
        self.should_quit = true;
    }

    /// Force application stop
    /// アプリケーション強制停止
    ///
    /// This immediately stops the application by setting running to false.
    /// これはrunningをfalseに設定してアプリケーションを即座に停止します。
    pub const fn force_quit(&mut self) {
        self.running = false;
        self.should_quit = true;
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
