mod constants;
mod decoder;
mod encoder;
mod pixel;
use constants::QOI_EOF;
use decoder::Decoder;
use encoder::{Encoder, Pixel, QoiHeader};
use image::{self, ColorType};
use std::env;
fn encode(image_path: &str) {
    let file_name = std::path::Path::new(image_path).with_extension("qoi");
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
        let alpha = if is_rgb { 255 } else { chunk[3] };
        let current = Pixel::new(chunk[0], chunk[1], chunk[2], alpha);
        encoder.encode(current);
        encoder.update_prev(current);
    }
    let mut output: Vec<u8> = Vec::new();
    output.extend(header);
    output.extend(encoder.get_chunks());
    output.extend(QOI_EOF);
    match std::fs::write(&file_name, output) {
        Ok(_) => println!("Encoded successfully to {}", file_name.display()),
        Err(e) => {
            eprintln!("Failed to write output file: {e}");
            std::process::exit(1);
        }
    }
}
fn decode(outfile: &str) {
    let file_name = std::path::Path::new(outfile).with_extension("png");
    let file: Vec<u8> = std::fs::read(outfile).expect("Couldn't read the file");
    let decoder = Decoder::new(file).expect("File format is wrong");
    let decompressed = decoder.decode();
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
        .save(file_name)
        .expect("failed to save output image");
}
fn main() {
    let args: Vec<String> = env::args().collect();
    let subcommand = &args[1];
    if subcommand == "encode" {
        encode(&args[2])
    } else if subcommand == "decode" {
        decode(&args[2])
    } else {
        println!("unknown command , try again with encode or decode please !")
    }
}
