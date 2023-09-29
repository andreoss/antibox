use super::*;
use antibox_core::backend::RenderBackend;
use antibox_core::mock::{MockDisplay, MockWindow};
use antibox_core::rect::Rect;

fn mk() -> FrameWindow {
    let _display = MockDisplay::new(1280, 720, 24);
    FrameWindow::new(
        ClientWindow::new(Box::new(MockWindow::new(1))),
        Box::new(MockWindow::new(2)),
    )
}

#[test]
fn test_create_frame_decoration_insets() {
    let d = MockDisplay::new(1280, 720, 24);
    let client = d
        .create_window(
            1,
            Rect::new(0, 0, 1280, 30),
            WmWindowClass::InputOutput,
            false,
            EventMask::NO_EVENT,
        )
        .unwrap();
    let (_f, fr) =
        FrameWindow::create_frame_ex(&d, client.id(), Rect::new(0, 0, 1280, 30), false, 0xD4D0C8)
            .unwrap();
    assert_eq!(fr, Rect::new(0, 0, 1280, 30));

    let c2 = d
        .create_window(
            1,
            Rect::new(0, 0, 300, 200),
            WmWindowClass::InputOutput,
            false,
            EventMask::NO_EVENT,
        )
        .unwrap();
    let (_f2, fr2) =
        FrameWindow::create_frame_ex(&d, c2.id(), Rect::new(100, 100, 300, 200), true, 0xD4D0C8)
            .unwrap();
    assert_eq!(
        fr2,
        Rect::new(
            100 - border_width(),
            100 - title_bar_height() - border_width(),
            300 + border_width() * 2,
            200 + title_bar_height() + border_width() * 2
        )
    );
}

#[test]
fn test_frame_actions() {
    let mut fw = mk();
    assert_eq!(fw.workspace(), 0);
    assert!(!fw.state().maximized);
    fw.maximize();
    assert!(fw.state().maximized);
    fw.minimize();
    assert!(fw.state().minimized);
    fw.set_workspace(2);
    assert_eq!(fw.workspace(), 2);
}

#[test]
fn test_mwm_functions_hide_minimize_and_maximize_buttons() {
    use antibox_core::backend::hints::{mwm_func, mwm_hints_flags, MwmHints};
    let mut c = ClientWindow::new(Box::new(MockWindow::new(1)));
    c.mwm_hints = Some(MwmHints {
        flags: mwm_hints_flags::FUNCTIONS,
        functions: mwm_func::ALL | mwm_func::MAXIMIZE | mwm_func::MINIMIZE,
        decorations: 0,
        input_mode: 0,
    });
    let fw = FrameWindow::new(c, Box::new(MockWindow::new(2)));
    assert!(fw.title_button_for('i').is_none());
    assert!(fw.title_button_for('m').is_none());
    assert!(fw.title_button_for('x').is_some());
    let plain = mk();
    assert!(plain.title_button_for('i').is_some());
    assert!(plain.title_button_for('m').is_some());
}

