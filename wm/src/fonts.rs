use antibox_core::backend::{set_element_font, set_ui_font, FontRole};

pub fn apply_fonts() {
    set_ui_font(&antibox_ui::theme::ui_font_name());
    for role in [
        FontRole::Title,
        FontRole::Menu,
        FontRole::Switch,
        FontRole::NormalTaskBar,
        FontRole::ActiveTaskBar,
        FontRole::Clock,
        FontRole::Input,
        FontRole::ToolTip,
    ] {
        set_element_font(role, "");
    }
}
