mod data;
pub use self::data::*;

use crate::action::*;
use crate::menu::{MenuRow, MenuView};

pub type WindowActionMenu = MenuView<Action>;

impl MenuView<Action> {
    pub fn for_focused_client(workspace_count: u32, tc: &crate::render::ThemeColors) -> Self {
        Self::for_focused_client_opts(workspace_count, tc)
    }

    pub fn for_focused_client_opts(workspace_count: u32, tc: &crate::render::ThemeColors) -> Self {
        let mut items: Vec<MenuRow<Action>> = Vec::new();
        items.push(MenuRow::item("_Restore", Action::Window(WindowOp::Restore)));
        items.push(MenuRow::item("_Move", Action::Window(WindowOp::Move)));
        items.push(MenuRow::item("_Size", Action::Window(WindowOp::Resize)));
        items.push(MenuRow::item(
            "Mi_nimize",
            Action::Window(WindowOp::Minimize),
        ));
        items.push(MenuRow::submenu(
            "Ma_ximize",
            vec![
                MenuRow::item("Ma_ximize", Action::Window(WindowOp::Maximize)),
                MenuRow::item("Maximize _Vertical", Action::Window(WindowOp::MaximizeVert)),
                MenuRow::item(
                    "Maximize Hori_zontal",
                    Action::Window(WindowOp::MaximizeHoriz),
                ),
            ],
        ));
        items.push(MenuRow::submenu(
            "_Tile",
            vec![
                MenuRow::item("Left _Half", Action::Tile(TileOp::TileLeft)),
                MenuRow::item("_Right Half", Action::Tile(TileOp::TileRight)),
                MenuRow::item("_Top Half", Action::Tile(TileOp::TileTop)),
                MenuRow::item("_Bottom Half", Action::Tile(TileOp::TileBottom)),
                MenuRow::item("Top _Left", Action::Tile(TileOp::TileTopLeft)),
                MenuRow::item("Top _Right", Action::Tile(TileOp::TileTopRight)),
                MenuRow::item("Bottom _Left", Action::Tile(TileOp::TileBottomLeft)),
                MenuRow::item("Bottom _Right", Action::Tile(TileOp::TileBottomRight)),
            ],
        ));
        items.push(MenuRow::submenu(
            "Roll_up",
            vec![
                MenuRow::item("Roll_up", Action::Window(WindowOp::Rollup)),
                MenuRow::item("_Shade", Action::Window(WindowOp::Shade)),
            ],
        ));
        items.push(MenuRow::item("_Hide", Action::Window(WindowOp::Hide)));
        items.push(MenuRow::separator());
        items.push(MenuRow::submenu(
            "La_yer",
            vec![
                MenuRow::item("_Above All", Action::Layer(LayerOp::AboveAll)),
                MenuRow::item("_Dock", Action::Layer(LayerOp::Dock)),
                MenuRow::item("_Fullscreen", Action::Layer(LayerOp::Fullscreen)),
                MenuRow::item("_On Top", Action::Layer(LayerOp::OnTop)),
                MenuRow::item("_Normal", Action::Layer(LayerOp::Normal)),
                MenuRow::item("_Below", Action::Layer(LayerOp::Below)),
                MenuRow::item("D_esktop", Action::Layer(LayerOp::Desktop)),
            ],
        ));
        items.push(MenuRow::submenu("_Workspace", {
            let mut ws = Vec::new();
            for i in 0..workspace_count {
                ws.push(MenuRow::item(
                    format!("Workspace {}", i + 1),
                    Action::Workspace(WorkspaceOp::MoveWindowTo(i)),
                ));
            }
            ws
        }));
        items.push(MenuRow::separator());
        items.push(MenuRow::item("_Close", Action::Window(WindowOp::Close)));
        items.push(MenuRow::item("_Kill", Action::Window(WindowOp::Kill)));

        let mut m = Self::with_items(items);
        m.colours = crate::menu::MenuColors::from_theme(tc);
        m
    }
}

#[cfg(test)]
mod tests;
