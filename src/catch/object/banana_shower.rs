pub struct BananaShower {
    pub n_bananas: usize,
}

impl BananaShower {
    pub fn new(start_time: f64, end_time: f64) -> Self {
        let duration = end_time - start_time;
        let mut spacing = duration;

        while spacing > 100.0 {
            spacing /= 2.0;
        }

        let n_bananas = if spacing <= 0.0 {
            0
        } else {
            let mut time = start_time;
            let mut i = 0;

            while time <= end_time {
                time += spacing;
                i += 1;
            }

            i
        };

        Self { n_bananas }
    }
}
