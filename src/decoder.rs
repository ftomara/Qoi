use crate::pixel::Pixel;
pub struct Decoder {
    img_width: u32,
    img_height: u32,
    chunks: Vec<u8>,
    image: Vec<u8>,
    channels: u8,
    _color_space: u8,
    current_chunk_index: usize,
    prev_seen_list: [Pixel; 64],
}

impl Decoder {
    pub fn new(outfile: Vec<u8>) -> Result<Self, String> {
        let header = outfile.first_chunk::<14>().unwrap();
        match str::from_utf8(&header[0..4]) {
            Ok(s) => {
                if !(s == "qoif") {
                    Err("File format invalid".to_string())
                } else {
                    Ok(Decoder {
                        img_width: u32::from_be_bytes(header[4..8].try_into().unwrap()),
                        img_height: u32::from_be_bytes(header[8..12].try_into().unwrap()),
                        chunks: outfile[14..outfile.len()].to_vec(),
                        image: Vec::new(),
                        channels: header[12],
                        _color_space: header[13],
                        current_chunk_index: 0,
                        prev_seen_list: [Pixel(0, 0, 0, 255); 64],
                    })
                }
            }
            Err(_e) => panic!("Invalid Qoi file format"),
        }
    }
    fn get_tag_value(mut byte: u8) -> (u8, u8) {
        let tag = byte >> 6;
        byte <<= 2;
        byte >>= 2;
        (tag, byte)
    }
    fn get_start_end(&self) -> (usize, usize) {
        (self.image.len() - self.channels as usize, self.image.len())
    }
    fn get_last_pixel(&self) -> Vec<u8> {
        let last_pixel: Vec<u8>;
        if self.image.is_empty() {
            last_pixel = Vec::from([0, 0, 0, 255]);
        } else {
            let (start, end) = self.get_start_end();
            last_pixel = Vec::from(&self.image[start..end]);
        }
        last_pixel
    }
    fn get_index(r: u8, g: u8, b: u8, a: u8) -> usize {
        ((r as u32 * 3 + g as u32 * 5 + b as u32 * 7 + a as u32 * 11) % 64) as usize
    }
    fn update_prev_seen(&mut self, last_pixel: Vec<u8>) {
        let (r, g, b, a) = (
            last_pixel[0],
            last_pixel[1],
            last_pixel[2],
            if self.channels == 4 {
                last_pixel[3]
            } else {
                255
            },
        );
        let index = Self::get_index(r, g, b, a);
        self.prev_seen_list[index] = Pixel(r, g, b, a);
    }
    fn push(&mut self, dr: i8, dg: i8, db: i8) {
        let last_pixel = self.get_last_pixel();
        self.image.push(last_pixel[0].wrapping_add(dr as u8));
        self.image.push(last_pixel[1].wrapping_add(dg as u8));
        self.image.push(last_pixel[2].wrapping_add(db as u8));
        if self.channels == 4 {
            self.image.push(last_pixel[3]);
        }
        self.update_prev_seen(self.get_last_pixel());
    }
    fn decode_run(&mut self) -> bool {
        let (tag, mut byte) = Self::get_tag_value(self.chunks[self.current_chunk_index]);
        if tag != 3 {
            return false;
        }
        byte += 1;
        let last_pixel = self.get_last_pixel();
        for _i in 1..=byte {
            self.image.extend(&last_pixel);
        }
        self.update_prev_seen(last_pixel);
        self.current_chunk_index += 1;
        true
    }
    fn decode_index(&mut self) -> bool {
        let (tag, byte) = Self::get_tag_value(self.chunks[self.current_chunk_index]);
        if tag != 0 {
            return false;
        }
        let p = self.prev_seen_list[byte as usize];
        self.image.push(p.0);
        self.image.push(p.1);
        self.image.push(p.2);
        if self.channels == 4 {
            self.image.push(p.3)
        }
        self.update_prev_seen(Vec::from([p.0, p.1, p.2, p.3]));
        self.current_chunk_index += 1;
        true
    }
    fn decode_diff(&mut self) -> bool {
        let (tag, byte) = Self::get_tag_value(self.chunks[self.current_chunk_index]);
        if tag != 1 {
            return false;
        }
        let dr = (byte >> 4) as i8 - 2;
        let dg = (byte >> 2 & 3) as i8 - 2;
        let db = (byte & 3) as i8 - 2;
        self.push(dr, dg, db);
        self.current_chunk_index += 1;
        true
    }
    fn decode_luma(&mut self) -> bool {
        let (tag, byte) = Self::get_tag_value(self.chunks[self.current_chunk_index]);
        if tag != 2 {
            return false;
        }
        let dr_db = self.chunks[self.current_chunk_index + 1];
        let dg = byte as i8 - 32;
        let dr = (dr_db >> 4) as i8 - 8 + dg;
        let db = (dr_db & 15) as i8 - 8 + dg;
        self.push(dr, dg, db);
        self.current_chunk_index += 2;
        true
    }
    fn decode_rgb(&mut self) -> bool {
        if self.chunks[self.current_chunk_index] != 254 {
            return false;
        }
        self.image
            .extend(&self.chunks[self.current_chunk_index + 1..self.current_chunk_index + 4]);
        self.update_prev_seen(Vec::from(
            &self.chunks[self.current_chunk_index + 1..self.current_chunk_index + 4],
        ));
        self.current_chunk_index += 4;
        true
    }
    fn decode_rgba(&mut self) -> bool {
        if self.chunks[self.current_chunk_index] != 255 {
            return false;
        }
        self.image
            .extend(&self.chunks[self.current_chunk_index + 1..self.current_chunk_index + 5]);
        self.update_prev_seen(Vec::from(
            &self.chunks[self.current_chunk_index + 1..self.current_chunk_index + 5],
        ));
        self.current_chunk_index += 5;
        true
    }
    pub fn decode(mut self) -> (Vec<u8>, u32, u32, u8) {
        let image_size: usize =
            self.img_width as usize * self.img_height as usize * self.channels as usize;
        while self.image.len() < image_size {
            // println!(" len : {}", self.image.len());
            if self.decode_rgba() {
                continue;
            }
            if self.decode_rgb() {
                continue;
            }
            if self.decode_diff() {
                continue;
            }
            if self.decode_luma() {
                continue;
            }
            if self.decode_run() {
                continue;
            }
            if self.decode_index() {
                continue;
            }
        }
        println!(
            "current chunk index {} , length : {} ",
            self.current_chunk_index,
            self.chunks.len()
        );
        (self.image, self.img_width, self.img_height, self.channels)
    }
}

/*
let header = outfile.first_chunk::<14>().unwrap();
    let magic_format = &header[0..4];
    let img_width: [u8; 4] = header[4..8].try_into().unwrap();
    let img_height:[u8;4] = header[8..12].try_into().unwrap();
    let img_is_rgb = &header[12];
    let img_color_space = &header[13];
    for i in magic_format {
        print!("{}", *i as char);
    }
    println!(" ");
    let w: u32 = u32::from_be_bytes(img_width);
    println!("w : {w} ");
    let h: u32 = u32::from_be_bytes(img_height);
    println!("h : {h} ");
    println!("{}", img_is_rgb);
    println!("{}", img_color_space);
*/
