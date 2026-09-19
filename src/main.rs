mod decoder;
mod encoder;
use encoder::{Encoder, Pixel, QoiHeader , QOI_EOF};
use image::{self, ColorType};
use std::env;
fn main() {
    let args: Vec<String> = env::args().collect();
    let image_path = &args[1];
    let dyn_img = match image::open(image_path) {
        Ok(img) => img,
        Err(e) => {
            eprintln!("Failed to open {image_path}: {e}");
            std::process::exit(1);
        }
    };
    let color_type: u8 = match dyn_img.color() {
        ColorType::Rgb8 => 3,
        ColorType::Rgba8 => 4,
        _ => todo!(),
    };
    let is_rgb = if color_type == 3 { true } else { false };
    let width = dyn_img.width();
    let height = dyn_img.height();
    let image = dyn_img.into_bytes();

    let header = QoiHeader::new(width, height, color_type).to_bytes();
    let mut encoder = Encoder::new(is_rgb);
    for chunk in image.chunks_exact(color_type as usize) {
        let alpha = if is_rgb { 255 } else {chunk[3]};
        let current = Pixel::new(chunk[0], chunk[1], chunk[2], alpha);
        encoder.encode(current);
        encoder.update_prev(current);
    }
    let mut output:Vec<u8>=Vec::new();
    output.extend(header);
    output.extend(encoder.get_chunks());
    output.extend(QOI_EOF);
    match std::fs::write("./output.qoi", output) {
    Ok(_) => println!("Encoded successfully to ./output.qoi"),
    Err(e) => {
        eprintln!("Failed to write output file: {e}");
        std::process::exit(1);
    }
}
}
