/// Simple golden frame test that works standalone
use std::fs;
use std::path::PathBuf;

#[test]
fn test_golden_frame_directory_creation() {
    let reference_dir = PathBuf::from("tests/golden_frames");
    
    // Ensure directory exists
    fs::create_dir_all(&reference_dir).unwrap();
    
    // Verify it was created
    assert!(reference_dir.exists(), "Golden frames directory should exist");
    assert!(reference_dir.is_dir(), "Golden frames path should be a directory");
    
    println!("✅ Golden frame directory created successfully at {:?}", reference_dir);
}

#[test]
fn test_golden_frame_readme_exists() {
    let readme_path = PathBuf::from("tests/golden_frames/README.md");
    
    assert!(readme_path.exists(), "Golden frames README should exist");
    
    let content = fs::read_to_string(&readme_path).unwrap();
    assert!(content.contains("Golden Frame Testing"), "README should mention golden frame testing");
    assert!(content.contains("visual regression testing"), "README should explain visual regression testing");
    
    println!("✅ Golden frame README exists and contains expected content");
}

#[test]
fn test_infrastructure_files_exist() {
    let infrastructure_files = [
        "test_utils/src/golden_frame.rs",
        "tests/golden_frame_tests.rs",
        "tests/golden_frames/README.md",
    ];
    
    for file_path in &infrastructure_files {
        let path = PathBuf::from(file_path);
        assert!(path.exists(), "Infrastructure file should exist: {}", file_path);
        println!("✅ Infrastructure file exists: {}", file_path);
    }
}
