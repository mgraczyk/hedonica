use average::*;
use average::{Max, Min, Variance};

use serde;
use serde::ser::{SerializeStruct};

pub struct Stats {
    min: Min,
    max: Max,
    var: Variance,
}
impl Stats {
    pub fn new() -> Stats {
        Stats {
            min: Min::default(),
            max: Max::default(),
            var: Variance::default(),
        }
    }

    pub fn add(&mut self, x: f64) {
        self.min.add(x);
        self.max.add(x);
        self.var.add(x);
    }

    pub fn min(&self) -> f64 {
        self.min.min()
    }

    pub fn max(&self) -> f64 {
        self.max.max()
    }

    pub fn mean(&self) -> f64 {
        self.var.mean()
    }

    pub fn var(&self) -> f64 {
        self.var.population_variance()
    }
}

impl Default for Stats {
    fn default() -> Stats {
        Stats::new()
    }
}

impl_from_iterator!(Stats);

impl serde::Serialize for Stats {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut state = serializer.serialize_struct("Stats", 4)?;
        state.serialize_field("min",  &self.min())?;
        state.serialize_field("max",  &self.max())?;
        state.serialize_field("mean", &self.mean())?;
        state.serialize_field("var",  &self.var())?;
        state.end()
    }
}
