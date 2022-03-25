// Copyright (C) 2019-2022 Aleo Systems Inc.
// This file is part of the snarkVM library.

// The snarkVM library is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// The snarkVM library is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with the snarkVM library. If not, see <https://www.gnu.org/licenses/>.

use super::*;
use snarkvm_utilities::{FromBytes, ToBytes};

impl<E: Environment> FromBits for Scalar<E> {
    type Boolean = Boolean<E>;

    /// Initializes a new scalar field element from a list of little-endian bits *without* trailing zeros.
    fn from_bits_le(bits_le: &[Self::Boolean]) -> Self {
        // Retrieve the data and scalar size.
        let size_in_data_bits = E::ScalarField::size_in_data_bits();
        let size_in_bits = E::ScalarField::size_in_bits();

        // Ensure the list of booleans is within the allowed size in bits.
        let num_bits = bits_le.len();
        if num_bits > size_in_bits {
            // Check if all excess bits are zero.
            let should_be_zero = bits_le[size_in_bits..].iter().fold(Boolean::constant(false), |acc, bit| acc | bit);
            // Ensure `should_be_zero` is zero.
            E::assert_eq(E::zero(), should_be_zero);
        }

        // Construct the sanitized list of bits, resizing up if necessary.
        let mut bits_le = bits_le.iter().take(size_in_bits).cloned().collect::<Vec<_>>();
        bits_le.resize(size_in_bits, Boolean::constant(false));

        // Construct the candidate scalar field element.
        let output = Scalar { bits_le };

        // If the number of bits is equivalent to the scalar size in bits,
        // ensure the scalar is below the scalar field modulus.
        if num_bits > size_in_data_bits {
            // Retrieve the modulus & subtract by 1 as we'll check `output.bits_le` is less than or *equal* to this value.
            // (For advanced users) ScalarField::MODULUS - 1 is equivalent to -1 in the field.
            let modulus_minus_one = -E::ScalarField::one();
            let modulus_minus_one_bits_le = modulus_minus_one.to_bits_le();

            // Initialize an iterator over `self` and `other` from MSB to LSB.
            let bit_pairs_le = modulus_minus_one_bits_le.iter().zip_eq(bits_le);

            let modulus_minus_one_less_than_bits =
                bit_pairs_le.fold(Boolean::constant(false), |rest_is_less, (modulus_minus_one_bit, other_bit)| {
                    if *modulus_minus_one_bit {
                        Boolean::ternary(&!other_bit, other_bit, &rest_is_less)
                    } else {
                        Boolean::ternary(other_bit, other_bit, &rest_is_less)
                    }
                });

            // Enforce that ScalarField::MODULUS - 1 is not less than the field element given by `bits_le`.
            // In other words, enforce that ScalarField::MODULUS - 1 is greater than or equal to the field element given by `bits_le`.
            E::assert(!modulus_minus_one_less_than_bits);
        }

        output
    }

    /// Initializes a new scalar field element from a list of big-endian bits *without* leading zeros.
    fn from_bits_be(bits_be: &[Self::Boolean]) -> Self {
        // Reverse the given bits from big-endian into little-endian.
        // Note: This is safe as the bit representation is consistent (there are no leading zeros).
        let mut bits_le = bits_be.to_vec();
        bits_le.reverse();

        Self::from_bits_le(&bits_le)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use snarkvm_circuits_environment::Circuit;
    use snarkvm_utilities::{test_rng, UniformRand};

    const ITERATIONS: usize = 100;

    // TODO: Need to test for case where bits are greater than the scalar field modulus.

    fn check_from_bits_le(
        mode: Mode,
        num_constants: usize,
        num_public: usize,
        num_private: usize,
        num_constraints: usize,
    ) {
        for i in 0..ITERATIONS {
            // Sample a random element.
            let expected: <Circuit as Environment>::ScalarField = UniformRand::rand(&mut test_rng());
            let given_bits = Scalar::<Circuit>::new(mode, expected).to_bits_le();
            let expected_size_in_bits = given_bits.len();

            Circuit::scope(&format!("{} {}", mode, i), || {
                let candidate = Scalar::<Circuit>::from_bits_le(&given_bits);
                assert_eq!(expected, candidate.eject_value());
                assert_eq!(expected_size_in_bits, candidate.bits_le.len());
                assert_scope!(num_constants, num_public, num_private, num_constraints);
            });

            // Add excess zero bits.
            let candidate = vec![given_bits, vec![Boolean::new(mode, false); i]].concat();

            Circuit::scope(&format!("Excess {} {}", mode, i), || {
                let candidate = Scalar::<Circuit>::from_bits_le(&candidate);
                assert_eq!(expected, candidate.eject_value());
                assert_eq!(expected_size_in_bits, candidate.bits_le.len());
                match mode.is_constant() {
                    true => assert_scope!(num_constants, num_public, num_private, num_constraints),
                    // `num_private` gets 1 free excess bit, then is incremented by one for each excess bit.
                    // `num_constraints` is incremented by one for each excess bit.
                    false => {
                        assert_scope!(num_constants, num_public, num_private + i.saturating_sub(1), num_constraints + i)
                    }
                };
            });
        }
    }

    fn check_from_bits_be(
        mode: Mode,
        num_constants: usize,
        num_public: usize,
        num_private: usize,
        num_constraints: usize,
    ) {
        for i in 0..ITERATIONS {
            // Sample a random element.
            let expected: <Circuit as Environment>::ScalarField = UniformRand::rand(&mut test_rng());
            let given_bits = Scalar::<Circuit>::new(mode, expected).to_bits_be();
            let expected_size_in_bits = given_bits.len();

            Circuit::scope(&format!("{} {}", mode, i), || {
                let candidate = Scalar::<Circuit>::from_bits_be(&given_bits);
                assert_eq!(expected, candidate.eject_value());
                assert_eq!(expected_size_in_bits, candidate.to_bits_be().len());
                assert_scope!(num_constants, num_public, num_private, num_constraints);
            });

            // Add excess zero bits.
            let candidate = vec![vec![Boolean::new(mode, false); i], given_bits].concat();

            Circuit::scope(&format!("Excess {} {}", mode, i), || {
                let candidate = Scalar::<Circuit>::from_bits_be(&candidate);
                assert_eq!(expected, candidate.eject_value());
                assert_eq!(expected_size_in_bits, candidate.bits_le.len());
                match mode.is_constant() {
                    true => assert_scope!(num_constants, num_public, num_private, num_constraints),
                    // `num_private` gets 1 free excess bit, then is incremented by one for each excess bit.
                    // `num_constraints` is incremented by one for each excess bit.
                    false => {
                        assert_scope!(num_constants, num_public, num_private + i.saturating_sub(1), num_constraints + i)
                    }
                };
            });
        }
    }

    #[test]
    fn test_from_bits_le_constant() {
        check_from_bits_le(Mode::Constant, 0, 0, 0, 0);
    }

    #[test]
    fn test_from_bits_le_public() {
        check_from_bits_le(Mode::Public, 0, 0, 251, 252);
    }

    #[test]
    fn test_from_bits_le_private() {
        check_from_bits_le(Mode::Private, 0, 0, 251, 252);
    }

    #[test]
    fn test_from_bits_be_constant() {
        check_from_bits_be(Mode::Constant, 0, 0, 0, 0);
    }

    #[test]
    fn test_from_bits_be_public() {
        check_from_bits_be(Mode::Public, 0, 0, 251, 252);
    }

    #[test]
    fn test_from_bits_be_private() {
        check_from_bits_be(Mode::Private, 0, 0, 251, 252);
    }
}
