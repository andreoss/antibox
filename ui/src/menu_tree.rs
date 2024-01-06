use antibox_core::backend::PixmapData;

#[derive(Clone)]
pub enum MenuNode<T> {
    Leaf {
        title: String,
        payload: T,
        icon: Option<PixmapData>,
    },
    Group {
        title: String,
        icon: Option<PixmapData>,
        children: Vec<MenuNode<T>>,
        expanded: bool,
    },
    Separator,
}

#[derive(Clone)]
pub enum FlatEntry<T> {
    Leaf(T),
    Group { expanded: bool, path: Vec<usize> },
    Separator,
}

#[derive(Clone)]
pub struct FlatRow<T> {
    pub title: String,
    pub icon: Option<PixmapData>,
    pub depth: u16,
    pub entry: FlatEntry<T>,
}

impl<T> FlatRow<T> {
    pub fn payload(&self) -> Option<&T> {
        match &self.entry {
            FlatEntry::Leaf(p) => Some(p),
            _ => None,
        }
    }

    pub fn is_group(&self) -> bool {
        matches!(self.entry, FlatEntry::Group { .. })
    }

    pub fn is_separator(&self) -> bool {
        matches!(self.entry, FlatEntry::Separator)
    }

    pub fn selectable(&self) -> bool {
        !self.is_separator()
    }
}

fn norm(title: &str) -> String {
    crate::menurender::parse_mnemonic(title).0.to_lowercase()
}

impl<T: Clone> MenuNode<T> {
    pub fn leaf(title: impl Into<String>, payload: T) -> Self {
        MenuNode::Leaf {
            title: title.into(),
            payload,
            icon: None,
        }
    }

    pub fn group(title: impl Into<String>, children: Vec<Self>) -> Self {
        MenuNode::Group {
            title: title.into(),
            icon: None,
            children,
            expanded: false,
        }
    }

    pub fn group_expanded(title: impl Into<String>, children: Vec<Self>) -> Self {
        MenuNode::Group {
            title: title.into(),
            icon: None,
            children,
            expanded: true,
        }
    }

    pub fn separator() -> Self {
        MenuNode::Separator
    }

    pub fn with_icon(mut self, ic: Option<PixmapData>) -> Self {
        match &mut self {
            MenuNode::Leaf { icon, .. } | MenuNode::Group { icon, .. } => *icon = ic,
            MenuNode::Separator => {}
        }
        self
    }

    pub fn title(&self) -> &str {
        match self {
            MenuNode::Leaf { title, .. } | MenuNode::Group { title, .. } => title,
            MenuNode::Separator => "",
        }
    }

    pub fn is_separator(&self) -> bool {
        matches!(self, MenuNode::Separator)
    }

    pub fn payload(&self) -> Option<&T> {
        match self {
            MenuNode::Leaf { payload, .. } => Some(payload),
            _ => None,
        }
    }

    pub fn children(&self) -> Option<&[MenuNode<T>]> {
        match self {
            MenuNode::Group { children, .. } => Some(children),
            _ => None,
        }
    }

    pub fn icon(&self) -> Option<&PixmapData> {
        match self {
            MenuNode::Leaf { icon, .. } | MenuNode::Group { icon, .. } => icon.as_ref(),
            MenuNode::Separator => None,
        }
    }

    pub fn flatten(&self, depth: u16) -> Vec<FlatRow<T>> {
        let mut out = Vec::new();
        let mut path = Vec::new();
        flatten_into(std::slice::from_ref(self), depth, &mut path, &mut out);
        out
    }

    pub fn filter(&self, needle: &str) -> Option<MenuNode<T>> {
        let needle = needle.to_lowercase();
        self.filter_norm(&needle)
    }

    fn filter_norm(&self, needle: &str) -> Option<MenuNode<T>> {
        match self {
            MenuNode::Separator => None,
            MenuNode::Leaf { title, .. } => {
                norm(title).contains(needle).then(|| self.clone())
            }
            MenuNode::Group {
                title,
                icon,
                children,
                ..
            } => {
                let kept = if norm(title).contains(needle) {
                    children.clone()
                } else {
                    children
                        .iter()
                        .filter_map(|c| c.filter_norm(needle))
                        .collect()
                };
                (!kept.is_empty()).then(|| MenuNode::Group {
                    title: title.clone(),
                    icon: icon.clone(),
                    children: kept,
                    expanded: true,
                })
            }
        }
    }
}

fn flatten_into<T: Clone>(
    nodes: &[MenuNode<T>],
    depth: u16,
    path: &mut Vec<usize>,
    out: &mut Vec<FlatRow<T>>,
) {
    for (i, node) in nodes.iter().enumerate() {
        match node {
            MenuNode::Leaf {
                title,
                payload,
                icon,
            } => out.push(FlatRow {
                title: title.clone(),
                icon: icon.clone(),
                depth,
                entry: FlatEntry::Leaf(payload.clone()),
            }),
            MenuNode::Separator => out.push(FlatRow {
                title: String::new(),
                icon: None,
                depth,
                entry: FlatEntry::Separator,
            }),
            MenuNode::Group {
                title,
                icon,
                children,
                expanded,
            } => {
                path.push(i);
                out.push(FlatRow {
                    title: title.clone(),
                    icon: icon.clone(),
                    depth,
                    entry: FlatEntry::Group {
                        expanded: *expanded,
                        path: path.clone(),
                    },
                });
                if *expanded {
                    flatten_into(children, depth + 1, path, out);
                }
                path.pop();
            }
        }
    }
}

pub fn flatten_nodes<T: Clone>(nodes: &[MenuNode<T>]) -> Vec<FlatRow<T>> {
    let mut out = Vec::new();
    let mut path = Vec::new();
    flatten_into(nodes, 0, &mut path, &mut out);
    out
}

pub fn filter_nodes<T: Clone>(nodes: &[MenuNode<T>], needle: &str) -> Vec<MenuNode<T>> {
    let needle = needle.to_lowercase();
    nodes.iter().filter_map(|n| n.filter_norm(&needle)).collect()
}

pub fn collect_leaves<T: Clone>(nodes: &[MenuNode<T>], out: &mut Vec<MenuNode<T>>) {
    for n in nodes {
        match n {
            MenuNode::Leaf { .. } => out.push(n.clone()),
            MenuNode::Group { children, .. } => collect_leaves(children, out),
            MenuNode::Separator => {}
        }
    }
}

pub fn toggle_at<T>(nodes: &mut [MenuNode<T>], path: &[usize]) -> bool {
    let (first, rest) = match path.split_first() {
        Some(p) => p,
        None => return false,
    };
    match nodes.get_mut(*first) {
        Some(MenuNode::Group {
            children, expanded, ..
        }) => {
            if rest.is_empty() {
                *expanded = !*expanded;
                true
            } else {
                toggle_at(children, rest)
            }
        }
        _ => false,
    }
}

#[cfg(test)]
#[path = "menu_tree_tests.rs"]
mod tests;
