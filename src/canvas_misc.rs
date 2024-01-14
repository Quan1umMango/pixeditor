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


pub fn rect_get_border_pixels(init_pos:&Vec2,dimensions:(u32,u32),px:u32,py:u32,image:&Image) -> Vec<(u32,u32)> {
    let (mut ipx, mut ipy) = get_pixel_coords(*init_pos,dimensions);
    if ipx == px || ipy == py {
        return Vec::new();
    }

    if ipx >= image.width() as u32 {
        ipx = image.width() as u32 - 1;
    }
    if ipy >= image.height() as u32{
        ipy = image.height() as u32 - 1;
    }

    let mut out =  Vec::new();

    for x in px.min(ipx)+1..=px.max(ipx) {                        
        if x >= image.width() as u32 {
            return Vec::new();
        }
        out.push((x,ipy));
        out.push((x,py));

    }

    for y in py.min(ipy)..=py.max(ipy) {
        if y >= image.height() as u32{
            return Vec::new();
        }
        out.push((ipx,y));
        out.push((px,y));
    }

    out

}
