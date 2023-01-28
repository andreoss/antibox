    use super::*;

    #[test]
    fn test_load_average_non_negative() {
        let avgs = load_average().unwrap();
        assert!(avgs.iter().all(|a| *a >= 0.0));
    }

    #[test]
    fn test_read_proc_meminfo() {
        if let Some((total, free)) = read_proc_meminfo() {
            assert!(total > 0);
            assert!(free <= total);
        }
    }

    #[test]
    #[cfg(not(target_os = "linux"))]
    fn test_read_uvmexp() {
        let m = read_uvmexp().unwrap();
        assert!(m.total > 0);
        assert!(m.free <= m.total);
    }

    #[test]
    #[cfg(not(target_os = "linux"))]
    fn test_read_cptime() {
        assert!(read_cptime().unwrap().iter().any(|v| *v > 0));
    }

    #[test]
    fn test_pid_command_zero_is_none() {
        assert!(pid_command(0).is_none());
    }

    #[test]
    #[cfg(target_os = "linux")]
    fn test_pid_command_reads_own_cmdline() {
        let pid = std::process::id();
        let cmd = pid_command(pid).expect("own cmdline readable");
        assert!(!cmd.is_empty());
        assert!(!cmd.contains('\0'));
    }
