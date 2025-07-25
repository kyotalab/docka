// src/domain/entities/image.rs
// Image domain entity for Docker image management
// Docker イメージ管理用イメージドメインエンティティ

use crate::error::{DockaError, DockaResult};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Docker image domain entity
/// Dockerイメージドメインエンティティ
///
/// Represents a Docker image with metadata and provides business logic
/// for image operations and validation.
///
/// メタデータを持つDockerイメージを表し、イメージ操作と検証の
/// ビジネスロジックを提供します。
///
/// # Examples
///
/// ```rust
/// # use docka::domain::entities::Image;
/// let image = Image::builder()
///     .id("sha256:abc123")
///     .repository("nginx")
///     .tag("latest")
///     .size(100_000_000)
///     .build()
///     .expect("Valid image");
///
/// assert_eq!(image.full_name_explicit(), "nginx:latest");
/// assert!(image.size_mb() > 0);
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Image {
    /// Image SHA256 identifier
    /// イメージSHA256識別子
    pub id: String,

    /// Repository name (e.g., "nginx", "postgres")
    /// リポジトリ名（例："nginx"、"postgres"）
    pub repository: String,

    /// Image tag (e.g., "latest", "1.0", "alpine")
    /// イメージタグ（例："latest"、"1.0"、"alpine"）
    pub tag: String,

    /// Image size in bytes
    /// イメージサイズ（バイト）
    pub size: u64,

    /// Image creation timestamp
    /// イメージ作成タイムスタンプ
    pub created_at: DateTime<Utc>,

    /// Image labels (metadata)
    /// イメージラベル（メタデータ）
    pub labels: HashMap<String, String>,

    /// Whether this image is being used by containers
    /// このイメージがコンテナで使用されているか
    pub in_use: bool,
}

impl Image {
    /// Create a new image builder
    /// 新しいイメージビルダーを作成
    #[must_use]
    pub fn builder() -> ImageBuilder {
        ImageBuilder::new()
    }

    /// Get the display name for UI (Docker CLI compatible, :latest omitted)
    /// UI表示用の名前を取得（Docker CLI互換、:latest省略）
    #[must_use]
    pub fn display_name(&self) -> String {
        if self.tag.is_empty() || self.tag == "latest" {
            self.repository.clone()
        } else {
            format!("{}:{}", self.repository, self.tag)
        }
    }

    /// Get the full image name with explicit tag (always includes tag)
    /// 明示的なタグ付き完全イメージ名を取得（常にタグを含む）
    #[must_use]
    pub fn full_name_explicit(&self) -> String {
        let tag = if self.tag.is_empty() {
            "latest"
        } else {
            &self.tag
        };
        format!("{}:{}", self.repository, tag)
    }

    /// Get the full image name (repository:tag) - Legacy method for backward compatibility
    /// 完全なイメージ名を取得（リポジトリ:タグ）- 後方互換性のためのレガシーメソッド
    ///
    /// **Deprecated**: Use `display_name()` for UI display or `full_name_explicit()` for explicit tag
    ///
    /// # Examples
    ///
    /// ```text
    /// let image = Image::builder()
    ///     .id("sha256:abc123")
    ///     .repository("nginx")
    ///     .tag("latest")
    ///     .build()
    ///     .expect("Valid image");
    ///
    /// // Legacy method behaves like display_name() (omits :latest)
    /// assert_eq!(image.full_name(), "nginx");  // Not "nginx:latest"!
    /// ```
    #[must_use]
    #[deprecated(
        since = "0.1.0",
        note = "Use display_name() or full_name_explicit() instead"
    )]
    pub fn full_name(&self) -> String {
        self.display_name()
    }

    /// Get image size in megabytes
    /// イメージサイズをメガバイトで取得
    #[must_use]
    pub const fn size_mb(&self) -> u64 {
        self.size / 1_000_000
    }

    /// Get image size in human-readable format
    /// 人間が読める形式でイメージサイズを取得
    #[must_use]
    #[allow(clippy::cast_precision_loss)] // u64 to f64 conversion for display purposes
    pub fn size_human(&self) -> String {
        const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
        let mut size = self.size as f64;
        let mut unit_index = 0;

        while size >= 1024.0 && unit_index < UNITS.len() - 1 {
            size /= 1024.0;
            unit_index += 1;
        }

        if unit_index == 0 {
            format!("{size:.0} {}", UNITS[unit_index])
        } else {
            format!("{size:.1} {}", UNITS[unit_index])
        }
    }

    /// Get short image ID (first 12 characters)
    /// 短縮イメージIDを取得（最初の12文字）
    #[must_use]
    pub fn short_id(&self) -> &str {
        let id_without_prefix = self.id.strip_prefix("sha256:").unwrap_or(&self.id);
        let len = id_without_prefix.len().min(12);
        &id_without_prefix[..len]
    }

    /// Check if image can be removed
    /// イメージが削除可能かチェック
    #[must_use]
    pub const fn can_remove(&self) -> bool {
        !self.in_use
    }

    /// Get image age in human-readable format
    /// 人間が読める形式でイメージの経過時間を取得
    #[must_use]
    pub fn age(&self) -> String {
        let duration = Utc::now().signed_duration_since(self.created_at);

        if duration.num_days() > 30 {
            let months = duration.num_days() / 30;
            format!("{months} months ago")
        } else if duration.num_days() > 0 {
            format!("{} days ago", duration.num_days())
        } else if duration.num_hours() > 0 {
            format!("{} hours ago", duration.num_hours())
        } else {
            "Recently".to_string()
        }
    }

    /// Get label value by key
    /// キーによるラベル値の取得
    #[must_use]
    pub fn get_label(&self, key: &str) -> Option<&String> {
        self.labels.get(key)
    }

    /// Validate image entity
    /// イメージエンティティを検証
    ///
    /// # Errors
    /// * `DockaError::InvalidInput` - When validation fails
    pub fn validate(&self) -> DockaResult<()> {
        // Validate ID format
        // IDフォーマットの検証
        if self.id.is_empty() {
            return Err(DockaError::invalid_input("Image ID cannot be empty"));
        }

        // Validate repository name
        // リポジトリ名の検証
        if self.repository.is_empty() {
            return Err(DockaError::invalid_input("Repository name cannot be empty"));
        }

        if self.repository.len() > 255 {
            return Err(DockaError::invalid_input(
                "Repository name too long (max 255 characters)",
            ));
        }

        // Validate tag
        // タグの検証
        if self.tag.len() > 128 {
            return Err(DockaError::invalid_input(
                "Image tag too long (max 128 characters)",
            ));
        }

        // Validate creation time
        // 作成時間の検証
        if self.created_at > Utc::now() {
            return Err(DockaError::invalid_input(
                "Image creation time cannot be in the future",
            ));
        }

        Ok(())
    }
}

/// Builder for creating Image instances with validation
/// 検証付きでImageインスタンスを作成するビルダー
#[derive(Debug, Default)]
pub struct ImageBuilder {
    id: Option<String>,
    repository: Option<String>,
    tag: Option<String>,
    size: Option<u64>,
    created_at: Option<DateTime<Utc>>,
    labels: HashMap<String, String>,
    in_use: bool,
}

impl ImageBuilder {
    /// Create a new image builder
    /// 新しいイメージビルダーを作成
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Set image ID
    /// イメージIDを設定
    #[must_use]
    pub fn id(mut self, id: impl Into<String>) -> Self {
        self.id = Some(id.into());
        self
    }

    /// Set repository name
    /// リポジトリ名を設定
    #[must_use]
    pub fn repository(mut self, repository: impl Into<String>) -> Self {
        self.repository = Some(repository.into());
        self
    }

    /// Set image tag
    /// イメージタグを設定
    #[must_use]
    pub fn tag(mut self, tag: impl Into<String>) -> Self {
        self.tag = Some(tag.into());
        self
    }

    /// Set image size in bytes
    /// イメージサイズをバイトで設定
    #[must_use]
    pub const fn size(mut self, size: u64) -> Self {
        self.size = Some(size);
        self
    }

    /// Set creation timestamp
    /// 作成タイムスタンプを設定
    #[must_use]
    pub const fn created_at(mut self, created_at: DateTime<Utc>) -> Self {
        self.created_at = Some(created_at);
        self
    }

    /// Add a label
    /// ラベルを追加
    #[must_use]
    pub fn label(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.labels.insert(key.into(), value.into());
        self
    }

    /// Set multiple labels
    /// 複数のラベルを設定
    #[must_use]
    pub fn labels(mut self, labels: HashMap<String, String>) -> Self {
        self.labels = labels;
        self
    }

    /// Set whether image is in use
    /// イメージが使用中かを設定
    #[must_use]
    pub const fn in_use(mut self, in_use: bool) -> Self {
        self.in_use = in_use;
        self
    }

    /// Build the image with validation
    /// 検証付きでイメージを構築
    ///
    /// # Errors
    /// * `DockaError::InvalidInput` - When required fields are missing or validation fails
    pub fn build(self) -> DockaResult<Image> {
        // Validate required fields
        // 必須フィールドの検証
        let id = self
            .id
            .ok_or_else(|| DockaError::invalid_input("Image ID is required"))?;

        let repository = self
            .repository
            .ok_or_else(|| DockaError::invalid_input("Repository name is required"))?;

        let tag = self.tag.unwrap_or_else(|| "latest".to_string());
        let size = self.size.unwrap_or(0);
        let created_at = self.created_at.unwrap_or_else(Utc::now);

        // Create image instance
        // イメージインスタンスを作成
        let image = Image {
            id,
            repository,
            tag,
            size,
            created_at,
            labels: self.labels,
            in_use: self.in_use,
        };

        // Validate the complete image
        // 完全なイメージを検証
        image.validate()?;

        Ok(image)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_image() -> Image {
        Image::builder()
            .id("sha256:abc123def456")
            .repository("nginx")
            .tag("latest")
            .size(50_000_000)
            .created_at(Utc::now())
            .label("maintainer", "nginx team")
            .in_use(true)
            .build()
            .expect("Valid image")
    }

    #[test]
    fn test_image_builder_success() {
        // Test successful image creation
        // 成功したイメージ作成のテスト
        let image = create_test_image();

        assert!(image.id.starts_with("sha256:"));
        assert_eq!(image.repository, "nginx");
        assert_eq!(image.tag, "latest");
        assert_eq!(image.size, 50_000_000);
        assert_eq!(
            image.get_label("maintainer"),
            Some(&"nginx team".to_string())
        );
        assert!(image.in_use);
    }

    #[test]
    fn test_image_builder_defaults() {
        // Test builder with default values
        // デフォルト値でのビルダーテスト
        let image = Image::builder()
            .id("sha256:test123")
            .repository("test-repo")
            .build()
            .unwrap();

        assert_eq!(image.tag, "latest"); // Default tag
        assert_eq!(image.size, 0); // Default size
        assert!(!image.in_use); // Default not in use
    }

    #[test]
    fn test_image_display_name() {
        // Test display name formatting (Docker CLI compatible)
        // 表示名フォーマットのテスト（Docker CLI互換）

        let latest_image = Image::builder()
            .id("test")
            .repository("nginx")
            .tag("latest")
            .build()
            .unwrap();
        assert_eq!(latest_image.display_name(), "nginx"); // :latest omitted

        let tagged_image = Image::builder()
            .id("test")
            .repository("nginx")
            .tag("1.21")
            .build()
            .unwrap();
        assert_eq!(tagged_image.display_name(), "nginx:1.21");

        let empty_tag_image = Image::builder()
            .id("test")
            .repository("nginx")
            .tag("")
            .build()
            .unwrap();
        assert_eq!(empty_tag_image.display_name(), "nginx");
    }

    #[test]
    fn test_image_full_name_explicit() {
        // Test explicit full name formatting (always shows tag)
        // 明示的完全名フォーマットのテスト（常にタグ表示）

        let latest_image = Image::builder()
            .id("test")
            .repository("nginx")
            .tag("latest")
            .build()
            .unwrap();
        assert_eq!(latest_image.full_name_explicit(), "nginx:latest");

        let tagged_image = Image::builder()
            .id("test")
            .repository("nginx")
            .tag("1.21")
            .build()
            .unwrap();
        assert_eq!(tagged_image.full_name_explicit(), "nginx:1.21");

        let empty_tag_image = Image::builder()
            .id("test")
            .repository("nginx")
            .tag("")
            .build()
            .unwrap();
        assert_eq!(empty_tag_image.full_name_explicit(), "nginx:latest"); // Empty tag becomes latest
    }

    #[test]
    #[allow(deprecated)]
    fn test_image_full_name_legacy() {
        // Test legacy full_name method (for backward compatibility)
        // レガシーfull_nameメソッドのテスト（後方互換性用）

        let latest_image = Image::builder()
            .id("test")
            .repository("nginx")
            .tag("latest")
            .build()
            .unwrap();
        assert_eq!(latest_image.full_name(), "nginx"); // Should behave like display_name()

        let tagged_image = Image::builder()
            .id("test")
            .repository("nginx")
            .tag("1.21")
            .build()
            .unwrap();
        assert_eq!(tagged_image.full_name(), "nginx:1.21");
    }

    #[test]
    fn test_image_size_formatting() {
        // Test size formatting methods
        // サイズフォーマットメソッドのテスト

        let small_image = Image::builder()
            .id("test")
            .repository("alpine")
            .size(5_000_000) // 5MB
            .build()
            .unwrap();
        assert_eq!(small_image.size_mb(), 5);
        assert_eq!(small_image.size_human(), "4.8 MB");

        let large_image = Image::builder()
            .id("test")
            .repository("ubuntu")
            .size(2_000_000_000) // 2GB
            .build()
            .unwrap();
        assert_eq!(large_image.size_mb(), 2000);
        assert_eq!(large_image.size_human(), "1.9 GB");
    }

    #[test]
    fn test_image_short_id() {
        // Test short ID generation
        // 短縮ID生成のテスト
        let image = Image::builder()
            .id("sha256:abcdef123456789")
            .repository("test")
            .build()
            .unwrap();
        assert_eq!(image.short_id(), "abcdef123456");

        let image_without_prefix = Image::builder()
            .id("abcdef123456789")
            .repository("test")
            .build()
            .unwrap();
        assert_eq!(image_without_prefix.short_id(), "abcdef123456");
    }

    #[test]
    fn test_image_validation() {
        // Test image validation rules
        // イメージ検証ルールのテスト

        // Valid image
        // 有効なイメージ
        let valid_image = create_test_image();
        assert!(valid_image.validate().is_ok());

        // Image with empty ID
        // 空のIDのイメージ
        let result = Image::builder().id("").repository("nginx").build();
        assert!(result.is_err());

        // Image with empty repository
        // 空のリポジトリのイメージ
        let result = Image::builder().id("test").repository("").build();
        assert!(result.is_err());

        // Image with too long repository name
        // 長すぎるリポジトリ名のイメージ
        let result = Image::builder()
            .id("test")
            .repository("a".repeat(256))
            .build();
        assert!(result.is_err());
    }

    #[test]
    fn test_serialization() {
        // Test serde serialization/deserialization
        // serdeシリアライゼーション/デシリアライゼーションのテスト
        let original = create_test_image();

        let json = serde_json::to_string(&original).expect("Serialization should work");
        let deserialized: Image = serde_json::from_str(&json).expect("Deserialization should work");

        assert_eq!(original, deserialized);
    }
}
