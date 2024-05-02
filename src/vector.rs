use anyhow::{anyhow, Result};
use std::{
    fmt::Debug,
    ops::{Add, AddAssign, Deref, Mul},
};

#[derive(Debug)]
pub struct Vector<T> {
    data: Vec<T>,
}

impl<T> Deref for Vector<T> {
    type Target = Vec<T>;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl<T> Vector<T> {
    pub fn new(data: impl Into<Vec<T>>) -> Self {
        Self { data: data.into() }
    }
}

pub fn dot_product<T>(a: &Vector<T>, b: &Vector<T>) -> Result<T>
where
    T: Debug + Default + Copy + Add<Output = T> + AddAssign + Mul<Output = T>,
{
    if a.len() != b.len() {
        return Err(anyhow!("a.len not equal b.len"));
    }
    let mut sum = T::default();
    for i in 0..a.len() {
        sum += a[i] * b[i]
    }
    Ok(sum)
}
