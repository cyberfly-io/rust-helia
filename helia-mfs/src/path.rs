//! Path resolution and manipulation for MFS

use crate::MfsError;

/// Represents a parsed MFS path
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MfsPath {
    /// Path segments (empty for root "/")
    pub segments: Vec<String>,
    /// Whether this is an absolute path
    pub is_absolute: bool,
}

impl MfsPath {
    /// Parse a path string into segments
    pub fn parse(path: &str) -> Result<Self, MfsError> {
        let path = path.trim();
        
        if path.is_empty() {
            return Err(MfsError::InvalidPath("Empty path".to_string()));
        }

        let is_absolute = path.starts_with('/');
        
        if !is_absolute {
            return Err(MfsError::InvalidPath(
                "Path must be absolute (start with /)".to_string(),
            ));
        }

        // Root path
        if path == "/" {
            return Ok(Self {
                segments: vec![],
                is_absolute: true,
            });
        }

        // Parse segments
        let segments: Vec<String> = path
            .trim_start_matches('/')
            .trim_end_matches('/')
            .split('/')
            .filter(|s| !s.is_empty() && *s != ".")
            .map(|s| s.to_string())
            .collect();

        // Validate segments
        for segment in &segments {
            if segment == ".." {
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

        Ok(Self {
            segments,
            is_absolute,
        })
    }

    /// Get the parent path
    pub fn parent(&self) -> Option<Self> {
        if self.segments.is_empty() {
            return None; // Root has no parent
        }

        let mut parent_segments = self.segments.clone();
        parent_segments.pop();

        Some(Self {
            segments: parent_segments,
            is_absolute: self.is_absolute,
        })
    }

    /// Get the file/directory name (last segment)
    pub fn name(&self) -> Option<&str> {
        self.segments.last().map(|s| s.as_str())
    }

    /// Convert back to string representation
    pub fn to_string(&self) -> String {
        if self.segments.is_empty() {
            return "/".to_string();
        }

        if self.is_absolute {
            format!("/{}", self.segments.join("/"))
        } else {
            self.segments.join("/")
        }
    }

    /// Get path depth (number of segments)
    pub fn depth(&self) -> usize {
        self.segments.len()
    }

    /// Check if this is the root path
    pub fn is_root(&self) -> bool {
        self.segments.is_empty()
    }

    /// Join with another path segment
    pub fn join(&self, segment: &str) -> Result<Self, MfsError> {
        if segment.is_empty() || segment == "." {
            return Ok(self.clone());
        }

        if segment.contains('/') {
            return Err(MfsError::InvalidPath(
                "Segment cannot contain /".to_string(),
            ));
        }

        if segment == ".." {
            return self.parent().ok_or_else(|| {
                MfsError::InvalidPath("Cannot go above root".to_string())
            });
        }

        let mut new_segments = self.segments.clone();
        new_segments.push(segment.to_string());

        Ok(Self {
            segments: new_segments,
            is_absolute: self.is_absolute,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_root() {
        let path = MfsPath::parse("/").unwrap();
        assert!(path.is_root());
        assert_eq!(path.segments.len(), 0);
        assert_eq!(path.to_string(), "/");
    }

    #[test]
    fn test_parse_simple() {
        let path = MfsPath::parse("/foo").unwrap();
        assert_eq!(path.segments, vec!["foo"]);
        assert_eq!(path.to_string(), "/foo");
    }

    #[test]
    fn test_parse_nested() {
        let path = MfsPath::parse("/foo/bar/baz").unwrap();
        assert_eq!(path.segments, vec!["foo", "bar", "baz"]);
        assert_eq!(path.depth(), 3);
    }

    #[test]
    fn test_parse_trailing_slash() {
        let path = MfsPath::parse("/foo/bar/").unwrap();
        assert_eq!(path.segments, vec!["foo", "bar"]);
    }

    #[test]
    fn test_parse_dot() {
        let path = MfsPath::parse("/foo/./bar").unwrap();
        assert_eq!(path.segments, vec!["foo", "bar"]);
    }

    #[test]
    fn test_parse_double_dot_fails() {
        let result = MfsPath::parse("/foo/../bar");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_relative_fails() {
        let result = MfsPath::parse("foo/bar");
        assert!(result.is_err());
    }

    #[test]
    fn test_parent() {
        let path = MfsPath::parse("/foo/bar/baz").unwrap();
        let parent = path.parent().unwrap();
        assert_eq!(parent.to_string(), "/foo/bar");

        let grandparent = parent.parent().unwrap();
        assert_eq!(grandparent.to_string(), "/foo");

        let root = grandparent.parent().unwrap();
        assert!(root.is_root());

        assert!(root.parent().is_none());
    }

    #[test]
    fn test_name() {
        let path = MfsPath::parse("/foo/bar/baz.txt").unwrap();
        assert_eq!(path.name(), Some("baz.txt"));

        let root = MfsPath::parse("/").unwrap();
        assert_eq!(root.name(), None);
    }

    #[test]
    fn test_join() {
        let path = MfsPath::parse("/foo").unwrap();
        let joined = path.join("bar").unwrap();
        assert_eq!(joined.to_string(), "/foo/bar");

        let joined2 = joined.join("baz").unwrap();
        assert_eq!(joined2.to_string(), "/foo/bar/baz");
    }

    #[test]
    fn test_join_with_slash_fails() {
        let path = MfsPath::parse("/foo").unwrap();
        let result = path.join("bar/baz");
        assert!(result.is_err());
    }
}
