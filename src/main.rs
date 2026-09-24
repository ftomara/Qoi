mod decoder;
mod encoder;
mod pixel;
use decoder::Decoder;
use encoder::{Encoder, Pixel, QoiHeader, QOI_EOF};
use image::{self, ColorType};
use std::env;
fn main() -> Result<(), Box<dyn std::error::Error + 'static>> {
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
    println!("width {width}");
    println!("height {height}");
    let image = dyn_img.into_bytes();

    let header = QoiHeader::new(width, height, color_type).to_bytes();
    let mut encoder = Encoder::new(is_rgb);
    for chunk in image.chunks_exact(color_type as usize) {
        let alpha = if is_rgb { 255 } else { chunk[3] };
        let current = Pixel::new(chunk[0], chunk[1], chunk[2], alpha);
        encoder.encode(current);
        encoder.update_prev(current);
    }
    let mut output: Vec<u8> = Vec::new();
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
    //consider using starts with and ends with for qoif and the eof to make sure file is correct
    let outfile: Vec<u8> = std::fs::read("./output.qoi")?;

    let decoder = Decoder::new(outfile)?;
    let decompressed = decoder.decode();
    println!(
        "vector size : {} , w: {} , h: {} , w*h: {}",
        decompressed.0.len(),
        decompressed.1,
        decompressed.2,
        decompressed.1 * decompressed.2
    );
    let image: image::DynamicImage = if decompressed.3 == 3 {
        image::DynamicImage::ImageRgb8(
            image::RgbImage::from_raw(decompressed.1, decompressed.2, decompressed.0)
                .expect("pixel data length doesn't match width/height"),
        )
    } else {
        image::DynamicImage::ImageRgba8(
            image::RgbaImage::from_raw(decompressed.1, decompressed.2, decompressed.0)
                .expect("pixel data length doesn't match width/height"),
        )
    };

    image
        .save("decoded_output.jpg")
        .expect("failed to save output image");
    Ok(())
}
