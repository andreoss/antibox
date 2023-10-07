mod data;
pub use self::data::*;

use crate::action::*;
use crate::menu::{MenuNav, MenuView};
use crate::menu_tree::MenuNode;
use antibox_core::backend::*;
use antibox_core::point::Point;

pub struct WindowActionMenu {
    pub view: MenuView<Action>,
    nodes: Vec<MenuNode<Action>>,
}

impl Default for WindowActionMenu {
    fn default() -> Self {
        Self::new()
    }
}

impl WindowActionMenu {
    pub fn new() -> Self {
        WindowActionMenu {
            view: MenuView::new(),
            nodes: Vec::new(),
        }
    }

    pub fn for_focused_client(
        workspace_count: u32,
        tc: &crate::render::ThemeColors,
        shaded: bool,
    ) -> Self {
        Self::for_focused_client_opts(workspace_count, tc, &[], shaded)
    }

    pub fn for_focused_client_opts(
        workspace_count: u32,
        tc: &crate::render::ThemeColors,
        join: &[(u32, String)],
        shaded: bool,
    ) -> Self {
        let mut m = Self::new();
        m.view.colours = crate::menu::MenuColors::from_theme(tc);
        m.nodes = Self::action_nodes(workspace_count, join, shaded);
        m
    }

    pub fn action_nodes(
        workspace_count: u32,
        join: &[(u32, String)],
        shaded: bool,
    ) -> Vec<MenuNode<Action>> {
        let mut items: Vec<MenuNode<Action>> = vec![
            MenuNode::leaf("_Restore", Action::Window(WindowOp::Restore)),
            MenuNode::leaf("_Move", Action::Window(WindowOp::Move)),
            MenuNode::leaf("_Size", Action::Window(WindowOp::Resize)),
            MenuNode::leaf("Mi_nimize", Action::Window(WindowOp::Minimize)),
        ];
        items.push(MenuNode::group(
            "Ma_ximize",
            vec![
                MenuNode::leaf("Ma_ximize", Action::Window(WindowOp::Maximize)),
                MenuNode::leaf("Maximize _Vertical", Action::Window(WindowOp::MaximizeVert)),
                MenuNode::leaf(
                    "Maximize Hori_zontal",
                    Action::Window(WindowOp::MaximizeHoriz),
                ),
            ],
        ));
        items.push(MenuNode::group(
            "_Tile",
            vec![
                MenuNode::leaf("Left _Half", Action::Tile(TileOp::TileLeft)),
                MenuNode::leaf("_Right Half", Action::Tile(TileOp::TileRight)),
                MenuNode::leaf("_Top Half", Action::Tile(TileOp::TileTop)),
                MenuNode::leaf("_Bottom Half", Action::Tile(TileOp::TileBottom)),
                MenuNode::leaf("Top _Left", Action::Tile(TileOp::TileTopLeft)),
                MenuNode::leaf("Top _Right", Action::Tile(TileOp::TileTopRight)),
                MenuNode::leaf("Bottom _Left", Action::Tile(TileOp::TileBottomLeft)),
                MenuNode::leaf("Bottom _Right", Action::Tile(TileOp::TileBottomRight)),
            ],
        ));
        items.push(MenuNode::leaf(
            if shaded { "Un_roll" } else { "Roll_up" },
            Action::Window(WindowOp::Rollup),
        ));
        items.push(MenuNode::leaf("_Hide", Action::Window(WindowOp::Hide)));
        let mut tab_rows = vec![MenuNode::leaf("_Untab", Action::Tab(TabOp::Untab))];
        if !join.is_empty() {
            tab_rows.push(MenuNode::separator());
            for (xid, title) in join {
                tab_rows.push(MenuNode::leaf(
                    title.clone(),
                    Action::Tab(TabOp::JoinWindow(*xid)),
                ));
            }
        }
        items.push(MenuNode::group("Ta_b", tab_rows));
        items.push(MenuNode::separator());
        items.push(MenuNode::group(
            "La_yer",
            vec![
                MenuNode::leaf("_Above All", Action::Layer(LayerOp::AboveAll)),
                MenuNode::leaf("_Dock", Action::Layer(LayerOp::Dock)),
                MenuNode::leaf("_Fullscreen", Action::Layer(LayerOp::Fullscreen)),
                MenuNode::leaf("_On Top", Action::Layer(LayerOp::OnTop)),
                MenuNode::leaf("_Normal", Action::Layer(LayerOp::Normal)),
                MenuNode::leaf("_Below", Action::Layer(LayerOp::Below)),
                MenuNode::leaf("D_esktop", Action::Layer(LayerOp::Desktop)),
            ],
        ));
        items.push(MenuNode::group("_Workspace", {
            let mut ws = Vec::new();
            for i in 0..workspace_count {
                ws.push(MenuNode::leaf(
                    format!("Workspace {}", i + 1),
                    Action::Workspace(WorkspaceOp::MoveWindowTo(i)),
                ));
            }
            ws
        }));
        items.push(MenuNode::separator());
        items.push(MenuNode::leaf("_Close", Action::Window(WindowOp::Close)));
        items.push(MenuNode::leaf("_Kill", Action::Window(WindowOp::Kill)));
        items
    }

    pub fn visible(&self) -> bool {
        self.view.visible
    }

    pub fn contains_window(&self, window: u32) -> bool {
        self.view.contains_window(window)
    }

    pub fn show<H: DisplayBackend + ?Sized>(&mut self, conn: &H, pos: Point) {
        let colours = self.view.colours;
        let mut view = MenuView::with_nodes(std::mem::take(&mut self.nodes));
        view.colours = colours;
        view.show(conn, pos);
        self.view = view;
    }

    pub fn enable_filter(&mut self, rb: &std::sync::Arc<dyn RenderBackend>) {
        self.view.enable_filter(rb);
    }

    pub fn hide<H: DisplayBackend + ?Sized>(&mut self, conn: &H) {
        self.view.hide(conn);
    }

    pub fn paint<H: DisplayBackend + ?Sized>(&self, conn: &H) {
        self.view.paint(conn);
    }

    pub fn handle_event<H: DisplayBackend + ?Sized>(
        &mut self,
        conn: &H,
        event: &BackendEvent,
    ) -> MenuNav<Action> {
        self.view.handle_event(conn, event)
    }
}

#[cfg(test)]
mod tests;
