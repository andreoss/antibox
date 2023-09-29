#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum WindowOp {
    Close,
    Kill,
    Move,
    Resize,
    Raise,
    Lower,
    Depth,
    Maximize,
    MaximizeVert,
    MaximizeHoriz,
    Minimize,
    Restore,
    Fullscreen,
    Shade,
    Hide,
    Rollup,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TileOp {
    Cascade,
    Tile,
    Arrange,
    UndoArrange,
    TileLeft,
    TileRight,
    TileTop,
    TileBottom,
    TileTopLeft,
    TileTopRight,
    TileBottomLeft,
    TileBottomRight,
    SnapLeft,
    SnapRight,
    SnapUp,
    SnapDown,
    TileCenter,
    TileVertical,
    TileHorizontal,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum WorkspaceOp {
    NextWorkspace,
    PrevWorkspace,
    Workspace(u32),
    WorkspaceNextTaken,
    WorkspacePrevTaken,
    WorkspaceNextTakeWin,
    WorkspacePrevTakeWin,
    MoveWindowTo(u32),
    MinimizeAll,
    HideAll,
    ShowDesktop,
    OccupyAllOrCurrent,
    NextLayout,
    SetLayout(u32, u8),
    WorkspaceMenu(u32),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum FocusOp {
    Next,
    Prev,
    ClickToFocus,
    Explicit,
    MouseSloppy,
    MouseStrict,
    QuietSloppy,
    Custom,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum LayerOp {
    Layer(i32),
    AboveAll,
    Dock,
    Fullscreen,
    Menu,
    Normal,
    OnTop,
    Below,
    Desktop,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TabOp {
    Untab,
    Next,
    Prev,
    JoinWindow(u32),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum MenuOp {
    RootMenu,
    WindowPickerList,
    WindowActionMenu,
    Pager,
    Omni,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum MiscOp {
    Command(String),
    Show,
    WinOptions,
    ReloadKeys,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Action {
    Window(WindowOp),
    Tile(TileOp),
    Workspace(WorkspaceOp),
    Focus(FocusOp),
    Layer(LayerOp),
    Tab(TabOp),
    Menu(MenuOp),
    Misc(MiscOp),
}

fn window_name(op: &WindowOp) -> &'static str {
    match op {
        WindowOp::Close => "Close",
        WindowOp::Kill => "Kill",
        WindowOp::Move => "Move",
        WindowOp::Resize => "Resize",
        WindowOp::Raise => "Raise",
        WindowOp::Lower => "Lower",
        WindowOp::Depth => "Depth",
        WindowOp::Maximize => "Maximize",
        WindowOp::MaximizeVert => "Maximize Vert",
        WindowOp::MaximizeHoriz => "Maximize Horiz",
        WindowOp::Minimize => "Minimize",
        WindowOp::Restore => "Restore",
        WindowOp::Fullscreen => "Fullscreen",
        WindowOp::Shade => "Shade",
        WindowOp::Hide => "Hide",
        WindowOp::Rollup => "Rollup",
    }
}

fn tile_name(op: &TileOp) -> &'static str {
    match op {
        TileOp::Cascade => "Cascade",
        TileOp::Tile => "Tile",
        TileOp::Arrange => "Arrange",
        TileOp::UndoArrange => "Undo Arrange",
        TileOp::TileLeft => "Tile Left",
        TileOp::TileRight => "Tile Right",
        TileOp::TileTop => "Tile Top",
        TileOp::TileBottom => "Tile Bottom",
        TileOp::TileTopLeft => "Tile Top-Left",
        TileOp::TileTopRight => "Tile Top-Right",
        TileOp::TileBottomLeft => "Tile Bottom-Left",
        TileOp::TileBottomRight => "Tile Bottom-Right",
        TileOp::SnapLeft => "Snap Left",
        TileOp::SnapRight => "Snap Right",
        TileOp::SnapUp => "Snap Up",
        TileOp::SnapDown => "Snap Down",
        TileOp::TileCenter => "Tile Center",
        TileOp::TileVertical => "Tile Vertical",
        TileOp::TileHorizontal => "Tile Horizontal",
    }
}

fn workspace_name(op: &WorkspaceOp) -> &'static str {
    match op {
        WorkspaceOp::NextWorkspace => "Next Workspace",
        WorkspaceOp::PrevWorkspace => "Prev Workspace",
        WorkspaceOp::Workspace(_) => "Switch Workspace",
        WorkspaceOp::WorkspaceNextTaken => "Next Workspace With Windows",
        WorkspaceOp::WorkspacePrevTaken => "Prev Workspace With Windows",
        WorkspaceOp::WorkspaceNextTakeWin => "Next Workspace, Take Window",
        WorkspaceOp::WorkspacePrevTakeWin => "Prev Workspace, Take Window",
        WorkspaceOp::MoveWindowTo(_) => "Move Window To Workspace",
        WorkspaceOp::MinimizeAll => "Minimize All",
        WorkspaceOp::HideAll => "Hide All",
        WorkspaceOp::ShowDesktop => "Show Desktop",
        WorkspaceOp::OccupyAllOrCurrent => "Occupy All/Current",
        WorkspaceOp::NextLayout => "Next Layout",
        WorkspaceOp::SetLayout(..) => "Set Layout",
        WorkspaceOp::WorkspaceMenu(_) => "Workspace Menu",
    }
}

fn focus_name(op: &FocusOp) -> &'static str {
    match op {
        FocusOp::Next => "Focus Next",
        FocusOp::Prev => "Focus Previous",
        FocusOp::ClickToFocus => "Click To Focus",
        FocusOp::Explicit => "Explicit Focus",
        FocusOp::MouseSloppy => "Sloppy Focus",
        FocusOp::MouseStrict => "Strict Focus",
        FocusOp::QuietSloppy => "Quiet Focus",
        FocusOp::Custom => "Custom Focus",
    }
}

fn layer_name(op: &LayerOp) -> &'static str {
    match op {
        LayerOp::Layer(_) => "Layer",
        LayerOp::AboveAll => "Layer Above All",
        LayerOp::Dock => "Layer Dock",
        LayerOp::Fullscreen => "Layer Fullscreen",
        LayerOp::Menu => "Layer Menu",
        LayerOp::Normal => "Layer Normal",
        LayerOp::OnTop => "Layer On Top",
        LayerOp::Below => "Layer Below",
        LayerOp::Desktop => "Layer Desktop",
    }
}

fn menu_name(op: &MenuOp) -> &'static str {
    match op {
        MenuOp::WindowPickerList => "Window List",
        MenuOp::WindowActionMenu => "Window Menu",
        MenuOp::RootMenu => "RootMenu",
        MenuOp::Pager => "Pager",
        MenuOp::Omni => "Omni",
    }
}

fn misc_name(op: &MiscOp) -> &'static str {
    match op {
        MiscOp::Command(_) => "Command",
        MiscOp::Show => "Show",
        MiscOp::WinOptions => "WinOptions",
        MiscOp::ReloadKeys => "Reload Keys",
    }
}

pub fn action_name(action: &Action) -> &'static str {
    match action {
        Action::Window(op) => window_name(op),
        Action::Tile(op) => tile_name(op),
        Action::Workspace(op) => workspace_name(op),
        Action::Focus(op) => focus_name(op),
        Action::Layer(op) => layer_name(op),
        Action::Tab(op) => match op {
            TabOp::Untab => "Untab",
            TabOp::Next => "Tab Next",
            TabOp::Prev => "Tab Prev",
            TabOp::JoinWindow(_) => "Join Window",
        },
        Action::Menu(op) => menu_name(op),
        Action::Misc(op) => misc_name(op),
    }
}

#[cfg(test)]
#[path = "action_tests.rs"]
mod tests;
