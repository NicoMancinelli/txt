use std::fs;
use tempfile::tempdir;
use txt_core::project::{FileOps, ProjectFolder, ProjectManager, TreeNode};

#[test]
fn test_tree_building_and_sorting() {
    let dir = tempdir().unwrap();
    let root = dir.path();

    // Create files and dirs out of alphabetical order
    fs::create_dir(root.join("z_dir")).unwrap();
    fs::create_dir(root.join("a_dir")).unwrap();
    fs::write(root.join("b_file.txt"), "b").unwrap();
    fs::write(root.join("a_file.rs"), "a").unwrap();
    fs::write(root.join("z_dir").join("nested.txt"), "nested").unwrap();
    fs::write(root.join("a_dir").join("sub.rs"), "sub").unwrap();

    let folder = ProjectFolder::new(root.to_path_buf());
    let children = &folder.root_node.children;

    // Must have 4 children: a_dir, z_dir, a_file.rs, b_file.txt
    assert_eq!(children.len(), 4);

    // Directories first, sorted alphabetically
    assert_eq!(children[0].name, "a_dir");
    assert!(children[0].is_dir);

    assert_eq!(children[1].name, "z_dir");
    assert!(children[1].is_dir);

    // Files next, sorted alphabetically
    assert_eq!(children[2].name, "a_file.rs");
    assert!(!children[2].is_dir);

    assert_eq!(children[3].name, "b_file.txt");
    assert!(!children[3].is_dir);

    // Verify nested children
    let a_dir = &children[0];
    assert_eq!(a_dir.children.len(), 1);
    assert_eq!(a_dir.children[0].name, "sub.rs");

    // Verify TreeNode count_nodes and flatten_visible
    assert!(folder.root_node.count_nodes() >= 6);
    let visible = folder.root_node.flatten_visible();
    assert!(!visible.is_empty());

    // Test TreeNode constructor
    let standalone = TreeNode::new(
        root.join("manual.txt"),
        "manual.txt".to_string(),
        false,
        123,
        false,
    );
    assert_eq!(standalone.name, "manual.txt");
    assert_eq!(standalone.size_bytes, 123);
    assert_eq!(standalone.count_nodes(), 1);
}

#[test]
fn test_expand_collapse_all_and_toggle() {
    let dir = tempdir().unwrap();
    let root = dir.path();

    fs::create_dir_all(root.join("dir1").join("subdir1")).unwrap();
    fs::create_dir(root.join("dir2")).unwrap();
    fs::write(root.join("dir1").join("subdir1").join("deep.txt"), "deep").unwrap();

    let mut folder = ProjectFolder::new(root.to_path_buf());
    assert!(folder.root_node.is_expanded);

    // Expand all
    folder.expand_all();
    assert!(folder.root_node.is_expanded);
    let dir1_path = root.join("dir1");
    let subdir1_path = dir1_path.join("subdir1");
    assert!(folder.expanded_paths.contains(&dir1_path));
    assert!(folder.expanded_paths.contains(&subdir1_path));

    let dir1_node = folder.root_node.find_node(&dir1_path).unwrap();
    assert!(dir1_node.is_expanded);

    // Collapse all
    folder.collapse_all();
    assert!(!folder.root_node.is_expanded);
    assert!(folder.expanded_paths.is_empty());
    let dir1_node = folder.root_node.find_node(&dir1_path).unwrap();
    assert!(!dir1_node.is_expanded);

    // Toggle expand
    folder.toggle_expand(&dir1_path);
    assert!(folder.expanded_paths.contains(&dir1_path));
    let dir1_node = folder.root_node.find_node(&dir1_path).unwrap();
    assert!(dir1_node.is_expanded);

    folder.toggle_expand(&dir1_path);
    assert!(!folder.expanded_paths.contains(&dir1_path));
    let dir1_node = folder.root_node.find_node(&dir1_path).unwrap();
    assert!(!dir1_node.is_expanded);
}

#[test]
fn test_reveal_path() {
    let dir = tempdir().unwrap();
    let root = dir.path();

    let target_dir = root.join("nested").join("sub");
    fs::create_dir_all(&target_dir).unwrap();
    let target_file = target_dir.join("item.txt");
    fs::write(&target_file, "content").unwrap();

    let mut folder = ProjectFolder::new(root.to_path_buf());
    folder.collapse_all();

    // Target file is not currently expanded in ancestors
    assert!(!folder.expanded_paths.contains(&root.join("nested")));
    assert!(!folder.expanded_paths.contains(&target_dir));

    // Reveal target file
    let revealed = folder.reveal_path(&target_file);
    assert!(revealed);

    // Ancestor directories must now be expanded
    assert!(folder.root_node.is_expanded);
    assert!(folder.expanded_paths.contains(&root.join("nested")));
    assert!(folder.expanded_paths.contains(&target_dir));

    let nested_node = folder.root_node.find_node(&root.join("nested")).unwrap();
    assert!(nested_node.is_expanded);
    let sub_node = folder.root_node.find_node(&target_dir).unwrap();
    assert!(sub_node.is_expanded);

    // Revealing non-existent file returns false
    assert!(!folder.reveal_path(&root.join("does_not_exist.txt")));
    // Revealing path outside root returns false
    assert!(!folder.reveal_path(&std::path::PathBuf::from("/etc/hosts")));
}

#[test]
fn test_ignore_filtering_and_hidden_files() {
    let dir = tempdir().unwrap();
    let root = dir.path();

    fs::create_dir(root.join(".git")).unwrap();
    fs::create_dir(root.join("target")).unwrap();
    fs::create_dir(root.join("node_modules")).unwrap();
    fs::create_dir(root.join("src")).unwrap();
    fs::write(root.join(".DS_Store"), "").unwrap();
    fs::write(root.join(".env"), "SECRET=1").unwrap();
    fs::write(root.join("src").join("main.rs"), "fn main() {}").unwrap();

    let mut folder = ProjectFolder::new(root.to_path_buf());

    // By default, show_hidden is false -> .git, target, node_modules, .DS_Store, .env are filtered
    let visible_names: Vec<String> = folder.root_node.children.iter().map(|c| c.name.clone()).collect();
    assert_eq!(visible_names, vec!["src"]);

    // Refresh with show_hidden = true
    folder.refresh(true, None);
    let all_names: Vec<String> = folder.root_node.children.iter().map(|c| c.name.clone()).collect();
    assert!(all_names.contains(&".git".to_string()));
    assert!(all_names.contains(&"target".to_string()));
    assert!(all_names.contains(&"node_modules".to_string()));
    assert!(all_names.contains(&".DS_Store".to_string()));
    assert!(all_names.contains(&".env".to_string()));
    assert!(all_names.contains(&"src".to_string()));

    let env_node = folder.root_node.children.iter().find(|c| c.name == ".env").unwrap();
    assert!(env_node.is_hidden);

    let src_node = folder.root_node.children.iter().find(|c| c.name == "src").unwrap();
    assert!(!src_node.is_hidden);
}

#[test]
fn test_search_filter_in_tree() {
    let dir = tempdir().unwrap();
    let root = dir.path();

    fs::create_dir(root.join("src")).unwrap();
    fs::create_dir(root.join("docs")).unwrap();
    fs::write(root.join("src").join("main.rs"), "").unwrap();
    fs::write(root.join("src").join("utils.rs"), "").unwrap();
    fs::write(root.join("docs").join("guide.md"), "").unwrap();

    let mut folder = ProjectFolder::new(root.to_path_buf());

    // Filter by "utils"
    folder.refresh(false, Some("utils"));
    let child_names: Vec<String> = folder.root_node.children.iter().map(|c| c.name.clone()).collect();
    assert_eq!(child_names, vec!["src"]);

    let src_node = &folder.root_node.children[0];
    let src_children: Vec<String> = src_node.children.iter().map(|c| c.name.clone()).collect();
    assert_eq!(src_children, vec!["utils.rs"]);

    // Filter by "guide"
    folder.refresh(false, Some("guide"));
    let child_names: Vec<String> = folder.root_node.children.iter().map(|c| c.name.clone()).collect();
    assert_eq!(child_names, vec!["docs"]);

    // Clear filter
    folder.refresh(false, None);
    assert_eq!(folder.root_node.children.len(), 2);
}

#[test]
fn test_project_manager_multi_root() {
    let dir1 = tempdir().unwrap();
    let dir2 = tempdir().unwrap();

    fs::write(dir1.path().join("f1.txt"), "1").unwrap();
    let f2_path = dir2.path().join("f2.txt");
    fs::write(&f2_path, "2").unwrap();

    let mut pm = ProjectManager::new();
    pm.add_folder(dir1.path().to_path_buf());
    pm.add_folder(dir2.path().to_path_buf());

    assert_eq!(pm.folders.len(), 2);

    // Reveal f2 in multi-root project
    assert!(pm.reveal_path(&f2_path));
    assert!(pm.folders[1].expanded_paths.contains(dir2.path()));

    // Toggle show_hidden
    assert!(!pm.show_hidden);
    pm.toggle_show_hidden();
    assert!(pm.show_hidden);

    // Remove folder
    pm.remove_folder(dir1.path());
    assert_eq!(pm.folders.len(), 1);
    assert_eq!(pm.folders[0].root_path, dir2.path());
}

#[test]
fn test_file_ops_create_and_rename() {
    let dir = tempdir().unwrap();
    let root = dir.path();

    // Create dir
    let nested_dir = root.join("nested").join("sub");
    FileOps::create_dir(&nested_dir).unwrap();
    assert!(nested_dir.is_dir());

    // Create file
    let new_file = nested_dir.join("test.txt");
    FileOps::create_file(&new_file).unwrap();
    assert!(new_file.is_file());

    // Rename file
    let renamed_file = root.join("renamed.txt");
    FileOps::rename(&new_file, &renamed_file).unwrap();
    assert!(!new_file.exists());
    assert!(renamed_file.is_file());
}

#[test]
fn test_file_ops_duplicate_file_and_dir() {
    let dir = tempdir().unwrap();
    let root = dir.path();

    let foo = root.join("foo.rs");
    fs::write(&foo, "pub fn foo() {}").unwrap();

    // First duplication: foo.rs -> foo_copy.rs
    let copy1 = FileOps::duplicate(&foo).unwrap();
    assert_eq!(copy1, root.join("foo_copy.rs"));
    assert!(copy1.is_file());
    assert_eq!(fs::read_to_string(&copy1).unwrap(), "pub fn foo() {}");

    // Second duplication: foo_copy.rs exists -> foo (1).rs
    let copy2 = FileOps::duplicate(&foo).unwrap();
    assert_eq!(copy2, root.join("foo (1).rs"));
    assert!(copy2.is_file());

    // Third duplication: foo (1).rs exists -> foo (2).rs
    let copy3 = FileOps::duplicate(&foo).unwrap();
    assert_eq!(copy3, root.join("foo (2).rs"));
    assert!(copy3.is_file());

    // Test directory duplication
    let test_dir = root.join("my_module");
    fs::create_dir(&test_dir).unwrap();
    fs::write(test_dir.join("mod.rs"), "pub mod child;").unwrap();

    let dir_copy1 = FileOps::duplicate(&test_dir).unwrap();
    assert_eq!(dir_copy1, root.join("my_module_copy"));
    assert!(dir_copy1.is_dir());
    assert!(dir_copy1.join("mod.rs").is_file());

    let dir_copy2 = FileOps::duplicate(&test_dir).unwrap();
    assert_eq!(dir_copy2, root.join("my_module (1)"));
    assert!(dir_copy2.is_dir());
    assert!(dir_copy2.join("mod.rs").is_file());
}

#[test]
fn test_file_ops_delete() {
    let dir = tempdir().unwrap();
    let root = dir.path();

    // Delete single file
    let file = root.join("to_delete.txt");
    fs::write(&file, "data").unwrap();
    assert!(file.exists());
    FileOps::delete(&file).unwrap();
    assert!(!file.exists());

    // Delete directory recursively
    let dir_to_delete = root.join("folder");
    fs::create_dir(&dir_to_delete).unwrap();
    fs::write(dir_to_delete.join("nested.txt"), "nested").unwrap();
    assert!(dir_to_delete.exists());
    FileOps::delete(&dir_to_delete).unwrap();
    assert!(!dir_to_delete.exists());

    // Deleting non-existent path returns NotFound error
    let err = FileOps::delete(&root.join("non_existent")).unwrap_err();
    assert_eq!(err.kind(), std::io::ErrorKind::NotFound);
}
