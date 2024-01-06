use crate::menu::MenuView;
use crate::menu_tree::MenuNode;

pub type DockMenu = MenuView<u32>;

pub struct DockMenuAction {
    pub label: String,
    pub window: u32,
}

pub fn dock_menu(items: Vec<DockMenuAction>) -> DockMenu {
    MenuView::with_nodes(
        items
            .into_iter()
            .map(|it| MenuNode::leaf(it.label, it.window))
            .collect(),
    )
}

#[cfg(test)]
#[path = "dockmenu_tests.rs"]
mod tests;
