use crate::backend::{FontSpec, GraphicsContext, PixmapData};
use crate::rect::Rect;
use crate::error::Result;
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone, Default)]
pub struct MockGraphics {
    drawable: u32,
    fg_pixel: Arc<Mutex<u32>>,
    bg_pixel: Arc<Mutex<u32>>,
    pub commands: Arc<Mutex<Vec<MockCommand>>>,
    current_font_size: Arc<Mutex<u16>>,
    text_width_calls: Arc<Mutex<usize>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MockCommand {
    SetForeground(u32),
    SetBackground(u32),
    FillRect(i16, i16, u16, u16),
    DrawRect(i16, i16, u16, u16),
    DrawText(i16, i16, String),
    DrawLine(i16, i16, i16, i16),
    ClearRect(i16, i16, u16, u16),
    SetFont(String, u16),
    DrawString(i16, i16, String),
    Draw3dRect(i16, i16, u16, u16, bool),
    DrawPixmap(i16, i16),
    DrawPoint(i16, i16),
    DrawArc(i16, i16, u16, u16, i16, i16),
    FillArc(i16, i16, u16, u16, i16, i16),
    FillPolygon(Vec<(i16, i16)>),
    CopyArea(i16, i16, u16, u16, i16, i16),
    DrawImage(i16, i16, u16, u16, Vec<u8>),
    CompositePixmap(u32, u16, u16, i16, i16, i16, i16),
}

impl MockGraphics {
    pub fn new(drawable: u32) -> Self {
        Self {
            drawable,
            fg_pixel: Arc::new(Mutex::new(0)),
            bg_pixel: Arc::new(Mutex::new(0)),
            commands: Arc::new(Mutex::new(Vec::new())),
            current_font_size: Arc::new(Mutex::new(12)),
            text_width_calls: Arc::new(Mutex::new(0)),
        }
    }

    pub fn text_width_calls(&self) -> usize {
        *self.text_width_calls.lock().unwrap()
    }

    pub fn commands(&self) -> Vec<MockCommand> {
        self.commands.lock().unwrap().clone()
    }

    pub fn assert_ordered(&self, expected: &[MockCommand]) {
        assert_ordered(&self.commands(), expected);
    }

    pub fn assert_painted(&self, region: &Rect) {
        assert_painted(&self.commands(), region);
    }

    pub fn assert_colour_at(&self, x: i16, y: i16, colour: crate::colour::Colour) {
        assert_colour_at(&self.commands(), x, y, colour);
    }

    pub fn colour_at(&self, x: i16, y: i16) -> Option<u32> {
        colour_at(&self.commands(), x, y)
    }
}

const fn covers(x: i16, y: i16, w: u16, h: u16, px: i32, py: i32) -> bool {
    px >= x as i32 && px < x as i32 + w as i32 && py >= y as i32 && py < y as i32 + h as i32
}

fn on_line(x1: i16, y1: i16, x2: i16, y2: i16, x: i16, y: i16) -> bool {
    if y1 == y2 {
        y == y1 && x >= x1.min(x2) && x <= x1.max(x2)
    } else if x1 == x2 {
        x == x1 && y >= y1.min(y2) && y <= y1.max(y2)
    } else {
        false
    }
}

const fn on_outline(rx: i16, ry: i16, w: u16, h: u16, x: i16, y: i16) -> bool {
    let left = rx as i32;
    let top = ry as i32;
    let right = left + w as i32;
    let bottom = top + h as i32;
    let x = x as i32;
    let y = y as i32;
    let inside_x = x >= left && x <= right;
    let inside_y = y >= top && y <= bottom;
    ((x == left || x == right) && inside_y) || ((y == top || y == bottom) && inside_x)
}

const fn paint_bounds(command: &MockCommand) -> Option<(i16, i16, u16, u16)> {
    match command {
        MockCommand::FillRect(x, y, w, h) | MockCommand::ClearRect(x, y, w, h) => {
            Some((*x, *y, *w, *h))
        }
        MockCommand::DrawImage(x, y, w, h, _) => Some((*x, *y, *w, *h)),
        MockCommand::CopyArea(_, _, w, h, dx, dy)
        | MockCommand::CompositePixmap(_, w, h, dx, dy, _, _) => Some((*dx, *dy, *w, *h)),
        _ => None,
    }
}

pub fn assert_ordered(commands: &[MockCommand], expected: &[MockCommand]) {
    let mut matched = 0;
    for command in commands {
        if matched < expected.len() && *command == expected[matched] {
            matched += 1;
        }
    }
    assert_eq!(matched, expected.len());
}

pub fn assert_painted(commands: &[MockCommand], region: &Rect) {
    let bounds: Vec<(i16, i16, u16, u16)> = commands.iter().filter_map(paint_bounds).collect();
    for py in region.y..region.y + region.h {
        for px in region.x..region.x + region.w {
            let covered = bounds
                .iter()
                .any(|&(x, y, w, h)| covers(x, y, w, h, px, py));
            assert!(covered);
        }
    }
}

pub fn colour_at(commands: &[MockCommand], x: i16, y: i16) -> Option<u32> {
    let mut fg = None;
    let mut bg = None;
    let mut colour = None;
    for command in commands {
        match command {
            MockCommand::SetForeground(pixel) => fg = Some(*pixel),
            MockCommand::SetBackground(pixel) => bg = Some(*pixel),
            MockCommand::FillRect(rx, ry, w, h) if covers(*rx, *ry, *w, *h, x as i32, y as i32) => {
                colour = fg;
            }
            MockCommand::ClearRect(rx, ry, w, h)
                if covers(*rx, *ry, *w, *h, x as i32, y as i32) =>
            {
                colour = bg;
            }
            MockCommand::DrawRect(rx, ry, w, h) if on_outline(*rx, *ry, *w, *h, x, y) => {
                colour = fg;
            }
            MockCommand::DrawLine(x1, y1, x2, y2) if on_line(*x1, *y1, *x2, *y2, x, y) => {
                colour = fg;
            }
            MockCommand::DrawPoint(px, py) if *px == x && *py == y => colour = fg,
            _ => {}
        }
    }
    colour
}

pub fn assert_colour_at(commands: &[MockCommand], x: i16, y: i16, colour: crate::colour::Colour) {
    let found = colour_at(commands, x, y);
    assert_eq!(found, Some(colour));
}

impl GraphicsContext for MockGraphics {
    fn set_foreground(&self, pixel: u32) -> Result<()> {
        *self.fg_pixel.lock().unwrap() = pixel;
        self.commands
            .lock()
            .unwrap()
            .push(MockCommand::SetForeground(pixel));
        Ok(())
    }

    fn set_background(&self, pixel: u32) -> Result<()> {
        *self.bg_pixel.lock().unwrap() = pixel;
        self.commands
            .lock()
            .unwrap()
            .push(MockCommand::SetBackground(pixel));
        Ok(())
    }

    fn fill_rect(&self, x: i16, y: i16, w: u16, h: u16) -> Result<()> {
        self.commands
            .lock()
            .unwrap()
            .push(MockCommand::FillRect(x, y, w, h));
        Ok(())
    }

    fn draw_rect(&self, x: i16, y: i16, w: u16, h: u16) -> Result<()> {
        self.commands
            .lock()
            .unwrap()
            .push(MockCommand::DrawRect(x, y, w, h));
        Ok(())
    }

    fn draw_text(&self, x: i16, y: i16, text: &str) -> Result<()> {
        self.commands
            .lock()
            .unwrap()
            .push(MockCommand::DrawText(x, y, text.to_string()));
        Ok(())
    }

    fn draw_line(&self, x1: i16, y1: i16, x2: i16, y2: i16) -> Result<()> {
        self.commands
            .lock()
            .unwrap()
            .push(MockCommand::DrawLine(x1, y1, x2, y2));
        Ok(())
    }

    fn clear_rect(&self, rect: &Rect) -> Result<()> {
        self.commands.lock().unwrap().push(MockCommand::ClearRect(
            rect.x as i16,
            rect.y as i16,
            rect.w as u16,
            rect.h as u16,
        ));
        Ok(())
    }

    fn set_font(&self, font: &FontSpec) -> Result<()> {
        *self.current_font_size.lock().unwrap() = font.size;
        self.commands
            .lock()
            .unwrap()
            .push(MockCommand::SetFont(font.family.clone(), font.size));
        Ok(())
    }

    fn text_width(&self, text: &str) -> Result<u32> {
        *self.text_width_calls.lock().unwrap() += 1;
        let size = *self.current_font_size.lock().unwrap();
        Ok((text.len() as u32) * size as u32 * 55 / 100)
    }

    fn font_metrics(&self) -> (u16, u16, u16) {
        let size = *self.current_font_size.lock().unwrap();
        let ascent = (size * 3 / 4).max(1);
        let descent = (size - ascent).max(1);
        (ascent, descent, size)
    }

    fn drawable(&self) -> u32 {
        self.drawable
    }

    fn draw_string(&self, x: i16, y: i16, text: &str) -> Result<()> {
        self.commands
            .lock()
            .unwrap()
            .push(MockCommand::DrawString(x, y, text.to_string()));
        Ok(())
    }

    fn fill_polygon(&self, points: &[(i16, i16)]) -> Result<()> {
        self.commands
            .lock()
            .unwrap()
            .push(MockCommand::FillPolygon(points.to_vec()));
        Ok(())
    }

    fn draw_image(
        &self,
        x: i16,
        y: i16,
        w: u16,
        h: u16,
        data: &[u8],
    ) -> Result<()> {
        self.commands
            .lock()
            .unwrap()
            .push(MockCommand::DrawImage(x, y, w, h, data.to_vec()));
        Ok(())
    }

    fn copy_area(
        &self,
        x: i16,
        y: i16,
        w: u16,
        h: u16,
        dx: i16,
        dy: i16,
    ) -> Result<()> {
        self.commands
            .lock()
            .unwrap()
            .push(MockCommand::CopyArea(x, y, w, h, dx, dy));
        Ok(())
    }

    fn draw_3d_rect(
        &self,
        x: i16,
        y: i16,
        w: u16,
        h: u16,
        sunken: bool,
    ) -> Result<()> {
        self.commands
            .lock()
            .unwrap()
            .push(MockCommand::Draw3dRect(x, y, w, h, sunken));
        Ok(())
    }

    fn draw_pixmap(&self, x: i16, y: i16, _data: &PixmapData) -> Result<()> {
        self.commands
            .lock()
            .unwrap()
            .push(MockCommand::DrawPixmap(x, y));
        Ok(())
    }

    fn draw_point(&self, x: i16, y: i16) -> Result<()> {
        self.commands
            .lock()
            .unwrap()
            .push(MockCommand::DrawPoint(x, y));
        Ok(())
    }

    fn draw_arc(
        &self,
        x: i16,
        y: i16,
        w: u16,
        h: u16,
        angle1: i16,
        angle2: i16,
    ) -> Result<()> {
        self.commands
            .lock()
            .unwrap()
            .push(MockCommand::DrawArc(x, y, w, h, angle1, angle2));
        Ok(())
    }

    fn fill_arc(
        &self,
        x: i16,
        y: i16,
        w: u16,
        h: u16,
        angle1: i16,
        angle2: i16,
    ) -> Result<()> {
        self.commands
            .lock()
            .unwrap()
            .push(MockCommand::FillArc(x, y, w, h, angle1, angle2));
        Ok(())
    }

    fn composite_pixmap(
        &self,
        src_pixmap: u32,
        src_size: crate::point::Dimension,
        dest: crate::point::Point,
        src: crate::point::Point,
    ) -> Result<()> {
        let (src_w, src_h) = src_size.as_px();
        let (dest_x, dest_y) = (dest.x as i16, dest.y as i16);
        let (src_x, src_y) = (src.x as i16, src.y as i16);
        self.commands
            .lock()
            .unwrap()
            .push(MockCommand::CompositePixmap(
                src_pixmap, src_w, src_h, dest_x, dest_y, src_x, src_y,
            ));
        Ok(())
    }
}

#[cfg(test)]
#[path = "graphics_tests.rs"]
mod tests;
