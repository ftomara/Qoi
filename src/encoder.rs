use crate::constants::{
    QOI_INDEX_SIZE, QOI_MAGIC, QOI_MAX_RUN, QOI_OP_DIFF, QOI_OP_LUMA, QOI_OP_RGB, QOI_OP_RGBA,
    QOI_OP_RUN,
};
pub use crate::pixel::Pixel;
pub struct QoiHeader {
    magic: [u8; 4],
    width: u32,
    height: u32,
    channels: u8,
    colorspace: u8,
}

impl QoiHeader {
    pub fn new(width: u32, height: u32, channels: u8) -> Self {
        QoiHeader {
            magic: QOI_MAGIC,
            width,
            height,
            channels,
            colorspace: 0,
        }
    }
    pub fn to_bytes(&mut self) -> Vec<u8> {
        let mut header: Vec<u8> = Vec::new();
        header.extend(self.magic);
        header.extend(self.width.to_be_bytes());
        header.extend(self.height.to_be_bytes());
        header.push(self.channels);
        header.push(self.colorspace);
        header
    }
}


pub struct Encoder {
    prev_pixel: Pixel,
    prev_seen_list: [Pixel; QOI_INDEX_SIZE],
    is_run: bool,
    chunks: Vec<u8>,
    is_rgb: bool,
}

impl Encoder {
    pub fn new(is_rgb: bool) -> Self {
        Encoder {
            prev_pixel: Pixel(0, 0, 0, 255),
            prev_seen_list: [Pixel(0, 0, 0, 0); QOI_INDEX_SIZE],
            is_run: false,
            chunks: Vec::new(),
            is_rgb,
        }
    }
    fn qoi_op_run(&mut self, current_pixel: Pixel) -> bool {
        if !self.prev_pixel.is_equal(&current_pixel) {
            self.is_run = false;
            return false;
        } // why this worked ? and this {false} didn't?
        if self.is_run {
            let last_chunck = match self.chunks.last_mut() {
                Some(ch) => ch,
                None => panic!(),
            };
            if *last_chunck < QOI_OP_RUN | (QOI_MAX_RUN - 1) {
                *last_chunck += 1;
            } else {
                self.chunks.push(QOI_OP_RUN);
            }
        } else {
            self.is_run = true;
            self.chunks.push(QOI_OP_RUN);
        }
        true
    }
    fn qoi_op_index(&mut self, current_pixel: Pixel) -> bool {
        let index: usize = Self::get_index(&current_pixel).into();
        let pixel = self.prev_seen_list[index];
        if pixel.is_equal(&current_pixel) {
            self.chunks.push(index as u8);
            return true;
        } else {
            return false;
        }
    }
    fn qoi_op_diff(&mut self, current_pixel: Pixel) -> bool {
        let mut diff_pixel = self.prev_pixel.sub_pixels(&current_pixel);
        if !diff_pixel.is_diff_range() {
            return false;
        }
        let tag: u8 = QOI_OP_DIFF;
        diff_pixel.0 = (diff_pixel.0 as i8 + 2) as u8;
        diff_pixel.1 = (diff_pixel.1 as i8 + 2) as u8;
        diff_pixel.2 = (diff_pixel.2 as i8 + 2) as u8;

        let dr = diff_pixel.0 << 4;
        let dg = diff_pixel.1 << 2;
        let db = diff_pixel.2;
        let chunk = tag | dr | dg | db;
        self.chunks.push(chunk);
        true
    }
    fn qoi_op_luma(&mut self, current_pixel: Pixel) -> bool {
        let diff_pixel = self.prev_pixel.sub_pixels(&current_pixel);
        if !diff_pixel.is_luma_range() {
            return false;
        }
        let tag: u8 = QOI_OP_LUMA;
        let dg = ((diff_pixel.1 as i8) + 32) as u8;
        let dr_dg = ((diff_pixel.0 as i8 - diff_pixel.1 as i8) + 8) as u8;
        let db_dg = ((diff_pixel.2 as i8 - diff_pixel.1 as i8) + 8) as u8;
        let chunk1 = tag | dg;
        let chunk2 = (dr_dg << 4) | db_dg;
        self.chunks.push(chunk1);
        self.chunks.push(chunk2);
        true
    }
    fn qoi_op_rgb(&mut self, current_pixel: Pixel) -> bool {
        let tag: u8 = QOI_OP_RGB;
        self.chunks.push(tag);
        self.chunks.push(current_pixel.0);
        self.chunks.push(current_pixel.1);
        self.chunks.push(current_pixel.2);
        true
    }
    fn qoi_op_rgba(&mut self, current_pixel: Pixel) -> bool {
        let tag: u8 = QOI_OP_RGBA;
        self.chunks.push(tag);
        self.chunks.push(current_pixel.0);
        self.chunks.push(current_pixel.1);
        self.chunks.push(current_pixel.2);
        self.chunks.push(current_pixel.3);
        true
    }

    fn get_index(current_pixel: &Pixel) -> u8 {
        (((current_pixel.0) as u32 * 3
            + (current_pixel.1) as u32 * 5
            + (current_pixel.2) as u32 * 7
            + (current_pixel.3) as u32 * 11)
            % QOI_INDEX_SIZE as u32) as u8
    }
    pub fn encode(&mut self, current_pixel: Pixel) {
        if self.qoi_op_run(current_pixel) {
            return;
        }
        if self.qoi_op_index(current_pixel) {
            return;
        }
        if self.qoi_op_diff(current_pixel) {
            return;
        }
        if self.qoi_op_luma(current_pixel) {
            return;
        }
        if self.is_rgb {
            self.qoi_op_rgb(current_pixel);
            return;
        } else {
            self.qoi_op_rgba(current_pixel);
            return;
        }
    }
    pub fn update_prev(&mut self, current_pixel: Pixel) {
        self.prev_pixel = current_pixel;
        let index: usize = Self::get_index(&current_pixel).into();
        self.prev_seen_list[index] = current_pixel;
    }
    pub fn get_chunks(self) -> Vec<u8>{
        self.chunks
    }
}
