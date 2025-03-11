use std::path::PathBuf;
use hemtt_workspace::{Workspace, LayerType, WorkspacePath};
use hemtt_workspace::position::{Position, LineCol};
use hemtt_common::config::PDriveOption;

pub fn setup_workspace(code: &str, file_path: &PathBuf) -> Result<(Position, WorkspacePath), String> {
    let start = LineCol::from_content(code, 0);
    let end = LineCol::from_content(code, code.len());
    
    // Get absolute paths to ensure correct workspace setup
    let abs_path = file_path.canonicalize()
        .map_err(|e| format!("Failed to get absolute path: {e:?}"))?;
    let parent = abs_path.parent()
        .ok_or_else(|| "Failed to get parent directory".to_string())?;
    
    // Create a workspace with a physical layer for the source
    let workspace = Workspace::builder()
        .physical(&PathBuf::from(parent), LayerType::Source)
        .finish(None, false, &PDriveOption::Disallow)
        .map_err(|e| format!("Failed to create workspace: {e:?}"))?;
    
    let workspace_path = workspace.join(abs_path.file_name().unwrap().to_str().unwrap())
        .map_err(|e| format!("Failed to create workspace path: {e:?}"))?;
    
    let position = Position::new(start, end, workspace_path.clone());
    
    Ok((position, workspace_path))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::fs;

    #[test]
    fn test_workspace_setup() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let file_path = temp_dir.path().join("test.sqf");
        fs::write(&file_path, "test content").expect("Failed to write test file");
        
        let result = setup_workspace("test content", &file_path);
        assert!(result.is_ok());
    }
} 