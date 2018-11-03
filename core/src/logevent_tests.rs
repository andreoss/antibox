    use super::*;
    
    

    #[test]
    fn test_filter_covers_all_variants() {
        let f = EventFilter::new_default();
        let _ = f.is_enabled(&BackendEvent::ScreenSizeChanged {
            width: 0,
            height: 0,
        });
        let _ = f.is_enabled(&BackendEvent::KeyboardChanged);
        assert!(BackendEvent::KeyboardChanged.variant_index() < EVENT_VARIANT_COUNT);
    }

    #[test]
    fn test_event_name() {
        assert_eq!(
            event_name(&BackendEvent::MapRequest { window: 0 }),
            "MapRequest"
        );
        assert_eq!(
            event_name(&BackendEvent::KeyPress {
                window: 0,
                event: 0,
                keycode: 0,
                state: 0
            }),
            "KeyPress"
        );
        assert_eq!(event_name(&BackendEvent::FocusIn { window: 0 }), "FocusIn");
    }

