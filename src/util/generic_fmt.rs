use std::{
    any,
    fmt::{Debug, Formatter, Result as FmtResult},
    marker::PhantomData,
};

pub struct GenericFormatter<T>(PhantomData<T>);

impl<T> GenericFormatter<T> {
    pub const fn new() -> Self {
        Self(PhantomData)
    }
}

impl<T> Debug for GenericFormatter<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        fn fmt_stripped(full_type_name: &str, f: &mut Formatter<'_>) -> FmtResult {
            // Strip fully qualified syntax
            if let Some(position) = full_type_name.rfind("::") {
                if let Some(type_name) = full_type_name.get(position + 2..) {
                    f.write_str(type_name)?;
                }
            }

            Ok(())
        }

        fmt_stripped(any::type_name::<T>(), f)
    }
}
