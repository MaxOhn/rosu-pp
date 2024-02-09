pub use rosu_map::section::timing_points::DifficultyPoint;

pub fn difficulty_point_at(points: &[DifficultyPoint], time: f64) -> Option<&DifficultyPoint> {
    points
        .binary_search_by(|probe| probe.time.total_cmp(&time))
        .map_or_else(|i| i.checked_sub(1), Some)
        .map(|i| &points[i])
}
