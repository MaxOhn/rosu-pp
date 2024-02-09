use super::object::COMMON_RHYTHMS;

#[derive(Debug)]
pub struct HitObjectRhythm {
    pub id: u8,
    pub ratio: f64,
    pub difficulty: f64,
}

impl HitObjectRhythm {
    /// A way to get a default static reference to [`HitObjectRhythm`].
    pub fn static_ref() -> &'static Self {
        &COMMON_RHYTHMS[0]
    }
}

impl PartialEq for HitObjectRhythm {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for HitObjectRhythm {}
