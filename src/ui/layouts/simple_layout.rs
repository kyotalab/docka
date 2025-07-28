//! Simple Layout Implementation for Docka TUI
//!
//! シンプルレイアウト実装 - Docka TUI用
//!
//! このモジュールは、TUIアプリケーションの基本的な3分割レイアウトを提供します。
//! This module provides basic 3-section layout for TUI applications.

use ratatui::layout::{Constraint, Direction, Layout, Rect};

/// Layout areas structure for organizing terminal space
/// ターミナルスペースを整理するためのレイアウトエリア構造体
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LayoutAreas {
    /// Main content area (container list)
    /// メインコンテンツエリア（コンテナリスト）
    pub main: Rect,

    /// Status bar area (current state display)
    /// ステータスバーエリア（現在の状態表示）
    pub status: Rect,

    /// Help line area (key bindings)
    /// ヘルプラインエリア（キーバインド）
    pub help: Rect,
}

/// Simple layout manager for 3-section vertical layout
/// 3セクション縦分割レイアウトのシンプルレイアウトマネージャー
pub struct SimpleLayout;

impl SimpleLayout {
    /// Calculate standard layout areas with 3 vertical sections
    /// 3つの縦セクションで標準レイアウトエリアを計算
    ///
    /// # Layout Structure / レイアウト構造
    /// - Main: Minimum 10 rows, expandable / 最小10行、拡張可能
    /// - Status: Fixed 3 rows / 固定3行
    /// - Help: Fixed 1 row / 固定1行
    ///
    /// # Arguments / 引数
    /// * `area` - Total terminal area / 全ターミナルエリア
    ///
    /// # Returns / 戻り値
    /// * `LayoutAreas` - Calculated layout sections / 計算されたレイアウトセクション
    #[must_use]
    pub fn calculate(area: Rect) -> LayoutAreas {
        // Create vertical layout with 3 sections
        // 3セクションの縦レイアウトを作成
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(10),   // Main area: minimum 10 rows / メインエリア：最小10行
                Constraint::Length(3), // Status area: fixed 3 rows / ステータスエリア：固定3行
                Constraint::Length(1), // Help area: fixed 1 row / ヘルプエリア：固定1行
            ])
            .split(area);

        LayoutAreas {
            main: chunks[0],
            status: chunks[1],
            help: chunks[2],
        }
    }

    /// Calculate responsive layout for small terminal sizes
    /// 小さなターミナルサイズ用のレスポンシブレイアウトを計算
    ///
    /// # Arguments / 引数
    /// * `area` - Total terminal area / 全ターミナルエリア
    ///
    /// # Returns / 戻り値
    /// * `LayoutAreas` - Responsive layout sections / レスポンシブレイアウトセクション
    ///
    /// # Responsive Behavior / レスポンシブ動作
    /// - Height < 10: Hide help area / 高さ < 10: ヘルプエリア非表示
    /// - Height < 7: Minimize status area / 高さ < 7: ステータスエリア最小化
    #[must_use]
    pub fn calculate_responsive(area: Rect) -> LayoutAreas {
        match area.height {
            // Very small terminal: only main area
            // 非常に小さなターミナル：メインエリアのみ
            0..=6 => {
                let chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([
                        Constraint::Min(1),    // Main takes all space / メインが全スペースを使用
                        Constraint::Length(2), // Minimal status / 最小ステータス
                        Constraint::Length(0), // No help area / ヘルプエリアなし
                    ])
                    .split(area);

                LayoutAreas {
                    main: chunks[0],
                    status: chunks[1],
                    help: Rect::default(), // Empty help area / 空のヘルプエリア
                }
            }

            // Small terminal: no help area
            // 小さなターミナル：ヘルプエリアなし
            7..=9 => {
                let chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([
                        Constraint::Min(4),    // Main area minimum / メインエリア最小
                        Constraint::Length(3), // Standard status / 標準ステータス
                        Constraint::Length(0), // No help area / ヘルプエリアなし
                    ])
                    .split(area);

                LayoutAreas {
                    main: chunks[0],
                    status: chunks[1],
                    help: Rect::default(), // Empty help area / 空のヘルプエリア
                }
            }

            // Standard terminal: full layout
            // 標準ターミナル：フルレイアウト
            _ => Self::calculate(area),
        }
    }

    /// Check if help area should be visible based on terminal size
    /// ターミナルサイズに基づいてヘルプエリアが表示されるべきかチェック
    ///
    /// # Arguments / 引数
    /// * `area` - Terminal area to check / チェックするターミナルエリア
    ///
    /// # Returns / 戻り値
    /// * `bool` - True if help should be visible / ヘルプが表示されるべき場合true
    #[must_use]
    pub const fn should_show_help(area: Rect) -> bool {
        area.height >= 10
    }

    /// Get minimum required terminal size for basic functionality
    /// 基本機能に必要な最小ターミナルサイズを取得
    ///
    /// # Returns / 戻り値
    /// * `(u16, u16)` - (width, height) minimum requirements / (幅, 高さ)最小要件
    #[must_use]
    pub const fn minimum_size() -> (u16, u16) {
        (40, 7) // Minimum 40 cols x 7 rows / 最小40列 x 7行
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_standard_layout() {
        // Test standard layout calculation
        // 標準レイアウト計算をテスト
        let area = Rect::new(0, 0, 80, 24);
        let layout = SimpleLayout::calculate(area);

        // Main area should have at least 10 rows
        // メインエリアは最低10行を持つべき
        assert!(layout.main.height >= 10);

        // Status area should have exactly 3 rows
        // ステータスエリアは正確に3行を持つべき
        assert_eq!(layout.status.height, 3);

        // Help area should have exactly 1 row
        // ヘルプエリアは正確に1行を持つべき
        assert_eq!(layout.help.height, 1);

        // Total height should match
        // 合計高さは一致すべき
        assert_eq!(
            layout.main.height + layout.status.height + layout.help.height,
            area.height
        );
    }

    #[test]
    fn test_calculate_responsive_large() {
        // Test responsive layout with large terminal
        // 大きなターミナルでのレスポンシブレイアウトをテスト
        let area = Rect::new(0, 0, 100, 30);
        let layout = SimpleLayout::calculate_responsive(area);

        // Should be same as standard layout for large terminals
        // 大きなターミナルでは標準レイアウトと同じであるべき
        let standard = SimpleLayout::calculate(area);
        assert_eq!(layout, standard);
    }

    #[test]
    fn test_calculate_responsive_medium() {
        // Test responsive layout with medium terminal (no help)
        // 中サイズターミナルでのレスポンシブレイアウトをテスト（ヘルプなし）
        let area = Rect::new(0, 0, 80, 8);
        let layout = SimpleLayout::calculate_responsive(area);

        // Help area should be empty
        // ヘルプエリアは空であるべき
        assert_eq!(layout.help.height, 0);

        // Status should still be 3 rows
        // ステータスは依然として3行であるべき
        assert_eq!(layout.status.height, 3);

        // Main should take remaining space
        // メインは残りのスペースを取るべき
        assert!(layout.main.height >= 4);
    }

    #[test]
    fn test_calculate_responsive_small() {
        // Test responsive layout with very small terminal
        // 非常に小さなターミナルでのレスポンシブレイアウトをテスト
        let area = Rect::new(0, 0, 40, 6);
        let layout = SimpleLayout::calculate_responsive(area);

        // Help area should be empty
        // ヘルプエリアは空であるべき
        assert_eq!(layout.help.height, 0);

        // Status should be minimal (2 rows)
        // ステータスは最小（2行）であるべき
        assert_eq!(layout.status.height, 2);

        // Main should take most space
        // メインはほとんどのスペースを取るべき
        assert!(layout.main.height >= 1);
    }

    #[test]
    fn test_should_show_help() {
        // Test help visibility logic
        // ヘルプ表示ロジックをテスト
        assert!(SimpleLayout::should_show_help(Rect::new(0, 0, 80, 24)));
        assert!(SimpleLayout::should_show_help(Rect::new(0, 0, 80, 10)));
        assert!(!SimpleLayout::should_show_help(Rect::new(0, 0, 80, 9)));
        assert!(!SimpleLayout::should_show_help(Rect::new(0, 0, 80, 5)));
    }

    #[test]
    fn test_minimum_size() {
        // Test minimum size requirements
        // 最小サイズ要件をテスト
        let (min_width, min_height) = SimpleLayout::minimum_size();
        assert_eq!(min_width, 40);
        assert_eq!(min_height, 7);
    }

    #[test]
    fn test_layout_areas_equality() {
        // Test LayoutAreas equality implementation
        // LayoutAreasの等価性実装をテスト
        let area1 = LayoutAreas {
            main: Rect::new(0, 0, 80, 20),
            status: Rect::new(0, 20, 80, 3),
            help: Rect::new(0, 23, 80, 1),
        };

        let area2 = LayoutAreas {
            main: Rect::new(0, 0, 80, 20),
            status: Rect::new(0, 20, 80, 3),
            help: Rect::new(0, 23, 80, 1),
        };

        assert_eq!(area1, area2);
    }
}
