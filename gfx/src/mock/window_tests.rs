    use super::*;

    #[test]
    fn with_lifecycle_shares_log() {
        let log: LifecycleLog = Arc::new(Mutex::new(Vec::new()));
        let window = MockWindow::with_lifecycle(5, log.clone());
        window.map().unwrap();
        window.unmap().unwrap();
        window.destroy().unwrap();
        assert_eq!(
            *log.lock().unwrap(),
            vec![
                (5, Lifecycle::Map),
                (5, Lifecycle::Unmap),
                (5, Lifecycle::Destroy)
            ]
        );
    }

    #[test]
    fn new_keeps_private_log() {
        let window = MockWindow::new(3);
        window.map().unwrap();
        assert_eq!(*window.lifecycle.lock().unwrap(), vec![(3, Lifecycle::Map)]);
    }
