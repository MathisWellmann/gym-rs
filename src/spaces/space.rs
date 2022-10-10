/// An interface that deals with subspaces.
pub trait Space<T>
where
    T: PartialOrd + PartialEq,
{
    /// Checks for the existence of a value within the defined subspace.
    ///
    /// Returns `true` if the value exists, `false` otherwise.
    fn contains(&self, value: T) -> bool;
}
