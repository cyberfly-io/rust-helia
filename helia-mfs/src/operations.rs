//! MFS operations - helper utilities for path navigation and operations
//!
//! This module provides internal utilities used by the MFS implementation
//! to navigate directory structures and perform file operations.

use crate::MfsError;

/// Helper to validate and normalize paths
pub fn normalize_path(path: &str) -> Result<String, MfsError> {
    let path = path.trim();
    
    if path.is_empty() {
        return Err(MfsError::InvalidPath("Empty path".to_string()));
    }

    if !path.starts_with('/') {
        return Err(MfsError::InvalidPath(
            "Path must be absolute (start with /)".to_string(),
        ));
    }

    // Normalize multiple slashes and trailing slashes
    let mut normalized = String::from("/");
    let segments: Vec<&str> = path
        .trim_start_matches('/')
        .trim_end_matches('/')
        .split('/')
        .filter(|s| !s.is_empty() && *s != ".")
        .collect();

    // Check for invalid segments
    for segment in &segments {
        if *segment == ".." {
            return Err(MfsError::InvalidPath(
                "Parent directory references (..) not supported".to_string(),
            ));
        }
        if segment.contains('\0') {
            return Err(MfsError::InvalidPath(
                "Path segments cannot contain null bytes".to_string(),
            ));
        }
    }

    if !segments.is_empty() {
        normalized.push_str(&segments.join("/"));
    }

    Ok(normalized)
}

/// Split a path into parent and name
pub fn split_path(path: &str) -> Result<(String, String), MfsError> {
    let normalized = normalize_path(path)?;
    
    if normalized == "/" {
        return Err(MfsError::InvalidPath("Root has no parent".to_string()));
    }

    let last_slash = normalized.rfind('/').unwrap();
    let parent = if last_slash == 0 {
        "/".to_string()
    } else {
        normalized[..last_slash].to_string()
    };
    
    let name = normalized[last_slash + 1..].to_string();
    
    Ok((parent, name))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_root() {
        assert_eq!(normalize_path("/").unwrap(), "/");
    }

    #[test]
    fn test_normalize_simple() {
        assert_eq!(normalize_path("/foo").unwrap(), "/foo");
    }

    #[test]
    fn test_normalize_nested() {
        assert_eq!(normalize_path("/foo/bar/baz").unwrap(), "/foo/bar/baz");
    }

    #[test]
    fn test_normalize_trailing_slash() {
        assert_eq!(normalize_path("/foo/bar/").unwrap(), "/foo/bar");
    }

    #[test]
    fn test_normalize_double_slash() {
        assert_eq!(normalize_path("/foo//bar").unwrap(), "/foo/bar");
    }

    #[test]
    fn test_normalize_dot() {
        assert_eq!(normalize_path("/foo/./bar").unwrap(), "/foo/bar");
    }

    #[test]
    fn test_normalize_double_dot_fails() {
        assert!(normalize_path("/foo/../bar").is_err());
    }

    #[test]
    fn test_normalize_relative_fails() {
        assert!(normalize_path("foo/bar").is_err());
    }

    #[test]
    fn test_split_simple() {
        let (parent, name) = split_path("/foo").unwrap();
        assert_eq!(parent, "/");
        assert_eq!(name, "foo");
    }

    #[test]
    fn test_split_nested() {
        let (parent, name) = split_path("/foo/bar/baz").unwrap();
        assert_eq!(parent, "/foo/bar");
        assert_eq!(name, "baz");
    }

    #[test]
    fn test_split_root_fails() {
        assert!(split_path("/").is_err());
    }
}
