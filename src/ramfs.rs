use alloc::{
    string::{String, ToString},
    vec::Vec,
};
use spin::Mutex;

#[derive(Debug)]
pub enum NodeType {
    File { data: Vec<u8> },
    Dir { children: Vec<Node> },
}

#[derive(Debug)]
pub struct Node {
    pub name: String,
    pub node_type: NodeType,
}

impl Node {
    fn new_dir(name: &str) -> Self {
        Node {
            name: name.to_string(),
            node_type: NodeType::Dir { children: Vec::new() },
        }
    }

    fn new_file(name: &str, data: Vec<u8>) -> Self {
        Node {
            name: name.to_string(),
            node_type: NodeType::File { data },
        }
    }
}

/// Global RAMFS root
static RAMFS_ROOT: Mutex<Node> = Mutex::new(Node {
    name: String::new(),
    node_type: NodeType::Dir { children: Vec::new() },
});

fn split_path(path: &str) -> Vec<&str> {
    path.split('/').filter(|p| !p.is_empty()).collect()
}

/// Resolve relative or absolute path
pub fn resolve_path(current_directory: &str, target: &str) -> String {
    if target.starts_with('/') {
        // Absolute path
        target.to_string()
    } else {
        // Relative path
        let mut parts: Vec<&str> = split_path(current_directory);
        for p in split_path(target) {
            match p {
                "." => {}
                ".." => { parts.pop(); }
                _ => parts.push(p),
            }
        }

        // Build path manually
        let mut path = String::new();
        path.push('/');
        let mut first = true;
        for part in parts {
            if !first {
                path.push('/');
            }
            first = false;
            path.push_str(part);
        }
        path
    }
}

fn find_dir_mut<'a>(mut current: &'a mut Node, parts: &[&str]) -> Option<&'a mut Node> {
    for part in parts {
        match &mut current.node_type {
            NodeType::Dir { children } => {
                current = children.iter_mut().find(|n| n.name == *part)?;
            }
            _ => return None,
        }
    }
    Some(current)
}

fn find_parent_dir_mut<'a, 'b>(
    root: &'a mut Node,
    path: &'b [&'b str],
) -> Option<(&'a mut Node, &'b str)> {
    if path.is_empty() {
        return None;
    }

    let (dirs, name) = path.split_at(path.len() - 1);
    let parent = find_dir_mut(root, dirs)?;
    Some((parent, name[0]))
}

/// Change current directory
pub fn change_directory(current_directory: &str, to_directory: &str) -> Option<String> {
    let path = resolve_path(current_directory, to_directory);
    let parts = split_path(&path);
    let root = &mut RAMFS_ROOT.lock();
    if find_dir_mut(root, &parts).is_some() {
        Some(path)
    } else {
        None
    }
}

/// Create directory
pub fn mkdir(current_directory: &str, path: &str) -> bool {
    let path = resolve_path(current_directory, path);
    let parts = split_path(&path);
    let mut root = RAMFS_ROOT.lock();

    let (parent, name) = match find_parent_dir_mut(&mut root, &parts) {
        Some(v) => v,
        None => return false,
    };

    if let NodeType::Dir { children } = &mut parent.node_type {
        if children.iter().any(|n| n.name == name) { return false; }
        children.push(Node::new_dir(name));
        true
    } else {
        false
    }
}

/// Create file
pub fn create_file(current_directory: &str, path: &str, data: &[u8]) -> bool {
    let path = resolve_path(current_directory, path);
    let parts = split_path(&path);
    let mut root = RAMFS_ROOT.lock();

    let (parent, name) = match find_parent_dir_mut(&mut root, &parts) {
        Some(v) => v,
        None => return false,
    };

    if let NodeType::Dir { children } = &mut parent.node_type {
        if children.iter().any(|n| n.name == name) { return false; }
        children.push(Node::new_file(name, data.to_vec()));
        true
    } else {
        false
    }
}

/// Update file
pub fn update_file(current_directory: &str, path: &str, data: &[u8]) -> bool {
    let path = resolve_path(current_directory, path);
    let parts = split_path(&path);
    let mut root = RAMFS_ROOT.lock();

    let file = match find_dir_mut(&mut root, &parts) {
        Some(f) => f,
        None => return false,
    };

    if let NodeType::File { data: file_data } = &mut file.node_type {
        *file_data = data.to_vec();
        true
    } else { false }
}

/// Rename file
pub fn rename_file(current_directory: &str, path: &str, new_name: &str) -> bool {
    let path = resolve_path(current_directory, path);
    let parts = split_path(&path);
    let mut root = RAMFS_ROOT.lock();

    let (parent, old_name) = match find_parent_dir_mut(&mut root, &parts) { Some(v) => v, None => return false };

    if let NodeType::Dir { children } = &mut parent.node_type {
        if children.iter().any(|n| n.name == new_name) { return false; }
        if let Some(f) = children.iter_mut().find(|n| n.name == old_name) {
            if matches!(f.node_type, NodeType::File { .. }) {
                f.name = new_name.to_string();
                return true;
            }
        }
    }
    false
}

/// Rename folder
pub fn rename_folder(current_directory: &str, path: &str, new_name: &str) -> bool {
    let path = resolve_path(current_directory, path);
    let parts = split_path(&path);
    let mut root = RAMFS_ROOT.lock();

    let (parent, old_name) = match find_parent_dir_mut(&mut root, &parts) { Some(v) => v, None => return false };

    if let NodeType::Dir { children } = &mut parent.node_type {
        if children.iter().any(|n| n.name == new_name) { return false; }
        if let Some(f) = children.iter_mut().find(|n| n.name == old_name) {
            if matches!(f.node_type, NodeType::Dir { .. }) {
                f.name = new_name.to_string();
                return true;
            }
        }
    }
    false
}

/// Read file
pub fn read_file(current_directory: &str, path: &str) -> Option<Vec<u8>> {
    let path = resolve_path(current_directory, path);
    let parts = split_path(&path);
    let mut root = RAMFS_ROOT.lock();

    let node = find_dir_mut(&mut root, &parts)?;
    match &node.node_type {
        NodeType::File { data } => Some(data.clone()),
        _ => None,
    }
}

/// Delete
pub fn delete(current_directory: &str, path: &str) -> bool {
    let path = resolve_path(current_directory, path);
    let parts = split_path(&path);
    let mut root = RAMFS_ROOT.lock();

    let (parent, name) = match find_parent_dir_mut(&mut root, &parts) { Some(v) => v, None => return false };

    if let NodeType::Dir { children } = &mut parent.node_type {
        let before = children.len();
        children.retain(|n| n.name != name);
        return before != children.len();
    }
    false
}

/// List directory
pub fn list_dir(current_directory: &str, path: &str) -> Option<Vec<String>> {
    let path = resolve_path(current_directory, path);
    let parts = split_path(&path);
    let mut root = RAMFS_ROOT.lock();

    let dir = find_dir_mut(&mut root, &parts)?;
    match &dir.node_type {
        NodeType::Dir { children } => Some(children.iter().map(|n| n.name.clone()).collect()),
        _ => None,
    }
}