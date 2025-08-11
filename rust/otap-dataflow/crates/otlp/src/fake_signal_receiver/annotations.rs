
use rand_distr::{Distribution, Normal, Pareto, Zipf};
use rand::distributions::Uniform;


pub struct IntConfig {
    distribution: Distribution
    range: Range

}

pub struct DoubleConfig {
    distriution: Distribution,
    range: Range
}

pub struct Range(f64, f64);


pub enum ValueDistribution {
    Pareto,
    Zipf,
    Normal,
}

impl Into<ValueDistribution> for impl Distribution

impl ValueDistribution {
    pub fn get_rand_distr(self) -> impl Distribution {
        match self {
            ValueDistribution::Pareto => Pareto,
            ValueDistribution::Zipf => Zipf
            ValueDistribution::Normal => Normal
        }
    }

}


impl IntConfig {
    pub fn new() -> Self {
        Self {

        }
    }

    pub fn get_value() -> u64 {

        let distribution = self.distribution.get_rand_distr()::new();
        let distribution
    }
}