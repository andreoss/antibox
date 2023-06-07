use crate::backend::{EventMask, StackMode, WindowHandle};
use crate::point::Point;
use crate::error::Result;
use std::error::Error;
use std::fmt;
use std::sync::{Arc, Mutex};

#[derive(Debug)]
pub struct MockError(String);

impl fmt::Display for MockError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Mock error: {}", self.0)
    }
}

impl Error for MockError {}

impl From<String> for MockError {
    fn from(s: String) -> MockError {
        MockError(s)
    }
}

impl From<&str> for MockError {
    fn from(s: &str) -> MockError {
        MockError(s.to_string())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Lifecycle {
    Create,
    Map,
    Unmap,
    Destroy,
}

pub type LifecycleLog = Arc<Mutex<Vec<(u32, Lifecycle)>>>;

#[derive(Debug, Clone)]
pub struct MockWindow {
    id: u32,
    mapped: Arc<Mutex<bool>>,
    geometry: Arc<Mutex<(i16, i16, u16, u16)>>,
    props: Arc<Mutex<std::collections::HashMap<u32, Vec<u8>>>>,
    lifecycle: LifecycleLog,
}

impl MockWindow {
    pub fn new(id: u32) -> MockWindow {
        Self::with_lifecycle(id, Arc::new(Mutex::new(Vec::new())))
    }

    pub fn with_lifecycle(id: u32, lifecycle: LifecycleLog) -> MockWindow {
        MockWindow {
            id,
            mapped: Arc::new(Mutex::new(false)),
            geometry: Arc::new(Mutex::new((0, 0, 0, 0))),
            props: Arc::new(Mutex::new(std::collections::HashMap::new())),
            lifecycle,
        }
    }

    fn log(&self, event: Lifecycle) {
        self.lifecycle.lock().unwrap().push((self.id, event));
    }

    pub fn get_property_data(&self, atom: u32) -> Option<Vec<u8>> {
        self.props.lock().unwrap().get(&atom).cloned()
    }

    pub fn change_property(&self, atom: u32, data: Vec<u8>) {
        self.props.lock().unwrap().insert(atom, data);
    }

    pub fn delete_property(&self, atom: u32) {
        self.props.lock().unwrap().remove(&atom);
    }
}

impl WindowHandle for MockWindow {
    fn id(&self) -> u32 {
        self.id
    }

    fn map(&self) -> Result<()> {
        *self.mapped.lock().unwrap() = true;
        self.log(Lifecycle::Map);
        Ok(())
    }

    fn unmap(&self) -> Result<()> {
        *self.mapped.lock().unwrap() = false;
        self.log(Lifecycle::Unmap);
        Ok(())
    }

    fn destroy(&self) -> Result<()> {
        self.log(Lifecycle::Destroy);
        Ok(())
    }

    fn configure(
        &self,
        x: Option<i32>,
        y: Option<i32>,
        w: Option<u16>,
        h: Option<u16>,
    ) -> Result<()> {
        let mut geo = self.geometry.lock().unwrap();
        geo.0 = x.unwrap_or(geo.0 as i32) as i16;
        geo.1 = y.unwrap_or(geo.1 as i32) as i16;
        if let Some(wv) = w {
            geo.2 = wv;
        }
        if let Some(hv) = h {
            geo.3 = hv;
        }
        Ok(())
    }

    fn raise(&self) -> Result<()> {
        Ok(())
    }

    fn lower(&self) -> Result<()> {
        Ok(())
    }

    fn reparent(&self, _parent: u32, _point: Point) -> Result<()> {
        Ok(())
    }

    fn set_title(&self, _title: &str) -> Result<()> {
        Ok(())
    }

    fn set_class(&self, _instance: &str, _class: &str) -> Result<()> {
        Ok(())
    }

    fn select_input(&self, _event_mask: EventMask) -> Result<()> {
        Ok(())
    }

    fn get_property(
        &self,
        atom: u32,
        _offset: u32,
        _length: u32,
    ) -> Result<Option<Vec<u8>>> {
        Ok(self.props.lock().unwrap().get(&atom).cloned())
    }

    fn get_geometry(&self) -> Result<(u16, u16)> {
        let geo = self.geometry.lock().unwrap();
        Ok((geo.2, geo.3))
    }

    fn move_window(&self, point: Point) -> Result<()> {
        let mut geo = self.geometry.lock().unwrap();
        geo.0 = point.x as i16;
        geo.1 = point.y as i16;
        Ok(())
    }

    fn resize(&self, w: u16, h: u16) -> Result<()> {
        let mut geo = self.geometry.lock().unwrap();
        geo.2 = w;
        geo.3 = h;
        Ok(())
    }

    fn restack(&self, _sibling: Option<u32>, _mode: StackMode) -> Result<()> {
        Ok(())
    }

    fn translate_coords(&self, point: Point) -> Result<Point> {
        Ok(Point::new(point.x, point.y))
    }
}

#[cfg(test)]
#[path = "window_tests.rs"]
mod tests;
