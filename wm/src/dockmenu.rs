use crate::menu::{MenuRow, MenuView};

pub type DockMenu = MenuView<u32>;

pub struct DockMenuAction {
    pub label: String,
    pub window: u32,
}

pub fn dock_menu(items: Vec<DockMenuAction>) -> DockMenu {
    MenuView::with_items(
        items
            .into_iter()
            .map(|it| MenuRow::item(it.label, it.window))
            .collect(),
    )
}

#[cfg(test)]
#[path = "dockmenu_tests.rs"]
mod tests;
