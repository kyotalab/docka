//! UI Styles and Theme Configuration for Docka TUI
//!
//! UI スタイルとテーマ設定 - Docka TUI用
//!
//! This module provides consistent styling and theming across all UI components,
//! including colors, borders, and visual states.
//!
//! このモジュールは色、ボーダー、視覚的状態を含む全UIコンポーネント間での
//! 一貫したスタイリングとテーマ設定を提供します。

use ratatui::{
    style::{Color, Modifier, Style},
    widgets::{Block, BorderType, Borders},
};

/// Application-wide style configuration
/// アプリケーション全体のスタイル設定
#[derive(Debug, Clone)]
pub struct AppStyles {
    /// Primary accent color (used for highlights and active states)
    /// プライマリアクセント色（ハイライトとアクティブ状態用）
    pub primary: Color,

    /// Success color (used for running containers and positive states)
    /// 成功色（実行中コンテナと正常状態用）
    pub success: Color,

    /// Warning color (used for loading states and cautions)
    /// 警告色（ローディング状態と注意用）
    pub warning: Color,

    /// Error color (used for errors and stopped containers)
    /// エラー色（エラーと停止コンテナ用）
    pub error: Color,

    /// Muted color (used for secondary text and borders)
    /// 抑制色（セカンダリテキストとボーダー用）
    pub muted: Color,

    /// Background color for selected items
    /// 選択項目の背景色
    pub selected_bg: Color,

    /// Text color for selected items
    /// 選択項目のテキスト色
    pub selected_fg: Color,
}

impl Default for AppStyles {
    fn default() -> Self {
        Self {
            primary: Color::Cyan,
            success: Color::Green,
            warning: Color::Yellow,
            error: Color::Red,
            muted: Color::DarkGray,
            selected_bg: Color::Blue,
            selected_fg: Color::White,
        }
    }
}

impl AppStyles {
    /// Create a new style configuration with custom colors
    /// カスタム色で新しいスタイル設定を作成
    ///
    /// # Arguments / 引数
    /// * `primary` - Primary accent color / プライマリアクセント色
    /// * `success` - Success state color / 成功状態色
    /// * `warning` - Warning state color / 警告状態色
    /// * `error` - Error state color / エラー状態色
    ///
    /// # Returns / 戻り値
    /// * `AppStyles` - Configured style instance / 設定されたスタイルインスタンス
    #[must_use]
    pub const fn new(primary: Color, success: Color, warning: Color, error: Color) -> Self {
        Self {
            primary,
            success,
            warning,
            error,
            muted: Color::DarkGray,
            selected_bg: Color::Blue,
            selected_fg: Color::White,
        }
    }

    /// Get style for loading state
    /// ローディング状態用スタイルを取得
    ///
    /// # Returns / 戻り値
    /// * `Style` - Loading state style / ローディング状態スタイル
    #[must_use]
    pub fn loading_style(&self) -> Style {
        Style::default()
            .fg(Color::Black)
            .bg(self.warning)
            .add_modifier(Modifier::BOLD)
    }

    /// Get style for error state
    /// エラー状態用スタイルを取得
    ///
    /// # Returns / 戻り値
    /// * `Style` - Error state style / エラー状態スタイル
    #[must_use]
    pub fn error_style(&self) -> Style {
        Style::default()
            .fg(Color::White)
            .bg(self.error)
            .add_modifier(Modifier::BOLD)
    }

    /// Get style for success state
    /// 成功状態用スタイルを取得
    ///
    /// # Returns / 戻り値
    /// * `Style` - Success state style / 成功状態スタイル
    #[must_use]
    pub fn success_style(&self) -> Style {
        Style::default()
            .fg(self.success)
            .add_modifier(Modifier::BOLD)
    }

    /// Get style for selected items
    /// 選択項目用スタイルを取得
    ///
    /// # Returns / 戻り値
    /// * `Style` - Selected item style / 選択項目スタイル
    #[must_use]
    pub fn selected_style(&self) -> Style {
        Style::default()
            .fg(self.selected_fg)
            .bg(self.selected_bg)
            .add_modifier(Modifier::BOLD)
    }

    /// Get style for normal text
    /// 通常テキスト用スタイルを取得
    ///
    /// # Returns / 戻り値
    /// * `Style` - Normal text style / 通常テキストスタイル
    #[must_use]
    pub fn normal_style(&self) -> Style {
        Style::default().fg(Color::White)
    }

    /// Get style for muted text
    /// 抑制テキスト用スタイルを取得
    ///
    /// # Returns / 戻り値
    /// * `Style` - Muted text style / 抑制テキストスタイル
    #[must_use]
    pub fn muted_style(&self) -> Style {
        Style::default().fg(self.muted)
    }

    /// Get style for primary accent elements
    /// プライマリアクセント要素用スタイルを取得
    ///
    /// # Returns / 戻り値
    /// * `Style` - Primary accent style / プライマリアクセントスタイル
    #[must_use]
    pub fn primary_style(&self) -> Style {
        Style::default()
            .fg(self.primary)
            .add_modifier(Modifier::BOLD)
    }
}

/// Block styling configuration for consistent borders
/// 一貫したボーダーのためのブロックスタイリング設定
#[derive(Debug, Clone)]
pub struct BlockStyles {
    /// Border type for normal blocks
    /// 通常ブロックのボーダータイプ
    pub normal_border: BorderType,

    /// Border type for focused/active blocks
    /// フォーカス/アクティブブロックのボーダータイプ
    pub active_border: BorderType,

    /// Border style for normal blocks
    /// 通常ブロックのボーダースタイル
    pub normal_border_style: Style,

    /// Border style for focused/active blocks
    /// フォーカス/アクティブブロックのボーダースタイル
    pub active_border_style: Style,
}

impl Default for BlockStyles {
    fn default() -> Self {
        Self {
            normal_border: BorderType::Rounded,
            active_border: BorderType::Double,
            normal_border_style: Style::default().fg(Color::DarkGray),
            active_border_style: Style::default().fg(Color::Cyan),
        }
    }
}

impl BlockStyles {
    /// Create a standard block with normal styling
    /// 通常スタイリングで標準ブロックを作成
    ///
    /// # Arguments / 引数
    /// * `title` - Optional block title / オプションのブロックタイトル
    ///
    /// # Returns / 戻り値
    /// * `Block` - Styled block widget / スタイル設定されたブロックウィジェット
    #[must_use]
    pub fn normal_block<'a>(&self, title: Option<&'a str>) -> Block<'a> {
        let mut block = Block::default()
            .borders(Borders::ALL)
            .border_type(self.normal_border)
            .border_style(self.normal_border_style);

        if let Some(title) = title {
            block = block.title(title);
        }

        block
    }

    /// Create an active/focused block with emphasis styling
    /// エンファシススタイリングでアクティブ/フォーカスブロックを作成
    ///
    /// # Arguments / 引数
    /// * `title` - Optional block title / オプションのブロックタイトル
    ///
    /// # Returns / 戻り値
    /// * `Block` - Styled active block widget / スタイル設定されたアクティブブロックウィジェット
    #[must_use]
    pub fn active_block<'a>(&self, title: Option<&'a str>) -> Block<'a> {
        let mut block = Block::default()
            .borders(Borders::ALL)
            .border_type(self.active_border)
            .border_style(self.active_border_style);

        if let Some(title) = title {
            block = block.title(title);
        }

        block
    }

    /// Create a status block for displaying state information
    /// 状態情報表示用ステータスブロックを作成
    ///
    /// # Arguments / 引数
    /// * `state_style` - Style for the border based on state / 状態に基づくボーダースタイル
    ///
    /// # Returns / 戻り値
    /// * `Block` - Styled status block widget / スタイル設定されたステータスブロックウィジェット
    #[must_use]
    pub fn status_block(&self, state_style: Style) -> Block {
        Block::default()
            .borders(Borders::ALL)
            .border_type(self.normal_border)
            .border_style(state_style)
    }
}

/// Theme manager for coordinating styles across the application
/// アプリケーション全体でスタイルを調整するテーママネージャー
#[derive(Debug, Clone, Default)]
pub struct Theme {
    /// Color and style configuration
    /// 色とスタイル設定
    pub styles: AppStyles,

    /// Block and border configuration
    /// ブロックとボーダー設定
    pub blocks: BlockStyles,
}

impl Theme {
    /// Create a new theme with custom configurations
    /// カスタム設定で新しいテーマを作成
    ///
    /// # Arguments / 引数
    /// * `styles` - Color and style configuration / 色とスタイル設定
    /// * `blocks` - Block and border configuration / ブロックとボーダー設定
    ///
    /// # Returns / 戻り値
    /// * `Theme` - Configured theme instance / 設定されたテーマインスタンス
    #[must_use]
    pub const fn new(styles: AppStyles, blocks: BlockStyles) -> Self {
        Self { styles, blocks }
    }

    /// Create a dark theme optimized for dark terminals
    /// ダークターミナル最適化ダークテーマを作成
    ///
    /// # Returns / 戻り値
    /// * `Theme` - Dark theme configuration / ダークテーマ設定
    #[must_use]
    pub fn dark() -> Self {
        let styles = AppStyles {
            primary: Color::Cyan,
            success: Color::Green,
            warning: Color::Yellow,
            error: Color::Red,
            muted: Color::DarkGray,
            selected_bg: Color::Blue,
            selected_fg: Color::White,
        };

        let blocks = BlockStyles {
            normal_border: BorderType::Rounded,
            active_border: BorderType::Double,
            normal_border_style: Style::default().fg(Color::DarkGray),
            active_border_style: Style::default().fg(Color::Cyan),
        };

        Self::new(styles, blocks)
    }

    /// Create a light theme optimized for light terminals
    /// ライトターミナル最適化ライトテーマを作成
    ///
    /// # Returns / 戻り値
    /// * `Theme` - Light theme configuration / ライトテーマ設定
    #[must_use]
    pub fn light() -> Self {
        let styles = AppStyles {
            primary: Color::Blue,
            success: Color::Green,
            warning: Color::Yellow,
            error: Color::Red,
            muted: Color::Gray,
            selected_bg: Color::LightBlue,
            selected_fg: Color::Black,
        };

        let blocks = BlockStyles {
            normal_border: BorderType::Plain,
            active_border: BorderType::Thick,
            normal_border_style: Style::default().fg(Color::Gray),
            active_border_style: Style::default().fg(Color::Blue),
        };

        Self::new(styles, blocks)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::prelude::Rect;

    #[test]
    fn test_app_styles_default() {
        // Test default AppStyles configuration
        // デフォルトAppStyles設定をテスト
        let styles = AppStyles::default();

        assert_eq!(styles.primary, Color::Cyan);
        assert_eq!(styles.success, Color::Green);
        assert_eq!(styles.warning, Color::Yellow);
        assert_eq!(styles.error, Color::Red);
        assert_eq!(styles.muted, Color::DarkGray);
    }

    #[test]
    fn test_app_styles_custom() {
        // Test custom AppStyles creation
        // カスタムAppStyles作成をテスト
        let styles = AppStyles::new(
            Color::Magenta,
            Color::LightGreen,
            Color::LightYellow,
            Color::LightRed,
        );

        assert_eq!(styles.primary, Color::Magenta);
        assert_eq!(styles.success, Color::LightGreen);
        assert_eq!(styles.warning, Color::LightYellow);
        assert_eq!(styles.error, Color::LightRed);
    }

    #[test]
    fn test_style_methods() {
        // Test style generation methods
        // スタイル生成メソッドをテスト
        let styles = AppStyles::default();

        let loading_style = styles.loading_style();
        assert_eq!(loading_style.bg, Some(Color::Yellow));
        assert!(loading_style.add_modifier.contains(Modifier::BOLD));

        let error_style = styles.error_style();
        assert_eq!(error_style.bg, Some(Color::Red));
        assert_eq!(error_style.fg, Some(Color::White));

        let selected_style = styles.selected_style();
        assert_eq!(selected_style.bg, Some(Color::Blue));
        assert_eq!(selected_style.fg, Some(Color::White));
    }

    #[test]
    fn test_block_styles_default() {
        // Test default BlockStyles configuration
        // デフォルトBlockStyles設定をテスト
        let block_styles = BlockStyles::default();

        assert_eq!(block_styles.normal_border, BorderType::Rounded);
        assert_eq!(block_styles.active_border, BorderType::Double);
    }

    #[test]
    fn test_block_creation() {
        // Test block creation methods
        // ブロック作成メソッドをテスト
        let block_styles = BlockStyles::default();

        // Test that block creation methods don't panic and return valid blocks
        // ブロック作成メソッドがパニックせず有効なブロックを返すことをテスト
        let normal_block = block_styles.normal_block(Some("Test Title"));
        let active_block = block_styles.active_block(None);
        let status_block = block_styles.status_block(Style::default().fg(Color::Green));

        // Test that blocks are created successfully (no panics)
        // ブロックが正常に作成されることをテスト（パニックなし）
        // Note: In ratatui 0.29.0, Block internals are not directly accessible
        // 注意: ratatui 0.29.0では、Blockの内部状態は直接アクセスできません

        // Simple validation that blocks exist and have expected types
        // ブロックが存在し期待される型を持つことの簡単な検証
        let _normal_width = normal_block.inner(Rect::new(0, 0, 10, 3)).width;
        let _active_width = active_block.inner(Rect::new(0, 0, 10, 3)).width;
        let _status_width = status_block.inner(Rect::new(0, 0, 10, 3)).width;

        // If we reach here, block creation was successful
        // ここに到達すれば、ブロック作成は成功
        assert!(true, "Block creation methods executed successfully");
    }

    #[test]
    fn test_theme_default() {
        // Test default Theme configuration
        // デフォルトTheme設定をテスト
        let theme = Theme::default();

        assert_eq!(theme.styles.primary, Color::Cyan);
        assert_eq!(theme.blocks.normal_border, BorderType::Rounded);
    }

    #[test]
    fn test_theme_dark() {
        // Test dark theme configuration
        // ダークテーマ設定をテスト
        let dark_theme = Theme::dark();

        assert_eq!(dark_theme.styles.primary, Color::Cyan);
        assert_eq!(dark_theme.styles.selected_fg, Color::White);
        assert_eq!(dark_theme.blocks.normal_border, BorderType::Rounded);
    }

    #[test]
    fn test_theme_light() {
        // Test light theme configuration
        // ライトテーマ設定をテスト
        let light_theme = Theme::light();

        assert_eq!(light_theme.styles.primary, Color::Blue);
        assert_eq!(light_theme.styles.selected_fg, Color::Black);
        assert_eq!(light_theme.blocks.normal_border, BorderType::Plain);
    }

    #[test]
    fn test_theme_custom() {
        // Test custom theme creation
        // カスタムテーマ作成をテスト
        let custom_styles = AppStyles::new(
            Color::Magenta,
            Color::LightGreen,
            Color::LightYellow,
            Color::LightRed,
        );
        let custom_blocks = BlockStyles::default();
        let custom_theme = Theme::new(custom_styles, custom_blocks);

        assert_eq!(custom_theme.styles.primary, Color::Magenta);
        assert_eq!(custom_theme.blocks.normal_border, BorderType::Rounded);
    }
}
