use crate::action::*;

pub fn keysym_from_name(name: &str) -> Option<u32> {
    let table: &[(&str, u32)] = &[
        ("F1", 0xFFBE),
        ("F2", 0xFFBF),
        ("F3", 0xFFC0),
        ("F4", 0xFFC1),
        ("F5", 0xFFC2),
        ("F6", 0xFFC3),
        ("F7", 0xFFC4),
        ("F8", 0xFFC5),
        ("F9", 0xFFC6),
        ("F10", 0xFFC7),
        ("F11", 0xFFC8),
        ("F12", 0xFFC9),
        ("F13", 0xFFCA),
        ("F14", 0xFFCB),
        ("F15", 0xFFCC),
        ("Left", 0xFF51),
        ("Right", 0xFF53),
        ("Up", 0xFF52),
        ("Down", 0xFF54),
        ("Prior", 0xFF55),
        ("Next", 0xFF56),
        ("Page_Up", 0xFF55),
        ("Page_Down", 0xFF56),
        ("Home", 0xFF50),
        ("End", 0xFF57),
        ("Begin", 0xFF58),
        ("KP_Up", 0xFF97),
        ("KP_Down", 0xFF99),
        ("KP_Left", 0xFF96),
        ("KP_Right", 0xFF98),
        ("KP_Prior", 0xFF9A),
        ("KP_Next", 0xFF9B),
        ("KP_Home", 0xFF95),
        ("KP_End", 0xFF9C),
        ("KP_Begin", 0xFF9D),
        ("KP_Insert", 0xFF9E),
        ("KP_Delete", 0xFF9F),
        ("KP_Enter", 0xFF8D),
        ("KP_Add", 0xFFAB),
        ("KP_Subtract", 0xFFAD),
        ("KP_Multiply", 0xFFAA),
        ("KP_Divide", 0xFFAF),
        ("Escape", 0xFF1B),
        ("Esc", 0xFF1B),
        ("Tab", 0xFF09),
        ("Return", 0xFF0D),
        ("Enter", 0xFF0D),
        ("Space", 0x0020),
        ("BackSpace", 0xFF08),
        ("BackSp", 0xFF08),
        ("Delete", 0xFFFF),
        ("Del", 0xFFFF),
        ("Insert", 0xFF63),
        ("Menu", 0xFF67),
        ("Print", 0xFF61),
        ("Pause", 0xFF13),
        ("Pointer_Button1", 0x010014),
        ("Pointer_Button2", 0x010015),
        ("Pointer_Button3", 0x010016),
        ("Pointer_Button4", 0x010017),
        ("Pointer_Button5", 0x010018),
    ];
    if let Some(sym) = table
        .iter()
        .find(|(k, _)| k.eq_ignore_ascii_case(name))
        .map(|(_, v)| *v)
    {
        return Some(sym);
    }
    let mut chars = name.chars();
    match (chars.next(), chars.next()) {
        (Some(c), None) if c.is_ascii_graphic() => Some(c.to_ascii_lowercase() as u32),
        _ => None,
    }
}

pub fn parse_modifiers(s: &str) -> (u16, &str) {
    const PREFIXES: &[(&str, u16)] = &[
        ("Ctrl+", 0x04),
        ("Control+", 0x04),
        ("Alt+", 0x08),
        ("Shift+", 0x01),
        ("Super+", 0x40),
        ("Meta+", 0x80),
        ("Hyper+", 0x100),
        ("Mod1+", 0x08),
        ("Mod4+", 0x40),
    ];
    let mut mods = 0u16;
    let mut rest = s;
    loop {
        let original = rest;
        for &(p, m) in PREFIXES {
            if rest.len() >= p.len() && rest[..p.len()].eq_ignore_ascii_case(p) {
                mods |= m;
                rest = &rest[p.len()..];
                break;
            }
        }
        if rest == original {
            break;
        }
    }
    (mods, rest)
}

pub fn parse_action(name: &str) -> Option<Action> {
    let table: &[(&str, Action)] = &[
        ("Arrange", Action::Tile(TileOp::Arrange)),
        ("Cascade", Action::Tile(TileOp::Cascade)),
        ("Tile", Action::Tile(TileOp::Tile)),
        ("Close", Action::Window(WindowOp::Close)),
        ("Depth", Action::Window(WindowOp::Depth)),
        ("Fullscreen", Action::Window(WindowOp::Fullscreen)),
        ("Hide", Action::Window(WindowOp::Hide)),
        ("HideAll", Action::Workspace(WorkspaceOp::HideAll)),
        ("Kill", Action::Window(WindowOp::Kill)),
        ("Lower", Action::Window(WindowOp::Lower)),
        ("Maximize", Action::Window(WindowOp::Maximize)),
        ("Minimize", Action::Window(WindowOp::Minimize)),
        ("MinimizeAll", Action::Workspace(WorkspaceOp::MinimizeAll)),
        ("Move", Action::Window(WindowOp::Move)),
        ("Raise", Action::Window(WindowOp::Raise)),
        ("NextLayout", Action::Workspace(WorkspaceOp::NextLayout)),
        ("ReloadKeys", Action::Misc(MiscOp::ReloadKeys)),
        ("Resize", Action::Window(WindowOp::Resize)),
        ("Restore", Action::Window(WindowOp::Restore)),
        ("Rollup", Action::Window(WindowOp::Rollup)),
        ("Shade", Action::Window(WindowOp::Shade)),
        ("Show", Action::Misc(MiscOp::Show)),
        ("ShowDesktop", Action::Workspace(WorkspaceOp::ShowDesktop)),
        ("TileLeft", Action::Tile(TileOp::TileLeft)),
        ("TileRight", Action::Tile(TileOp::TileRight)),
        ("TileTop", Action::Tile(TileOp::TileTop)),
        ("TileBottom", Action::Tile(TileOp::TileBottom)),
        ("TileTopLeft", Action::Tile(TileOp::TileTopLeft)),
        ("TileTopRight", Action::Tile(TileOp::TileTopRight)),
        ("TileBottomLeft", Action::Tile(TileOp::TileBottomLeft)),
        ("TileBottomRight", Action::Tile(TileOp::TileBottomRight)),
        ("SnapLeft", Action::Tile(TileOp::SnapLeft)),
        ("SnapRight", Action::Tile(TileOp::SnapRight)),
        ("SnapUp", Action::Tile(TileOp::SnapUp)),
        ("SnapDown", Action::Tile(TileOp::SnapDown)),
        ("TileCenter", Action::Tile(TileOp::TileCenter)),
        ("TileVertical", Action::Tile(TileOp::TileVertical)),
        ("TileHorizontal", Action::Tile(TileOp::TileHorizontal)),
        ("UndoArrange", Action::Tile(TileOp::UndoArrange)),
        ("Untab", Action::Tab(TabOp::Untab)),
        ("TabNext", Action::Tab(TabOp::Next)),
        ("TabPrev", Action::Tab(TabOp::Prev)),
        ("WindowActionMenu", Action::Menu(MenuOp::WindowActionMenu)),
        ("WindowPickerList", Action::Menu(MenuOp::WindowPickerList)),
        ("WinOptions", Action::Misc(MiscOp::WinOptions)),
        (
            "NextWorkspace",
            Action::Workspace(WorkspaceOp::NextWorkspace),
        ),
        (
            "PrevWorkspace",
            Action::Workspace(WorkspaceOp::PrevWorkspace),
        ),
        ("FocusNext", Action::Focus(FocusOp::Next)),
        ("FocusPrev", Action::Focus(FocusOp::Prev)),
        (
            "WorkspaceNextTaken",
            Action::Workspace(WorkspaceOp::WorkspaceNextTaken),
        ),
        (
            "WorkspacePrevTaken",
            Action::Workspace(WorkspaceOp::WorkspacePrevTaken),
        ),
        (
            "WorkspaceNextTakeWin",
            Action::Workspace(WorkspaceOp::WorkspaceNextTakeWin),
        ),
        (
            "WorkspacePrevTakeWin",
            Action::Workspace(WorkspaceOp::WorkspacePrevTakeWin),
        ),
        ("Pager", Action::Menu(MenuOp::Pager)),
        ("Omni", Action::Menu(MenuOp::Omni)),
        ("ClickToFocus", Action::Focus(FocusOp::ClickToFocus)),
        ("Explicit", Action::Focus(FocusOp::Explicit)),
        ("MouseSloppy", Action::Focus(FocusOp::MouseSloppy)),
        ("MouseStrict", Action::Focus(FocusOp::MouseStrict)),
        ("QuietSloppy", Action::Focus(FocusOp::QuietSloppy)),
        ("Custom", Action::Focus(FocusOp::Custom)),
        (
            "OccupyAllOrCurrent",
            Action::Workspace(WorkspaceOp::OccupyAllOrCurrent),
        ),
        ("LayerAboveAll", Action::Layer(LayerOp::AboveAll)),
        ("LayerDock", Action::Layer(LayerOp::Dock)),
        ("LayerFullscreen", Action::Layer(LayerOp::Fullscreen)),
        ("LayerMenu", Action::Layer(LayerOp::Menu)),
        ("LayerNormal", Action::Layer(LayerOp::Normal)),
        ("LayerOnTop", Action::Layer(LayerOp::OnTop)),
        ("LayerBelow", Action::Layer(LayerOp::Below)),
        ("LayerDesktop", Action::Layer(LayerOp::Desktop)),
        ("MaximizeVert", Action::Window(WindowOp::MaximizeVert)),
        ("MaximizeHoriz", Action::Window(WindowOp::MaximizeHoriz)),
    ];
    if let Some((_, action)) = table.iter().find(|(k, _)| *k == name) {
        return Some(action.clone());
    }
    if let Some(rest) = name.strip_prefix("Exec ") {
        let cmd = rest.trim();
        if !cmd.is_empty() {
            return Some(Action::Misc(MiscOp::Command(cmd.to_string())));
        }
    }
    if let Some(n) = name.strip_prefix("Workspace") { if let Ok(num) = n.trim().parse::<u32>() {
            return Some(Action::Workspace(WorkspaceOp::Workspace(
                num.wrapping_sub(1),
            )));
        }
    }
    if let Some(n) = name.strip_prefix("Layer") { if let Ok(num) = n.parse::<i32>() {
            return Some(Action::Layer(LayerOp::Layer(num)));
        }
    }
    None
}

pub struct KeyEntry {
    pub keysym: u32,
    pub modifiers: u16,
    pub action: Action,
}

pub fn entries_from(keys: &[(String, String)]) -> Vec<KeyEntry> {
    keys.iter()
        .filter_map(|(combo, action)| parse_key_binding(combo, action))
        .collect()
}

pub fn parse_key_binding(combo: &str, action: &str) -> Option<KeyEntry> {
    let action = action.trim();
    if action.is_empty() {
        return None;
    }
    let (modifiers, key_name) = parse_modifiers(combo.trim());
    let keysym = keysym_from_name(key_name)?;
    let action = parse_action(action)?;
    Some(KeyEntry {
        keysym,
        modifiers,
        action,
    })
}

#[cfg(test)]
#[path = "keys_parser_tests.rs"]
mod tests;
