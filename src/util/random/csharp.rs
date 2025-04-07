use crate::util::hint::unlikely;

// <https://github.com/dotnet/runtime/blob/5535e31a712343a63f5d7d796cd874e563e5ac14/src/libraries/System.Private.CoreLib/src/System/Random.cs#L13>
pub struct Random {
    prng: CompatPrng,
}

impl Random {
    // <https://github.com/dotnet/runtime/blob/5535e31a712343a63f5d7d796cd874e563e5ac14/src/libraries/System.Private.CoreLib/src/System/Random.cs#L41>
    pub fn new(seed: i32) -> Self {
        Self {
            // <https://github.com/dotnet/runtime/blob/15872212c29cecc8d82da4548c3060f2614665f7/src/libraries/System.Private.CoreLib/src/System/Random.CompatImpl.cs#L22>
            prng: CompatPrng::initialize(seed),
        }
    }

    // <https://github.com/dotnet/runtime/blob/15872212c29cecc8d82da4548c3060f2614665f7/src/libraries/System.Private.CoreLib/src/System/Random.CompatImpl.cs#L26>
    pub const fn next(&mut self) -> i32 {
        self.prng.internal_sample()
    }

    // <https://github.com/dotnet/runtime/blob/15872212c29cecc8d82da4548c3060f2614665f7/src/libraries/System.Private.CoreLib/src/System/Random.CompatImpl.cs#L28>
    pub fn next_max(&mut self, max: i32) -> i32 {
        (self.prng.sample() * f64::from(max)) as i32
    }
}

// <https://github.com/dotnet/runtime/blob/15872212c29cecc8d82da4548c3060f2614665f7/src/libraries/System.Private.CoreLib/src/System/Random.CompatImpl.cs#L256>
struct CompatPrng {
    seed_array: [i32; 56],
    inext: i32,
    inextp: i32,
}

impl CompatPrng {
    fn initialize(seed: i32) -> Self {
        let mut seed_array = [0; 56];

        let subtraction = if unlikely(seed == i32::MIN) {
            i32::MAX
        } else {
            i32::abs(seed)
        };

        let mut mj = 161_803_398 - subtraction; // * magic number based on Phi (golden ratio)
        seed_array[55] = mj;
        let mut mk = 1;
        let mut ii = 0;

        for _ in 1..55 {
            // * The range [1..55] is special (Knuth) and so we're wasting the 0'th position.
            ii += 21;

            if ii >= 55 {
                ii -= 55;
            }

            seed_array[ii] = mk;
            mk = mj - mk;
            if mk < 0 {
                mk += i32::MAX;
            }

            mj = seed_array[ii];
        }

        for _ in 1..5 {
            for i in 1..56 {
                let mut n = i + 30;

                if n >= 55 {
                    n -= 55;
                }

                seed_array[i] = seed_array[i].wrapping_sub(seed_array[1 + n]);

                if seed_array[i] < 0 {
                    seed_array[i] += i32::MAX;
                }
            }
        }

        Self {
            seed_array,
            inext: 0,
            inextp: 21,
        }
    }

    fn sample(&mut self) -> f64 {
        f64::from(self.internal_sample()) * (1.0 / f64::from(i32::MAX))
    }

    const fn internal_sample(&mut self) -> i32 {
        let mut loc_inext = self.inext;
        loc_inext += 1;

        if loc_inext >= 56 {
            loc_inext = 1;
        }

        let mut loc_inextp = self.inextp;
        loc_inextp += 1;

        if loc_inextp >= 56 {
            loc_inextp = 1;
        }

        let mut ret_val =
            self.seed_array[loc_inext as usize] - self.seed_array[loc_inextp as usize];

        if ret_val == i32::MAX {
            ret_val -= 1;
        }

        if ret_val < 0 {
            ret_val += i32::MAX;
        }

        self.seed_array[loc_inext as usize] = ret_val;
        self.inext = loc_inext;
        self.inextp = loc_inextp;

        ret_val
    }
}
