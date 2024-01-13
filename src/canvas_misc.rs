use crate::*;

// Gets all pixel which are needed in the fill tool 
pub fn fill_get_all_pixels(pixel_index:usize,image_repr:&ImageRepr,visited:&mut Vec<usize>,wanted_color:Color)  {
    visited.push(pixel_index);

    let neighbours = image_repr.get_neighbours(pixel_index);
    for n in neighbours.iter() {
        if !visited.contains(&n) && image_repr.data()[*n].1==wanted_color{
            visited.push(*n);
            fill_get_all_pixels(*n,image_repr,visited,wanted_color);
        }
    }


}


