use crate::Beatmap;

#[derive(Clone, Debug)]
pub(crate) enum MapOrElse<M, E> {
    Map(M),
    Else(E),
}

impl<M, E> MapOrElse<M, E>
where
    M: AsRef<Beatmap>,
{
    /// Return a mutable reference to `Else`.
    ///
    /// If `self` is of variant `Map`, store `other` in `self`, and return a
    /// mutable reference to it.
    pub(crate) fn else_or_insert(&mut self, other: E) -> &mut E {
        match self {
            MapOrElse::Map(_) => {
                *self = Self::Else(other);

                let Self::Else(ref mut other) = self else {
                    unreachable!()
                };

                other
            }
            MapOrElse::Else(ref mut other) => other,
        }
    }
}

impl<'map, E> From<&'map Beatmap> for MapOrElse<MapRef<'map>, E> {
    fn from(map: &'map Beatmap) -> Self {
        Self::Map(MapRef(map))
    }
}

/// References don't implement [`Deref`] so we implement a wrapper type.
#[derive(Copy, Clone, Debug)]
pub(crate) struct MapRef<'map>(&'map Beatmap);

impl<'map> MapRef<'map> {
    pub(crate) fn into_inner(self) -> &'map Beatmap {
        self.0
    }
}

impl AsRef<Beatmap> for MapRef<'_> {
    fn as_ref(&self) -> &Beatmap {
        self.0
    }
}
