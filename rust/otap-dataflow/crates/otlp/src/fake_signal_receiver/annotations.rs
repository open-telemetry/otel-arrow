
use rand_distr::{Distribution, Normal, Pareto, Zipf};
use rand::distributions::Uniform;


pub struct RandIntConfig {
    distribution: Distribution
}

pub struct RandDoubleConfig {
    distriution: Distribution,
}


pub enum ValueDistribution {
    Pareto(Pareto),
    Zipf(Zipf),
    Normal(Normal),
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



impl 

## Controlling signal data

### Attributes

For attributes there are three main types primative and array, Enums, and Template
 

#### Enum/Templates

select value from the examples


// attribute types ->
//enum -> use example
//template -> use example
//primitive or array ->
        // boolean
        // booleans
        //ints -> annotations
        // int -> annotations
        // doubles -> annotations
        // double -> annotations
        // string 
        // strings 


// metric values

    // define a range and distribution can use the same annotation config for ints and doubles for primitive attributes


