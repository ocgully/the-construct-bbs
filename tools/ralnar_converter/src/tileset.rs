//! Tile name registry for mapping indices to names
//!
//! The MMIFILES.TXT file contains the ordered list of all tiles.
//! Map files reference tiles by their 1-based index into this list.

use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// Load tile name registry from MMIFILES.TXT
pub fn load_tile_registry<P: AsRef<Path>>(path: P) -> Result<Vec<String>, std::io::Error> {
    let content = fs::read_to_string(path)?;
    let names: Vec<String> = content
        .lines()
        .map(|line| {
            // Remove .MMI extension and convert to lowercase
            line.trim()
                .trim_end_matches(".MMI")
                .trim_end_matches(".mmi")
                .to_lowercase()
        })
        .filter(|s| !s.is_empty())
        .collect();
    Ok(names)
}

/// Load tile attributes from metadata JSON files
pub fn load_tile_attributes<P: AsRef<Path>>(metadata_dir: P) -> HashMap<String, u8> {
    let mut attributes = HashMap::new();
    let dir = metadata_dir.as_ref();

    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.filter_map(|e| e.ok()) {
            let path = entry.path();
            if path.extension().map(|e| e == "json").unwrap_or(false) {
                if let Ok(content) = fs::read_to_string(&path) {
                    // Parse the metadata JSON to get attribute_code
                    if let Ok(meta) = serde_json::from_str::<serde_json::Value>(&content) {
                        if let (Some(name), Some(attr)) = (
                            meta.get("name").and_then(|v| v.as_str()),
                            meta.get("attribute_code").and_then(|v| v.as_u64()),
                        ) {
                            attributes.insert(name.to_lowercase(), attr as u8);
                        }
                    }
                }
            }
        }
    }

    attributes
}

/// Get tile name from 1-based index (as used in map files)
pub fn tile_name_from_index(registry: &[String], index: u16) -> String {
    if index == 0 || index as usize > registry.len() {
        return format!("unknown_{}", index);
    }
    registry[(index - 1) as usize].clone()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tile_name_from_index() {
        let registry = vec![
            "bc".to_string(),
            "castle1".to_string(),
            "cave1".to_string(),
        ];

        assert_eq!(tile_name_from_index(&registry, 1), "bc");
        assert_eq!(tile_name_from_index(&registry, 2), "castle1");
        assert_eq!(tile_name_from_index(&registry, 3), "cave1");
        assert_eq!(tile_name_from_index(&registry, 0), "unknown_0");
        assert_eq!(tile_name_from_index(&registry, 100), "unknown_100");
    }
}
