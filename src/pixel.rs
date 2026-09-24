#[derive(Clone, Copy)]
pub struct Pixel(pub u8, pub u8, pub u8, pub u8); //red,green,blue,alpha(255 if only rgb)

impl Pixel {
    pub fn new(r: u8, g: u8, b: u8, a: u8) -> Pixel {
        Pixel(r, g, b, a)
    }
    pub fn is_equal(&self, other: &Pixel) -> bool {
        self.0 == other.0 && self.1 == other.1 && self.2 == other.2 && self.3 == other.3
    }
    pub fn sub_pixels(&self, other: &Pixel) -> Pixel {
        Pixel(
            other.0.wrapping_sub(self.0),
            other.1.wrapping_sub(self.1),
            other.2.wrapping_sub(self.2),
            other.3.wrapping_sub(self.3),
        )
    }
    pub fn is_diff_range(&self) -> bool {
        (self.0 as i8 <= 1 && self.0 as i8 >= -2)
            && (self.1 as i8 <= 1 && self.1 as i8 >= -2)
            && (self.2 as i8 <= 1 && self.2 as i8 >= -2)
            && self.3 == 0
    }
    pub fn is_luma_range(&self) -> bool {
        let dr_dg = self.0.wrapping_sub(self.1) as i8;
        let db_dg = self.2.wrapping_sub(self.1)as i8;
        (dr_dg <= 7 && dr_dg >= -8)
            && (self.1 as i8 <= 31 && self.1 as i8 >= -32)
            && (db_dg <= 7 && db_dg >= -8)
            && self.3 == 0
    }
}