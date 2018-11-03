pub struct WindowOptions {
    opts: Vec<WindowOption>,
}
impl Default for WindowOptions {
    fn default() -> WindowOptions {
        Self::new()
    }
}
impl WindowOptions {
    pub fn new() -> WindowOptions {
        WindowOptions { opts: Vec::new() }
    }
    pub fn is_empty(&self) -> bool {
        self.opts.is_empty()
    }
    pub fn len(&self) -> usize {
        self.opts.len()
    }
    pub fn get(&self, i: usize) -> Option<&WindowOption> {
        self.opts.get(i)
    }
    pub fn find(&self, class_instance: &str) -> (bool, usize) {
        let (mut lo, mut hi) = (0usize, self.opts.len());
        while lo < hi {
            let mid = lo + (hi - lo) / 2;
            match class_instance.cmp(&self.opts[mid].class_instance) {
                std::cmp::Ordering::Greater => lo = mid + 1,
                std::cmp::Ordering::Less => hi = mid,
                std::cmp::Ordering::Equal => return (true, mid),
            }
        }
        (false, lo)
    }
    pub fn get_or_create(&mut self, class_instance: &str) -> &mut WindowOption {
        let (found, idx) = self.find(class_instance);
        if !found {
            self.opts.insert(idx, WindowOption::new(class_instance));
        }
        &mut self.opts[idx]
    }
    pub fn set_win_option(&mut self, class_instance: &str, opt: &str, arg: &str) {
        let op = self.get_or_create(class_instance);
        match opt {
            "workspace" => {
                op.placement.workspace = arg.parse::<i32>().ok().filter(|ws| *ws >= 0);
            }
            "opacity" => {
                if let Ok(v) = arg.parse::<i32>() {
                    if crate::compat::in_range(v, 0, 100) {
                        op.opacity = v;
                    }
                }
            }
            "geometry" => {
                let gf = parsing::parse_geometry(arg);
                op.geom.gflags = gf.0;
                op.geom.gx = gf.1;
                op.geom.gy = gf.2;
                op.geom.gw = gf.3;
                op.geom.gh = gf.4;
            }
            "layer" => op.placement.layer = parsing::parse_layer(arg),
            _ => {
                if let Some(flag) = parsing::lookup_option_flag(opt) {
                    if arg == "0" {
                        op.options = op.options & !flag;
                    } else {
                        op.options |= flag;
                    }
                    op.option_mask |= flag;
                }
            }
        }
    }
}
