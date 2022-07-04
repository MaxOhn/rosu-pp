const INT_TO_REAL: f64 = 1.0 / (i32::MAX as f64 + 1.0);
const INT_MASK: u32 = 0x7F_FF_FF_FF;

pub(crate) struct Random {
    x: u32,
    y: u32,
    z: u32,
    w: u32,
}

impl Random {
    pub(crate) fn new(seed: i32) -> Self {
        Self {
            x: seed as u32,
            y: 842_502_087,
            z: 3_579_807_591,
            w: 273_326_509,
        }
    }

    pub(crate) fn gen_unsigned(&mut self) -> u32 {
        let t = self.x ^ (self.x << 11);
        self.x = self.y;
        self.y = self.z;
        self.z = self.w;
        self.w = self.w ^ (self.w >> 19) ^ t ^ (t >> 8);

        self.w
    }

    pub(crate) fn gen_signed(&mut self) -> i32 {
        (INT_MASK & self.gen_unsigned()) as i32
    }

    pub(crate) fn gen_double(&mut self) -> f64 {
        INT_TO_REAL * self.gen_signed() as f64
    }

    pub(crate) fn gen_int_range(&mut self, min: i32, max: i32) -> i32 {
        (min as f64 + self.gen_double() * (max - min) as f64) as i32
    }
}
