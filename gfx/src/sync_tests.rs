    use super::*;
    use std::sync::Arc;

    #[test]
    fn recovers_poisoned_mutex() {
        let m = Arc::new(Mutex::new(7));
        let m2 = Arc::clone(&m);
        let _ = std::thread::spawn(move || {
            let _g = m2.lock().unwrap();
            panic!("poison it");
        })
        .join();
        assert!(m.lock().is_err());
        assert_eq!(*m.lock_recover(), 7);
    }
