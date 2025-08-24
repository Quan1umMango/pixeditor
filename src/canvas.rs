use crate::RECT_DIMS;
use crate::{FillType, Layer, Tool, ToolInfo, ToolKind};
use macroquad::prelude::*;

/// Handles all the things related to the actual drawing, like layers, inactive layers, tool
/// previews etc.
#[derive(Default)]
pub struct Canvas {
    num_pixels: usize,
    active_layer: usize,
    layers: Vec<Layer>,
    tool: Tool,
    selected_color: Color,
}

impl Canvas {
    pub fn new(num_pixels: usize) -> Self {
        Self {
            num_pixels,
            layers: vec![Layer::new(num_pixels)],
            active_layer: 0,
            selected_color: RED,
            ..Default::default()
        }
    }

    pub fn get_pos_from_tool_info(
        &self,
        camera: &Camera2D,
        info: &ToolInfo,
    ) -> Option<(Vec2, Vec2)> {
        if let Some((a, b)) = self.get_pos_from_tool_info_unordered(camera, info) {
            return Some((a.min(b), a.max(b)));
        }
        None
    }

    pub fn get_pos_from_tool_info_unordered(
        &self,
        camera: &Camera2D,
        info: &ToolInfo,
    ) -> Option<(Vec2, Vec2)> {
        if let (Some(i), Some(f)) = (info.initial_loc, info.final_loc) {
            let r1 = self.get_pixel_rect_from_position(camera.screen_to_world(i));
            let r2 = self.get_pixel_rect_from_position(camera.screen_to_world(f));
            if r1.is_none() || r2.is_none() {
                None
            } else {
                let r1 = r1.unwrap();
                let r2 = r2.unwrap();

                let p1 = vec2(r1.x, r1.y);
                let p2 = vec2(r2.x, r2.y);
                Some((p1, p2))
            }
        } else {
            None
        }
    }

    /// Returns the pixels to color based on the current tool. Only really useful for Line and Fill,
    /// otherwise we can just make do with get_pos_from_rect_border_info which will take O(n)
    /// This on the otherhand, combined with the drawing, takes O(n^2)
    ///
    /// ALWAYS returns a vector (empty if no pixels found) if its a Fill or a Line tool, otherwise
    /// returns None
    pub fn try_get_pixels_to_color(
        &self,
        camera: &Camera2D,
        kind: ToolKind,
        info: &ToolInfo,
    ) -> Option<Vec<usize>> {
        match kind {
            ToolKind::Fill => {
                let mut out = Vec::<usize>::new();

                let cpos = camera.screen_to_world(mouse_position().into());
                if let Some(index) = self.get_pixel_index_from_position(cpos) {
                    let c = self.active_layer().unwrap().data[index];
                    self.use_fill(&mut out, c, index);
                }
                Some(out)
            }
            ToolKind::Line => {
                let mut out = Vec::<usize>::new();

                if info.initial_loc.is_none() || info.final_loc.is_none() {
                    return Some(out);
                }

                let res = self.get_pos_from_tool_info_unordered(camera, info);
                if res.is_none() {
                    return Some(out);
                }
                let (start, end) = res.unwrap();

                let si = self.get_pixel_index_from_position(start).unwrap();
                let ei = self.get_pixel_index_from_position(end).unwrap();

                let s_coords = (si % self.num_pixels, si / self.num_pixels);
                let e_coords = (ei % self.num_pixels, ei / self.num_pixels);

                // Slope == inf
                if e_coords.1 == s_coords.1 {
                    for x in s_coords.0..=e_coords.0 {
                        out.push(x + s_coords.1 * self.num_pixels);
                    }
                    return Some(out);
                }

                let slope = (e_coords.0 as f32 - s_coords.0 as f32)
                    / (e_coords.1 as f32 - s_coords.1 as f32);

                if slope == 0. {
                    for y in s_coords.1..=e_coords.1 {
                        out.push(s_coords.0 + y * self.num_pixels);
                    }
                    return Some(out);
                }

                for x in s_coords.0..=e_coords.0 {
                    // m = (x1-x0)/(y1-y0) => y1 = (x1-x0)/m + y0
                    let yf32 = (x as f32 - s_coords.0 as f32) / slope + s_coords.1 as f32;
                    let index = ((yf32.floor() * self.num_pixels as f32) + x as f32) as usize;
                    out.push(index);
                }

                Some(out)
            }
            _ => None,
        }
    }

    fn use_fill(&self, res: &mut Vec<usize>, color_to_replace: Option<Color>, cur_index: usize) {
        let active_layer = self.active_layer().unwrap();
        if let Some(c) = active_layer.data.get(cur_index) {
            if *c != color_to_replace || res.contains(&cur_index) {
                return;
            }

            res.push(cur_index);
            // search right
            if cur_index % self.num_pixels != self.num_pixels - 1 {
                self.use_fill(res, color_to_replace, cur_index + 1);
            }

            // search left
            if cur_index % self.num_pixels != 0 {
                self.use_fill(res, color_to_replace, cur_index - 1);
            }

            // search top
            if cur_index >= self.num_pixels {
                self.use_fill(res, color_to_replace, cur_index - self.num_pixels);
            }

            // search bottom
            if cur_index <= self.num_pixels.pow(2) {
                self.use_fill(res, color_to_replace, cur_index + self.num_pixels);
            }
        }
    }

    pub fn draw(&self, camera: &Camera2D) {
        let c1 = WHITE;
        let c2 = BLACK;

        let tool = &self.tool;
        let kind = tool.kind;
        let info = &tool.info;
        let start_end_pos: Option<(Vec2, Vec2)> = self.get_pos_from_tool_info(camera, info);
        //let pixels_to_draw_special:Option<Vec<usize>> = self.get_pixels_to_draw_special();

        for i in 0..(self.num_pixels * self.num_pixels) {
            let active_layer = self.active_layer().unwrap();
            let x = (i % self.num_pixels) as f32;
            let y = (i / self.num_pixels) as f32;

            // Draw the contents of all layers or else the bg
            if let Some(c) = active_layer.data[i] {
                // -1.0 to start from the top left of the screen
                draw_rectangle(
                    x * RECT_DIMS.x - 1.0,
                    y * RECT_DIMS.y - 1.0,
                    RECT_DIMS.x,
                    RECT_DIMS.y,
                    c,
                );
            } else {
                'inner: {
                    for l in self.layers.iter() {
                        if l.data[i].is_some() {
                            draw_rectangle(
                                x * RECT_DIMS.x - 1.0,
                                y * RECT_DIMS.y - 1.0,
                                RECT_DIMS.x,
                                RECT_DIMS.y,
                                l.data[i].unwrap(),
                            );
                            break 'inner;
                        }
                    }

                    let c = if (x as usize + y as usize) % 2 == 0 {
                        c1
                    } else {
                        c2
                    };
                    draw_rectangle(
                        x * RECT_DIMS.x - 1.0,
                        y * RECT_DIMS.y - 1.0,
                        RECT_DIMS.x,
                        RECT_DIMS.y,
                        c,
                    );
                }
            }

            // Now draw any tool-specific things (example: rectangle borders for Rect tool)
            match kind {
                ToolKind::Rect(fill_type) => {
                    if start_end_pos.is_none() {
                        continue;
                    }
                    let (min, max) = start_end_pos.unwrap();
                    let cpos = vec2(x * RECT_DIMS.x - 1.0, y * RECT_DIMS.y - 1.0);

                    match fill_type {
                        FillType::NoFill => {
                            if !((cpos.x >= min.x && cpos.x <= max.x)
                                && (cpos.y == min.y || cpos.y == max.y))
                                && !((cpos.y >= min.y && cpos.y <= max.y)
                                    && (cpos.x == min.x || cpos.x == max.x))
                            {
                                continue;
                            }
                        }

                        FillType::SolidFill => {
                            if !((cpos.x >= min.x && cpos.x <= max.x)
                                && (cpos.y >= min.y && cpos.y <= max.y))
                            {
                                continue;
                            }
                        }
                    }
                    draw_rectangle(
                        x * RECT_DIMS.x - 1.0,
                        y * RECT_DIMS.y - 1.0,
                        RECT_DIMS.x,
                        RECT_DIMS.y,
                        self.selected_color,
                    );
                }
                ToolKind::Line => {
                    if let Some(v) = info.pixel_indices.as_ref() {
                        if v.contains(&i) {
                            draw_rectangle(
                                x * RECT_DIMS.x - 1.0,
                                y * RECT_DIMS.y - 1.0,
                                RECT_DIMS.x,
                                RECT_DIMS.y,
                                self.selected_color,
                            );
                        }
                    }
                }
                _ => {}
            }
        }
    }

    pub fn set_pixel_at_mouse_position_selected_color(&mut self, camera: &Camera2D) {
        self.set_pixel_at_mouse_position(Some(self.selected_color), camera);
    }

    pub fn erase_at_mouse_position(&mut self, camera: &Camera2D) {
        self.set_pixel_at_mouse_position(None, camera);
    }

    pub fn set_pixel_at_mouse_position(&mut self, color: Option<Color>, camera: &Camera2D) {
        let mp = camera.screen_to_world(mouse_position().into());
        let index = self.get_pixel_index_from_position(mp);
        let active_layer = self.active_layer_mut().unwrap();

        if let Some(index) = index {
            active_layer.set_pixel(index, color);
        }
    }

    pub fn set_pixel_at(&mut self, index: usize, color: Option<Color>) {
        let active_layer = self.active_layer_mut().unwrap();
        active_layer.set_pixel(index, color);
    }

    pub fn set_tool(&mut self, new_tool: Tool) {
        self.tool = new_tool;
    }

    pub fn get_pixel_index_at_mouse_position(&self, camera: &Camera2D) -> Option<usize> {
        let mp = camera.screen_to_world(mouse_position().into());
        self.get_pixel_index_from_position(mp)
    }

    pub fn use_tool_at_mouse_position(&mut self, camera: &Camera2D) {
        let np = self.num_pixels;
        let sc = Some(self.selected_color);

        match self.tool.kind {
            ToolKind::Pixel => {
                if !is_mouse_button_down(MouseButton::Left) {
                    return;
                }
                self.set_pixel_at_mouse_position(sc, camera);
            }
            ToolKind::Eraser => {
                if !is_mouse_button_down(MouseButton::Left) {
                    return;
                }

                self.erase_at_mouse_position(camera);
            }

            ToolKind::Rect(fill_type) => {
                if self.tool.info.initial_loc.is_none() {
                    if !is_mouse_button_down(MouseButton::Left) {
                        return;
                    }
                    self.tool.info.initial_loc = Some(mouse_position().into());
                }

                self.tool.info.final_loc = Some(mouse_position().into());

                if is_mouse_button_down(MouseButton::Left) {
                    return;
                }

                let rects = self.get_pos_from_tool_info(camera, &self.tool.info);

                self.tool.info = ToolInfo::default();

                if rects.is_none() {
                    return;
                }
                let (min, max) = rects.unwrap();

                for i in 0..(np.pow(2)) {
                    let x = (i % np) as f32;
                    let y = (i / np) as f32;

                    let cpos = vec2(x * RECT_DIMS.x - 1.0, y * RECT_DIMS.y - 1.0);
                    match fill_type {
                        FillType::NoFill => {
                            if !((cpos.x >= min.x && cpos.x <= max.x)
                                && (cpos.y == min.y || cpos.y == max.y))
                                && !((cpos.y >= min.y && cpos.y <= max.y)
                                    && (cpos.x == min.x || cpos.x == max.x))
                            {
                                continue;
                            }
                        }
                        FillType::SolidFill => {
                            if !((cpos.x >= min.x && cpos.x <= max.x)
                                && (cpos.y >= min.y && cpos.y <= max.y))
                            {
                                continue;
                            }
                        }
                    }
                    self.set_pixel_at(i, sc);
                }
            }
            ToolKind::Fill => {
                if !is_mouse_button_pressed(MouseButton::Left) {
                    return;
                }

                let s = self
                    .try_get_pixels_to_color(camera, ToolKind::Fill, &self.tool.info)
                    .unwrap();
                for i in s.iter() {
                    self.set_pixel_at(*i, sc);
                }
            }

            ToolKind::Line => {
                if self.tool.info.initial_loc.is_none() {
                    if !is_mouse_button_down(MouseButton::Left) {
                        return;
                    }
                    self.tool.info.initial_loc = Some(mouse_position().into());
                }

                self.tool.info.final_loc = Some(mouse_position().into());
                let s = self.try_get_pixels_to_color(camera, ToolKind::Line, &self.tool.info);

                if is_mouse_button_down(MouseButton::Left) {
                    self.tool.info.pixel_indices = s;
                    return;
                }

                if let Some(pis) = s {
                    for i in pis.into_iter() {
                        self.set_pixel_at(i, sc);
                    }
                }
                self.tool.info = ToolInfo::default();
            }
        }
    }

    // Currently the time complexity is O(n), but i think we can make it better by using just
    // maths? maybe?
    // Or we can make it O(nlogn) by using a binary search type thing
    pub fn get_pixel_index_from_position(&self, pos: Vec2) -> Option<usize> {
        let mp = Rect::new(pos.x, pos.y, 0.01, 0.01);

        for i in 0..(self.num_pixels.pow(2)) {
            let x = (i % self.num_pixels) as f32;
            let y = (i / self.num_pixels) as f32;

            let rect = Rect::new(
                x * RECT_DIMS.x - 1.0,
                y * RECT_DIMS.y - 1.0,
                RECT_DIMS.x,
                RECT_DIMS.y,
            );
            if rect.intersect(mp).is_some() {
                return Some(i);
            }
        }
        None
    }

    pub fn get_pixel_rect_from_index(&self, i: usize) -> Option<Rect> {
        if i > self.num_pixels.pow(2) {
            return None;
        }
        let x = (i % self.num_pixels) as f32;
        let y = (i / self.num_pixels) as f32;
        Some(Rect::new(
            x * RECT_DIMS.x - 1.0,
            y * RECT_DIMS.y - 1.0,
            RECT_DIMS.x,
            RECT_DIMS.y,
        ))
    }

    pub fn get_pixel_rect_from_position(&self, pos: Vec2) -> Option<Rect> {
        // NOTE: almost the same code as get_pixel_index_from_position, now any optimizations would apply here
        let p = Rect::new(pos.x, pos.y, 0.01, 0.01);

        for i in 0..(self.num_pixels.pow(2)) {
            let x = (i % self.num_pixels) as f32;
            let y = (i / self.num_pixels) as f32;

            let rect = Rect::new(
                x * RECT_DIMS.x - 1.0,
                y * RECT_DIMS.y - 1.0,
                RECT_DIMS.x,
                RECT_DIMS.y,
            );
            if rect.intersect(p).is_some() {
                return Some(rect);
            }
        }
        None
    }

    pub fn active_layer(&self) -> Option<&Layer> {
        self.layers.get(self.active_layer)
    }

    pub fn active_layer_mut(&mut self) -> Option<&mut Layer> {
        self.layers.get_mut(self.active_layer)
    }

    pub fn selected_color(&self) -> Color {
        self.selected_color
    }
    pub fn set_selected_color(&mut self, nc: Color) {
        self.selected_color = nc;
    }
}
