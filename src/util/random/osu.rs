const INT_TO_REAL: f64 = 1.0 / (i32::MAX as f64 + 1.0);
const INT_MASK: u32 = 0x7F_FF_FF_FF;

pub struct Random {
    x: u32,
    y: u32,
    z: u32,
    w: u32,
    bit_buf: u32,
    bit_idx: i32,
}

impl Random {
    pub const fn new(seed: i32) -> Self {
        Self {
            x: seed as u32,
            y: 842_502_087,
            z: 3_579_807_591,
            w: 273_326_509,
            bit_buf: 0,
            bit_idx: 32,
        }
    }

    pub const fn gen_unsigned(&mut self) -> u32 {
        let t = self.x ^ (self.x << 11);
        self.x = self.y;
        self.y = self.z;
        self.z = self.w;
        self.w = self.w ^ (self.w >> 19) ^ t ^ (t >> 8);

        self.w
    }

    pub const fn next_int(&mut self) -> i32 {
        (INT_MASK & self.gen_unsigned()) as i32
    }

    pub fn next_double(&mut self) -> f64 {
        INT_TO_REAL * f64::from(self.next_int())
    }

    pub fn next_int_range(&mut self, min: i32, max: i32) -> i32 {
        (f64::from(min) + self.next_double() * f64::from(max - min)) as i32
    }

    pub fn next_double_range(&mut self, min: f64, max: f64) -> i32 {
        (min + self.next_double() * (max - min)) as i32
    }

    pub const fn next_bool(&mut self) -> bool {
        if self.bit_idx == 32 {
            self.bit_buf = self.gen_unsigned();
            self.bit_idx = 1;
        } else {
            self.bit_idx += 1;
            self.bit_buf >>= 1;
        }

        (self.bit_buf & 1) == 1
    }
}
