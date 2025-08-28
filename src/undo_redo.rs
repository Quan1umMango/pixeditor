use crate::Color;

#[derive(Debug,Clone,PartialEq)]
pub enum Action {
    DrawPixels(DrawPixelsInfo),
}

#[derive(Debug,Clone,PartialEq)]
pub struct DrawPixelsInfo {
    // index of the pixel, from and to colors of the pixel (previous and new color)
    pub pixels:Vec<(usize,DrawInfo)>,
    pub layer_id:usize,
}

#[derive(Debug,Clone,PartialEq)]
pub struct DrawInfo {
    pub from:Option<Color>,
    pub to:Option<Color>
}

impl DrawPixelsInfo {
    pub fn new(pixels:Vec<(usize,DrawInfo)>,layer_id:usize) -> DrawPixelsInfo {
        DrawPixelsInfo {
            pixels,
            layer_id,
        }
    }
}

impl DrawInfo {
    pub fn new(from:Option<Color>,to:Option<Color>) -> DrawInfo {
        DrawInfo {
            from, to
        }
    }
}

impl Action {

    pub fn draw_pixels(pixels:Vec<(usize,DrawInfo)>,layer_id:usize) -> Action {
        Action::DrawPixels(DrawPixelsInfo::new(pixels,layer_id))
    }

    pub fn get_layer_id(&self) -> Option<usize> {
        match self {
            Action::DrawPixels(info) => return Some(info.layer_id),
            _ => None,
        }
    }
}

#[derive(Debug,Default,Clone)]
pub struct ActionsManager {
    undo_buf:Vec<Action>,
    redo_buf:Vec<Action>,
}

impl ActionsManager {
    pub fn new() -> Self {
        Self {
            undo_buf: Vec::new(),
            redo_buf: Vec::new(),
        }
    }

    pub fn undo(&mut self) -> Option<&Action> {
        if let Some(act) = self.undo_buf.pop() {
            self.redo_buf.push(act);
            return self.redo_buf.last(); 
        }
        None
    }

    pub fn redo(&mut self) -> Option<&Action> {
        if let Some(act) = self.redo_buf.pop() {
            self.undo_buf.push(act);
            return self.undo_buf.last(); 
        }
        None
    }

    pub fn add_action(&mut self, action:Action) {
        self.undo_buf.push(action);
        self.redo_buf.clear();
    }

}

