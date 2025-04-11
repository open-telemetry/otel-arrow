// SPDX-License-Identifier: Apache-2.0

//! OTAP Dataflow Configuration Model.

mod error;

/// A simple function that adds two numbers.
#[must_use]
pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
