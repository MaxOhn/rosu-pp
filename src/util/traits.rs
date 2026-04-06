use std::iter::Sum;

/// Mimics the C# `IEnumerable<T>` interface.
pub trait IEnumerable<T>: Sized {
    fn cs_where<F: FnMut(&T) -> bool>(self, f: F) -> Self;

    fn cs_sum<S>(&self) -> S
    where
        S: for<'a> Sum<&'a T>;
}

/// Mimics the C# `IOrderedEnumerable<T>` interface.
pub trait IOrderedEnumerable<T>: IEnumerable<T> {
    fn cs_order_descending(self) -> Self;
}

impl<T> IEnumerable<T> for Vec<T> {
    /// Filters a sequence of values based on a predicate.
    ///
    /// <https://learn.microsoft.com/en-us/dotnet/api/system.linq.enumerable.where>
    fn cs_where<F: FnMut(&T) -> bool>(mut self, f: F) -> Self {
        self.retain(f);

        self
    }

    /// Computes the sum of a sequence of numeric values.
    ///
    /// <https://learn.microsoft.com/en-us/dotnet/api/system.linq.enumerable.sum>
    fn cs_sum<S>(&self) -> S
    where
        S: for<'a> Sum<&'a T>,
    {
        self.iter().sum()
    }
}

impl IOrderedEnumerable<f64> for Vec<f64> {
    /// Sorts the elements of a sequence in descending order.
    ///
    /// This method performs a stable sort; that is, if the keys of two elements
    /// are equal, the order of the elements is preserved. In contrast, an unstable sort does not
    /// preserve the order of elements that have the same key.
    ///
    /// <https://learn.microsoft.com/en-us/dotnet/api/system.linq.enumerable.orderbydescending>
    fn cs_order_descending(mut self) -> Self {
        self.sort_by(|a, b| b.total_cmp(a));

        self
    }
}
