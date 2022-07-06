use derive_new::new;
use serde::Serialize;

pub trait Space<T>
where
    T: PartialOrd + PartialEq,
{
    fn contains(&self, value: T) -> bool;
}

