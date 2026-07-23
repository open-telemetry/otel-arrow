// Auto-generated inverse factor table: ln(2) / 2^scale
// for scales 1..=16. Indexed as INVERSE_FACTOR[scale - 1].

/// `ln(2) / 2^scale` for scales 1..=16, indexed as `[scale - 1]`.
/// Generated from 256-bit precision arithmetic, correctly rounded to f64.
const INVERSE_FACTOR: [f64; 16] = [
    crate::float64::from_bits(0x3FD62E42FEFA39EF), // scale 1 ≈ 3.465736e-1
    crate::float64::from_bits(0x3FC62E42FEFA39EF), // scale 2 ≈ 1.732868e-1
    crate::float64::from_bits(0x3FB62E42FEFA39EF), // scale 3 ≈ 8.664340e-2
    crate::float64::from_bits(0x3FA62E42FEFA39EF), // scale 4 ≈ 4.332170e-2
    crate::float64::from_bits(0x3F962E42FEFA39EF), // scale 5 ≈ 2.166085e-2
    crate::float64::from_bits(0x3F862E42FEFA39EF), // scale 6 ≈ 1.083042e-2
    crate::float64::from_bits(0x3F762E42FEFA39EF), // scale 7 ≈ 5.415212e-3
    crate::float64::from_bits(0x3F662E42FEFA39EF), // scale 8 ≈ 2.707606e-3
    crate::float64::from_bits(0x3F562E42FEFA39EF), // scale 9 ≈ 1.353803e-3
    crate::float64::from_bits(0x3F462E42FEFA39EF), // scale 10 ≈ 6.769015e-4
    crate::float64::from_bits(0x3F362E42FEFA39EF), // scale 11 ≈ 3.384508e-4
    crate::float64::from_bits(0x3F262E42FEFA39EF), // scale 12 ≈ 1.692254e-4
    crate::float64::from_bits(0x3F162E42FEFA39EF), // scale 13 ≈ 8.461269e-5
    crate::float64::from_bits(0x3F062E42FEFA39EF), // scale 14 ≈ 4.230635e-5
    crate::float64::from_bits(0x3EF62E42FEFA39EF), // scale 15 ≈ 2.115317e-5
    crate::float64::from_bits(0x3EE62E42FEFA39EF), // scale 16 ≈ 1.057659e-5
];
