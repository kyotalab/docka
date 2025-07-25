// src/domain/value_objects/container_id.rs
// Strong-typed container identifier for type safety
// 型安全性のためのstrong-typedコンテナ識別子

use crate::error::{DockaError, DockaResult};
use serde::{Deserialize, Serialize};
use std::fmt::{self, Display};

/// Strong-typed container identifier ensuring type safety
/// 型安全性を保証するstrong-typedコンテナ識別子
///
/// This type prevents mixing up container IDs with other string values
/// and provides validation to ensure the ID meets Docker's requirements.
///
/// この型はコンテナIDを他の文字列値と混同することを防ぎ、
/// `IDがDockerの要件を満たすことを保証する検証を提供します`。
///
/// # Examples
///
/// ```rust
/// use docka::domain::value_objects::ContainerId;
///
/// // Valid container ID creation
/// // 有効なコンテナID作成
/// let id = ContainerId::new("a1b2c3d4e5f6").expect("Valid ID");
/// assert_eq!(id.as_str(), "a1b2c3d4e5f6");
///
/// // Invalid ID handling
/// // 無効なID処理
/// let result = ContainerId::new("");
/// assert!(result.is_err());
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ContainerId(String);

impl ContainerId {
    /// Create a new `ContainerId` with validation
    /// `検証付きで新しいContainerIdを作成`
    ///
    /// # Arguments
    /// * `id` - The container ID string to validate and wrap
    ///
    /// # Returns
    /// * `Ok(ContainerId)` - Valid container ID
    /// * `Err(DockaError)` - Invalid container ID with reason
    ///
    /// # Errors
    /// * `DockaError::InvalidInput` - When ID is empty, too long, or contains invalid characters
    ///
    /// # Docker ID Requirements
    /// - Must not be empty
    /// - Must be between 1 and 64 characters
    /// - Must contain only alphanumeric characters and allowed symbols
    pub fn new(id: impl Into<String>) -> DockaResult<Self> {
        let id = id.into();

        // Validate ID is not empty
        // IDが空でないことを検証
        if id.is_empty() {
            return Err(DockaError::invalid_input("Container ID cannot be empty"));
        }

        // Validate ID length (Docker container IDs are max 64 characters)
        // ID長の検証（DockerコンテナIDは最大64文字）
        if id.len() > 64 {
            return Err(DockaError::invalid_input(format!(
                "Container ID too long: {} characters (max 64)",
                id.len()
            )));
        }

        // Validate ID contains only valid characters
        // IDが有効な文字のみを含むことを検証
        if !id
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
        {
            return Err(DockaError::invalid_input(
                "Container ID contains invalid characters. Only alphanumeric, '-', and '_' are allowed",
            ));
        }

        Ok(Self(id))
    }

    /// Create `ContainerId` from trusted source without validation
    /// `検証なしで信頼できるソースからContainerIdを作成`
    ///
    /// # Safety
    /// This method should only be used when the ID is known to be valid,
    /// such as when deserializing from Docker API responses.
    ///
    /// この方法はDocker APIレスポンスからデシリアライズする場合など、
    /// IDが有効であることが分かっている場合にのみ使用すべきです。
    ///
    /// # Arguments
    /// * `id` - Pre-validated container ID
    pub fn from_trusted(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    /// Get the container ID as a string slice
    /// コンテナIDを文字列スライスとして取得
    #[must_use]
    #[allow(clippy::missing_const_for_fn)] // Rust language limitation: String deref is not const
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Get the short version of container ID (first 12 characters)
    /// コンテナIDの短縮版を取得（最初の12文字）
    ///
    /// This is commonly used in Docker CLI output for readability.
    /// これはDocker CLI出力で可読性のために一般的に使用されます。
    #[must_use]
    pub fn short(&self) -> &str {
        let len = self.0.len().min(12);
        &self.0[..len]
    }

    /// Check if this ID matches another ID or short ID
    /// このIDが他のIDまたは短縮IDと一致するかチェック
    ///
    /// This allows matching both full IDs and commonly used short IDs.
    /// これにより完全なIDと一般的に使用される短縮IDの両方にマッチできます。
    #[must_use]
    pub fn matches(&self, other: &str) -> bool {
        self.0 == other || self.short() == other || self.0.starts_with(other)
    }
}

impl Display for ContainerId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl AsRef<str> for ContainerId {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl From<ContainerId> for String {
    fn from(id: ContainerId) -> Self {
        id.0
    }
}

// Enable comparison with string types for convenience
// 便利性のため文字列型との比較を有効化
impl PartialEq<str> for ContainerId {
    fn eq(&self, other: &str) -> bool {
        self.0 == other
    }
}

impl PartialEq<&str> for ContainerId {
    fn eq(&self, other: &&str) -> bool {
        self.0 == *other
    }
}

impl PartialEq<&&str> for ContainerId {
    fn eq(&self, other: &&&str) -> bool {
        self.0 == **other
    }
}

impl PartialEq<String> for ContainerId {
    fn eq(&self, other: &String) -> bool {
        &self.0 == other
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_container_id_creation() {
        // Test various valid container ID formats
        // 様々な有効なコンテナIDフォーマットのテスト
        let valid_ids = vec![
            "a1b2c3d4e5f6",
            "container-name",
            "web_app_1",
            "a1b2c3d4e5f67890abcdef1234567890abcdef1234567890abcdef1234567890",
        ];

        for id_str in valid_ids {
            let result = ContainerId::new(id_str);
            assert!(result.is_ok(), "ID '{id_str}' should be valid");

            let id = result.unwrap();
            assert_eq!(id.as_str(), id_str);
            assert_eq!(id.to_string(), id_str);
        }
    }

    #[test]
    fn test_invalid_container_id_creation() {
        // Test invalid container IDs
        // 無効なコンテナIDのテスト
        let binding = "a".repeat(65);
        let invalid_ids = vec![
            ("", "Container ID cannot be empty"),
            (binding.as_str(), "Container ID too long"),
            ("container with spaces", "invalid characters"),
            ("container@domain.com", "invalid characters"),
            ("container/path", "invalid characters"),
        ];

        for (id_str, expected_error_part) in invalid_ids {
            let result = ContainerId::new(id_str);
            assert!(result.is_err(), "ID '{id_str}' should be invalid");

            let error = result.unwrap_err();
            if let DockaError::InvalidInput { message } = error {
                assert!(
                    message.contains(expected_error_part),
                    "Error message '{message}' should contain '{expected_error_part}'"
                );
            } else {
                panic!("Expected InvalidInput error, got {error:?}");
            }
        }
    }

    #[test]
    fn test_trusted_creation() {
        // Test creation from trusted source
        // 信頼できるソースからの作成テスト
        let id = ContainerId::from_trusted("any-string-even-invalid!");
        assert_eq!(id.as_str(), "any-string-even-invalid!");
    }

    #[test]
    fn test_short_id() {
        // Test short ID generation
        // 短縮ID生成のテスト
        let long_id = ContainerId::new("a1b2c3d4e5f67890abcdef").unwrap();
        assert_eq!(long_id.short(), "a1b2c3d4e5f6");

        let short_id = ContainerId::new("abc123").unwrap();
        assert_eq!(short_id.short(), "abc123");
    }

    #[test]
    fn test_id_matching() {
        // Test ID matching with various formats
        // 様々なフォーマットでのIDマッチングテスト
        let id = ContainerId::new("a1b2c3d4e5f67890abcdef").unwrap();

        // Full ID match
        // 完全IDマッチ
        assert!(id.matches("a1b2c3d4e5f67890abcdef"));

        // Short ID match
        // 短縮IDマッチ
        assert!(id.matches("a1b2c3d4e5f6"));

        // Partial match
        // 部分マッチ
        assert!(id.matches("a1b2"));

        // No match
        // マッチなし
        assert!(!id.matches("xyz789"));
        assert!(!id.matches("b1c2d3"));
    }

    #[test]
    fn test_string_comparison() {
        // Test comparison with string types
        // 文字列型との比較テスト
        let id = ContainerId::new("test-container").unwrap();

        // Direct string literal comparison
        // 直接的な文字列リテラル比較
        assert_eq!(id, "test-container");

        // Reference to string literal
        // 文字列リテラルへの参照
        assert_eq!(id, &"test-container");

        // String type comparison
        // String型との比較
        assert_eq!(id, String::from("test-container"));

        // Negative comparison
        // 否定比較
        assert_ne!(id, "other-container");

        // Test with explicit string slice variable
        // 明示的な文字列スライス変数でのテスト
        let test_str: &str = "test-container";
        assert_eq!(id, test_str);
        assert_eq!(id, &test_str);
    }

    #[test]
    fn test_serialization() {
        // Test serde serialization/deserialization
        // serdeシリアライゼーション/デシリアライゼーションのテスト
        let original = ContainerId::new("test-id").unwrap();

        let json = serde_json::to_string(&original).expect("Serialization should work");
        let deserialized: ContainerId =
            serde_json::from_str(&json).expect("Deserialization should work");

        assert_eq!(original, deserialized);
    }

    #[test]
    fn test_hash_and_eq() {
        // Test that equal IDs have equal hashes (required for HashMap)
        // 等しいIDが等しいハッシュを持つことをテスト（HashMap用）
        use std::collections::HashMap;

        let id1 = ContainerId::new("same-id").unwrap();
        let id2 = ContainerId::new("same-id").unwrap();
        let id3 = ContainerId::new("different-id").unwrap();

        assert_eq!(id1, id2);
        assert_ne!(id1, id3);

        // Test in HashMap
        // HashMapでのテスト
        let mut map = HashMap::new();
        map.insert(id1.clone(), "value1");
        map.insert(id2, "value2"); // Should overwrite

        assert_eq!(map.len(), 1);
        assert_eq!(map.get(&id1), Some(&"value2"));
    }
}
