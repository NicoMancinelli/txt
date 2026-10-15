use std::fs;
use std::path::PathBuf;
use std::process::Command;
use tempfile::tempdir;
use txt_core::git_status::{GitFileStatus, GitStatusScanner};

#[test]
fn test_git_file_status_badges_and_colors() {
    assert_eq!(GitFileStatus::Modified.badge(), "M");
    assert_eq!(GitFileStatus::Untracked.badge(), "U");
    assert_eq!(GitFileStatus::Added.badge(), "A");
    assert_eq!(GitFileStatus::Deleted.badge(), "D");
    assert_eq!(GitFileStatus::Renamed.badge(), "R");
    assert_eq!(GitFileStatus::Ignored.badge(), "I");
    assert_eq!(GitFileStatus::Clean.badge(), "");

    assert_eq!(GitFileStatus::Modified.color_rgb(), (229, 192, 123));
    assert_eq!(GitFileStatus::Untracked.color_rgb(), (152, 195, 121));
    assert_eq!(GitFileStatus::Added.color_rgb(), (152, 195, 121));
    assert_eq!(GitFileStatus::Deleted.color_rgb(), (224, 108, 117));
    assert_eq!(GitFileStatus::Renamed.color_rgb(), (97, 175, 239));
    assert_eq!(GitFileStatus::Ignored.color_rgb(), (120, 120, 120));
    assert_eq!(GitFileStatus::Clean.color_rgb(), (171, 178, 191));

    assert!(GitFileStatus::Modified.is_dirty());
    assert!(GitFileStatus::Untracked.is_dirty());
    assert!(GitFileStatus::Added.is_dirty());
    assert!(GitFileStatus::Deleted.is_dirty());
    assert!(GitFileStatus::Renamed.is_dirty());
    assert!(!GitFileStatus::Clean.is_dirty());
    assert!(!GitFileStatus::Ignored.is_dirty());

    assert!(GitFileStatus::Clean.is_clean());
    assert!(!GitFileStatus::Modified.is_clean());
}

#[test]
fn test_parse_porcelain_output() {
    let repo_root = PathBuf::from("/mock/repo");
    let porcelain = "\
?? untracked.rs
 M modified_worktree.rs
M  modified_staged.rs
MM modified_both.rs
A  added.rs
 D deleted_worktree.rs
D  deleted_staged.rs
R  old_name.rs -> new_name.rs
!! ignored.log
";

    let status_map = GitStatusScanner::parse_porcelain_output(porcelain, &repo_root);

    assert_eq!(
        status_map.get(&repo_root.join("untracked.rs")),
        Some(&GitFileStatus::Untracked)
    );
    assert_eq!(
        status_map.get(&repo_root.join("modified_worktree.rs")),
        Some(&GitFileStatus::Modified)
    );
    assert_eq!(
        status_map.get(&repo_root.join("modified_staged.rs")),
        Some(&GitFileStatus::Modified)
    );
    assert_eq!(
        status_map.get(&repo_root.join("modified_both.rs")),
        Some(&GitFileStatus::Modified)
    );
    assert_eq!(
        status_map.get(&repo_root.join("added.rs")),
        Some(&GitFileStatus::Added)
    );
    assert_eq!(
        status_map.get(&repo_root.join("deleted_worktree.rs")),
        Some(&GitFileStatus::Deleted)
    );
    assert_eq!(
        status_map.get(&repo_root.join("deleted_staged.rs")),
        Some(&GitFileStatus::Deleted)
    );
    assert_eq!(
        status_map.get(&repo_root.join("new_name.rs")),
        Some(&GitFileStatus::Renamed)
    );
    assert_eq!(
        status_map.get(&repo_root.join("ignored.log")),
        Some(&GitFileStatus::Ignored)
    );
}

#[test]
fn test_find_repo_root_non_git() {
    let temp = tempdir().unwrap();
    let sub = temp.path().join("a").join("b").join("c");
    fs::create_dir_all(&sub).unwrap();

    assert_eq!(GitStatusScanner::find_repo_root(&sub), None);
    assert!(GitStatusScanner::scan_repository(&sub).is_empty());
}

#[test]
fn test_scan_repository_live_git_integration() {
    let temp = tempdir().unwrap();
    let root = temp.path().canonicalize().unwrap();

    // Check if git is available
    let git_check = Command::new("git").arg("--version").output();
    if git_check.is_err() || !git_check.unwrap().status.success() {
        eprintln!("git command not available, skipping live git test");
        return;
    }

    // Init git repo
    let init_res = Command::new("git")
        .arg("init")
        .current_dir(&root)
        .output()
        .expect("git init failed");
    assert!(init_res.status.success());

    // Configure dummy user for committing
    let _ = Command::new("git")
        .args(["config", "user.name", "Tester"])
        .current_dir(&root)
        .output();
    let _ = Command::new("git")
        .args(["config", "user.email", "tester@test.com"])
        .current_dir(&root)
        .output();

    // 1. Create a committed file
    let clean_file = root.join("clean.txt");
    fs::write(&clean_file, "clean content").unwrap();

    let _ = Command::new("git")
        .args(["add", "clean.txt"])
        .current_dir(&root)
        .output();
    let _ = Command::new("git")
        .args(["commit", "-m", "initial commit"])
        .current_dir(&root)
        .output();

    // 2. Create an untracked file
    let untracked_file = root.join("untracked.txt");
    fs::write(&untracked_file, "untracked content").unwrap();

    // 3. Create a modified file
    let mod_file = root.join("modified.txt");
    fs::write(&mod_file, "initial mod content").unwrap();
    let _ = Command::new("git")
        .args(["add", "modified.txt"])
        .current_dir(&root)
        .output();
    let _ = Command::new("git")
        .args(["commit", "-m", "commit mod"])
        .current_dir(&root)
        .output();
    fs::write(&mod_file, "updated mod content").unwrap();

    // 4. Create an added (staged) file
    let added_file = root.join("added.txt");
    fs::write(&added_file, "staged content").unwrap();
    let _ = Command::new("git")
        .args(["add", "added.txt"])
        .current_dir(&root)
        .output();

    // Scan repository
    let status_map = GitStatusScanner::scan_repository(&root);

    assert_eq!(
        status_map.get(&untracked_file.canonicalize().unwrap()),
        Some(&GitFileStatus::Untracked)
    );
    assert_eq!(
        status_map.get(&mod_file.canonicalize().unwrap()),
        Some(&GitFileStatus::Modified)
    );
    assert_eq!(
        status_map.get(&added_file.canonicalize().unwrap()),
        Some(&GitFileStatus::Added)
    );

    // Clean file is not in porcelain map, and get_file_status should return Clean
    assert_eq!(
        GitStatusScanner::get_file_status(&root, &clean_file),
        GitFileStatus::Clean
    );
    assert_eq!(
        GitStatusScanner::get_file_status(&root, &untracked_file),
        GitFileStatus::Untracked
    );
}
