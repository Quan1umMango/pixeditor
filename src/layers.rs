use crate::*;
use uuid::Uuid;

#[derive(Clone)]
pub struct Layers {
    layers: Vec<Canvas>,
    current_layer: Option<Uuid>,
    active_layers: Vec<Uuid>,
    width:u16,
    height:u16
}

impl Layers {
    pub fn new(cur_layer: Canvas) -> Self {
        let id = cur_layer.get_id().clone();
        let height = cur_layer.image_repr().height() as u16;
        let width = cur_layer.image_repr().width() as u16;
        Self {
            layers: vec![cur_layer],
            current_layer: Some(id.clone()),
            active_layers: vec![id.clone()],
            width,
            height
            
        }
    }

    pub fn add_layer(&mut self, new_layer: Canvas) {
        self.layers.push(new_layer);
    }

    pub fn remove_layer(&mut self, index: &Uuid) {
        // handle if active layers 
        if self.is_active(index) && self.active_layers.len() > 1 {
            self.active_layers.retain(|e| e != index);
            if let Some(s) = self.current_layer {
                if s == *index {
                    self.check_current_layer();
                }
            }
        }else if self.layers.len() > 1 && !self.is_active(index)  {

            self.layers.retain(|e| e.get_id() != index);
        }

    }



    pub fn check_current_layer(&mut self) {
        if let Some(cur_id) = self.current_layer {
            if !self.active_layers.contains(&cur_id) {
                self.current_layer = self.active_layers.get(0).copied();
            }
        } else {
            self.current_layer = self.active_layers.get(0).copied();
        }
    }


    pub fn get_current_layer(&mut self) -> Option<&Canvas> {
        self.check_current_layer();
        self.current_layer.and_then(|cl| self.layers.iter().find(|e| e.get_id() == &cl))
    }

    pub fn get_current_layer_mut(&mut self) -> Option<&mut Canvas> {
        self.check_current_layer();
        self.current_layer.and_then(|cl| self.layers.iter_mut().find(|e| e.get_id() == &cl))
    }

    pub fn set_current_layer(&mut self, i: &Uuid) {
        if self.layers.iter().any(|e| e.get_id() == i) {
            self.current_layer = Some(*i);
        }
    }

    pub fn set_current_layer_newest(&mut self) {
        if let Some(last_layer) = self.layers.last() {
            self.current_layer = Some(last_layer.get_id().clone());
        }
    }

    pub fn set_current_layer_oldest(&mut self) {
        if let Some(first_layer) = self.layers.first() {
            self.current_layer = Some(first_layer.get_id().clone());
        }
    }

    pub fn get_all_layers(&self) -> &Vec<Canvas> {
        &self.layers
    }

    pub fn get_current_layer_index(&self) -> Option<&Uuid> {
        self.current_layer.as_ref()
    }

    pub fn get_layer(&self, i: &Uuid) -> Option<&Canvas> {
        self.layers.iter().find(|e| e.get_id() == i)
    }

    pub fn get_layer_mut(&mut self, i: &Uuid) -> Option<&mut Canvas> {
        self.layers.iter_mut().find(|e| e.get_id() == i)
    }

    pub fn is_active(&self, i: &Uuid) -> bool {
        self.active_layers.contains(i)
    }

    pub fn set_active(&mut self, i: &Uuid) {
        if !self.is_active(i) {
            self.active_layers.push(i.clone());
            self.current_layer = Some(i.clone())
        }
    }

    pub fn get_all_active(&self) -> &Vec<Uuid> {
        &self.active_layers
    }

    pub fn get_all_active_mut(&mut self) -> &mut Vec<Uuid> {
        &mut self.active_layers
    }

    pub fn unactive(&mut self, i: &Uuid) {
        if self.active_layers.len() != 1 {
            self.active_layers.retain(|e| e != i);
        }
    }

    pub fn get_active_layer(&mut self, id:&Uuid) -> Option<&Canvas> {
        self.check_current_layer();
        self.layers.iter().find(|e| e.get_id() == id)
    }

    pub fn get_all_active_canvas(&self) -> Vec<Canvas> {
        let mut c = self.layers.clone(); 
        c.retain(|e| self.is_active(e.get_id()));

        c
    }

    pub fn get_combined_export(&self) -> Image {
        let mut output_image = Image::gen_image_color(self.width,self.height,Color::from_rgba(0,0,0,0));
        for aci in self.active_layers.iter() {
            if let Some(ac) = self.get_layer(aci) {

            ac.image_repr().draw_to_image(&mut output_image);
            }
        }
        output_image
    }

    pub fn save_combined_layers(&self) {
        use native_dialog::*;
        let path = FileDialog::new()
            .set_location("~/Desktop")
            .add_filter("PNG Image", &["png"])
            .add_filter("JPEG Image", &["jpg", "jpeg"])
            .show_save_single_file()
            .unwrap();

        if let Some(n) = path {
            let n = n.as_os_str().to_str().unwrap();
            //self.set_name(n.to_string());
            //self.get_selected_canvas_mut().unwrap().save_image_with_name(n);
            let image = self.get_combined_export();
            image.export_png(n);

        }
    }


}
