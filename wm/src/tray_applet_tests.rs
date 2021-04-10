    use super::*;
    
    use antibox_core::mock::MockDisplay;

    fn make_tray() -> TrayApplet {
        let conn = Arc::new(MockDisplay::new(800, 600, 24)) as Arc<dyn DisplayBackend>;
        TrayApplet {
            window: Box::new(antibox_core::mock::MockWindow::new(100)),
            conn: Arc::clone(&conn),
            tray_atom: 42,
            opcode_atom: 43,
            xembed_atom: 44,
            xembed_info_atom: 45,
            net_wm_name_atom: 46,
            embedded: vec![
                EmbeddedClient {
                    window: 200,
                    width: 24,
                    height: 24,
                    xembed_version: 0,
                    xembed_mapped: true,
                    title: "Client1".to_string(),
                },
                EmbeddedClient {
                    window: 201,
                    width: 24,
                    height: 24,
                    xembed_version: 0,
                    xembed_mapped: true,
                    title: "Client2".to_string(),
                },
            ],
            tray: None,
            damage_ids: Vec::new(),
            composite_available: false,
            hovered: None,
            tooltip: None,
            draw_bevel: false,
            bg: 0xD4D0C8,
            height: 28,
        }
    }

    #[test]
    fn test_new_default_state() {
        let app = make_tray();
        assert_eq!(app.embedded.len(), 2);
        assert_eq!(app.preferred_width(), 60);
        assert_eq!(
            app.preferred_height(),
            antibox_ui::metrics::panel_height() as u32
        );
    }

    #[test]
    fn test_owns_window_main() {
        let app = make_tray();
        assert!(app.owns_window(100));
        assert!(!app.owns_window(999));
    }

    #[test]
    fn test_set_geometry_resizes_slots() {
        let mut app = make_tray();
        app.set_geometry(0, 0, 60, 16);
        let icon = app.icon_size() as u32;
        let pad = app.tpad() as u32;
        let gap = app.tgap() as u32;
        assert!(icon <= 16);
        assert_eq!(app.preferred_width(), 2 * pad + 2 * icon + gap);
    }

    #[test]
    fn test_configure_request_keeps_slots() {
        let conn = Arc::new(MockDisplay::new(800, 600, 24)) as Arc<dyn DisplayBackend>;
        let mut app = make_tray();
        app.set_geometry(0, 0, 60, 16);
        let before = app.preferred_width();
        let ev = BackendEvent::ConfigureRequest {
            window: 200,
            parent: 100,
            rect: Rect::new(0, 0, 100, 100),
            border_width: 0,
            value_mask: 0xC,
        };
        app.handle_other_event(&ev, &conn);
        assert_eq!(app.embedded[0].width, 100);
        assert_eq!(app.preferred_width(), before);
    }

