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

impl<E: Environment, I: IntegerType> Compare<Self> for Integer<E, I> {
    type Output = Boolean<E>;

    /// Returns `true` if `self` is less than `other`.
    fn is_less_than(&self, other: &Self) -> Self::Output {
        // Determine the variable mode.
        if self.is_constant() && other.is_constant() {
            // Compute the comparison and return the new constant.
            Self::Output::new(Mode::Constant, self.eject_value() < other.eject_value())
        } else if I::is_signed() {
            // Compute the less than operation via a sign and overflow check.
            // If sign(a) != sign(b), then a < b, if a is negative and b is positive.
            // If sign(b) == sign(a), then a < b if the carry bit of I::NEG_ONE + a - b + 1 is set.
            let same_sign = self.msb().is_equal(other.msb());
            let self_is_negative_and_other_is_positive = self.msb() & !other.msb();
            let negative_one_plus_difference_plus_one =
                Integer::constant(I::zero() - I::one()).to_field() + self.to_field() - other.to_field() + Field::one();
            match negative_one_plus_difference_plus_one.to_lower_bits_le(I::BITS as usize + 1).last() {
                Some(bit) => Self::Output::ternary(&same_sign, &!bit, &self_is_negative_and_other_is_positive),
                None => E::halt("Malformed expression detected during signed integer comparison."),
            }
        } else {
            // Compute the less than operation via an overflow check.
            // If I::MAX + a - b + 1 overflows, then a >= b, otherwise a < b.
            let max_plus_difference_plus_one =
                Integer::constant(I::MAX).to_field() + self.to_field() - other.to_field() + Field::one();
            match max_plus_difference_plus_one.to_lower_bits_le(I::BITS as usize + 1).last() {
                Some(bit) => !bit,
                None => E::halt("Malformed expression detected during unsigned integer comparison."),
            }
        }
    }

    /// Returns `true` if `self` is greater than `other`.
    fn is_greater_than(&self, other: &Self) -> Self::Output {
        other.is_less_than(self)
    }

    /// Returns `true` if `self` is less than or equal to `other`.
    fn is_less_than_or_equal(&self, other: &Self) -> Self::Output {
        other.is_greater_than_or_equal(self)
    }

    /// Returns `true` if `self` is greater than or equal to `other`.
    fn is_greater_than_or_equal(&self, other: &Self) -> Self::Output {
        !self.is_less_than(other)
    }
}

impl<E: Environment, I: IntegerType> Metrics<dyn Compare<Integer<E, I>, Output = Boolean<E>>> for Integer<E, I> {
    type Case = (Mode, Mode);

    fn count(case: &Self::Case) -> Count {
        match I::is_signed() {
            true => match (case.0, case.1) {
                (Mode::Constant, Mode::Constant) => Count::is(1, 0, 0, 0),
                (Mode::Constant, _) | (_, Mode::Constant) => Count::is(I::BITS, 0, I::BITS + 2, I::BITS + 3),
                (_, _) => Count::is(I::BITS, 0, I::BITS + 4, I::BITS + 5),
            },
            false => match (case.0, case.1) {
                (Mode::Constant, Mode::Constant) => Count::is(1, 0, 0, 0),
                (_, _) => Count::is(I::BITS, 0, I::BITS + 1, I::BITS + 2),
            },
        }
    }
}

impl<E: Environment, I: IntegerType> OutputMode<dyn Compare<Integer<E, I>, Output = Boolean<E>>> for Integer<E, I> {
    type Case = (Mode, Mode);

    fn output_mode(case: &Self::Case) -> Mode {
        match (case.0, case.1) {
            (Mode::Constant, Mode::Constant) => Mode::Constant,
            (_, _) => Mode::Private,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use snarkvm_circuit_environment::Circuit;
    use snarkvm_utilities::{test_rng, UniformRand};

    use std::ops::RangeInclusive;

    const ITERATIONS: u64 = 100;

    fn check_compare<I: IntegerType>(name: &str, first: I, second: I, mode_a: Mode, mode_b: Mode) {
        let a = Integer::<Circuit, I>::new(mode_a, first);
        let b = Integer::<Circuit, I>::new(mode_b, second);

        // Check `is_less_than`.
        let expected = first < second;
        Circuit::scope(name, || {
            let candidate = a.is_less_than(&b);
            assert_eq!(expected, candidate.eject_value());
            assert_count!(Compare(Integer<I>, Integer<I>) => Boolean, &(mode_a, mode_b));
            assert_output_mode!(Compare(Integer<I>, Integer<I>) => Boolean, &(mode_a, mode_b), candidate);
        });
        Circuit::reset();

        // Check `is_less_than_or_equal`
        let expected = first <= second;
        Circuit::scope(name, || {
            let candidate = a.is_less_than_or_equal(&b);
            assert_eq!(expected, candidate.eject_value());
            assert_count!(Compare(Integer<I>, Integer<I>) => Boolean, &(mode_a, mode_b));
            assert_output_mode!(Compare(Integer<I>, Integer<I>) => Boolean, &(mode_a, mode_b), candidate);
        });
        Circuit::reset();

        // Check `is_greater_than`
        let expected = first > second;
        Circuit::scope(name, || {
            let candidate = a.is_greater_than(&b);
            assert_eq!(expected, candidate.eject_value());
            assert_count!(Compare(Integer<I>, Integer<I>) => Boolean, &(mode_a, mode_b));
            assert_output_mode!(Compare(Integer<I>, Integer<I>) => Boolean, &(mode_a, mode_b), candidate);
        });
        Circuit::reset();

        // Check `is_greater_than_or_equal`
        let expected = first >= second;
        Circuit::scope(name, || {
            let candidate = a.is_greater_than_or_equal(&b);
            assert_eq!(expected, candidate.eject_value());
            assert_count!(Compare(Integer<I>, Integer<I>) => Boolean, &(mode_a, mode_b));
            assert_output_mode!(Compare(Integer<I>, Integer<I>) => Boolean, &(mode_a, mode_b), candidate);
        });
        Circuit::reset();
    }

    fn run_test<I: IntegerType>(mode_a: Mode, mode_b: Mode) {
        for i in 0..ITERATIONS {
            let first: I = UniformRand::rand(&mut test_rng());
            let second: I = UniformRand::rand(&mut test_rng());

            let name = format!("Compare: ({}, {}) - {}th iteration", mode_a, mode_b, i);
            check_compare(&name, first, second, mode_a, mode_b);
        }
    }

    fn run_exhaustive_test<I: IntegerType>(mode_a: Mode, mode_b: Mode)
    where
        RangeInclusive<I>: Iterator<Item = I>,
    {
        for first in I::MIN..=I::MAX {
            for second in I::MIN..=I::MAX {
                let name = format!("Compare: ({}, {})", first, second);
                check_compare(&name, first, second, mode_a, mode_b);
            }
        }
    }

    test_integer_binary!(run_test, i8, compare_with);
    test_integer_binary!(run_test, i16, compare_with);
    test_integer_binary!(run_test, i32, compare_with);
    test_integer_binary!(run_test, i64, compare_with);
    test_integer_binary!(run_test, i128, compare_with);

    test_integer_binary!(run_test, u8, compare_with);
    test_integer_binary!(run_test, u16, compare_with);
    test_integer_binary!(run_test, u32, compare_with);
    test_integer_binary!(run_test, u64, compare_with);
    test_integer_binary!(run_test, u128, compare_with);

    test_integer_binary!(#[ignore], run_exhaustive_test, u8, bitand, exhaustive);
    test_integer_binary!(#[ignore], run_exhaustive_test, i8, bitand, exhaustive);
}
