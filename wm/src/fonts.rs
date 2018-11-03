use antibox_core::backend::{set_element_font, set_ui_font, FontRole};

pub fn apply_fonts() {
    set_ui_font(antibox_ui::theme::ui_font_name());
    let themed = antibox_ui::theme::chrome_font();
    set_element_font(FontRole::Title, themed);
    set_element_font(FontRole::Menu, themed);
    set_element_font(FontRole::Switch, "");
    set_element_font(FontRole::NormalTaskBar, "");
    set_element_font(FontRole::ActiveTaskBar, "");
    set_element_font(FontRole::Clock, "");
    set_element_font(FontRole::Input, "");
    set_element_font(FontRole::ToolTip, "");
}
