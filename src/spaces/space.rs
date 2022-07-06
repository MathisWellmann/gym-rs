pub trait Space<T>
where
    T: PartialOrd + PartialEq,
{
    fn contains(&self, value: T) -> bool;
}
