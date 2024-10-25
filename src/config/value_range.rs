use serde::Deserialize;
use std::ops::{Add, Sub};

#[derive(Debug, Deserialize, Copy, Clone)]
pub struct ValueRange<T> {
    pub value: T,
    pub min: T,
    pub max: T,
    #[serde(alias = "step")]
    pub increment_step: T,
}

impl<T> ValueRange<T>
where
    T: Copy + Add<Output = T> + Sub<Output = T> + PartialOrd,
{
    pub fn increment(&mut self) {
        self.value = num_traits::clamp(self.value.add(self.increment_step), self.min, self.max);
    }

    pub fn decrement(&mut self) {
        self.value = num_traits::clamp(self.value.sub(self.increment_step), self.min, self.max);
    }
}
