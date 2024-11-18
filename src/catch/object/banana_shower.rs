pub struct BananaShower {
    pub n_bananas: usize,
}

impl BananaShower {
    pub fn new(start_time: f64, end_time: f64) -> Self {
        // * Int truncation added to match osu!stable.
        let start_time = start_time as i32;
        let end_time = end_time as i32;
        let mut spacing = (end_time - start_time) as f32;

        while spacing > 100.0 {
            spacing /= 2.0;
        }

        let n_bananas = if spacing <= 0.0 {
            0
        } else {
            let end_time = end_time as f32;
            let mut time = start_time as f32;
            let mut count = 0;

            while time <= end_time {
                time += spacing;
                count += 1;
            }

            count
        };

        Self { n_bananas }
    }
}
