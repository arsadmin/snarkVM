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

impl<E: Environment, I: IntegerType> Neg for Integer<E, I> {
    type Output = Integer<E, I>;

    /// Performs the unary `-` operation.
    fn neg(self) -> Self::Output {
        (&self).neg()
    }
}

impl<E: Environment, I: IntegerType> Neg for &Integer<E, I> {
    type Output = Integer<E, I>;

    /// Performs the unary `-` operation.
    fn neg(self) -> Self::Output {
        match I::is_signed() {
            // Note: This addition must be checked as `-I::MIN` is an invalid operation.
            true => Integer::one().add_checked(&!self),
            false => E::halt("Attempted to negate an unsigned integer"),
        }
    }
}

impl<E: Environment, I: IntegerType> Metrics<dyn Neg<Output = Integer<E, I>>> for Integer<E, I> {
    type Case = Mode;

    fn count(case: &Self::Case) -> Count {
        match I::is_signed() {
            false => E::halt("Unsigned integers cannot be negated"),
            true => match case {
                Mode::Constant => Count::is(2 * I::BITS, 0, 0, 0),
                _ => Count::is(I::BITS, 0, I::BITS + 2, I::BITS + 4),
            },
        }
    }
}

impl<E: Environment, I: IntegerType> OutputMode<dyn Neg<Output = Integer<E, I>>> for Integer<E, I> {
    type Case = Mode;

    fn output_mode(case: &Self::Case) -> Mode {
        match case {
            Mode::Constant => Mode::Constant,
            _ => Mode::Private,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use snarkvm_circuit_environment::Circuit;
    use snarkvm_utilities::{test_rng, UniformRand};
    use test_utilities::*;

    use core::{ops::RangeInclusive, panic::UnwindSafe};

    const ITERATIONS: u64 = 128;

    fn check_neg<I: IntegerType + UnwindSafe + Neg<Output = I>>(name: &str, value: I, mode: Mode) {
        let a = Integer::<Circuit, I>::new(mode, value);
        match value.checked_neg() {
            Some(expected) => Circuit::scope(name, || {
                let candidate = a.neg();
                assert_eq!(expected, candidate.eject_value());
                assert_count!(Neg(Integer<I>) => Integer<I>, &mode);
                assert_output_mode!(Neg(Integer<I>) => Integer<I>, &mode, candidate);
            }),
            None => match mode {
                Mode::Constant => check_unary_operation_halts(a, |a: Integer<Circuit, I>| a.neg()),
                _ => Circuit::scope(name, || {
                    let _candidate = a.neg();
                    assert_count_fails!(Neg(Integer<I>) => Integer<I>, &mode);
                }),
            },
        }
        Circuit::reset();
    }

    fn run_test<I: IntegerType + UnwindSafe + Neg<Output = I>>(mode: Mode) {
        // Check the 0 case.
        check_neg(&format!("Neg: {} zero", mode), I::zero(), mode);
        // Check the 1 case.
        check_neg(&format!("Neg: {} one", mode), -I::one(), mode);
        // Check random values.
        for i in 0..ITERATIONS {
            let value: I = UniformRand::rand(&mut test_rng());
            check_neg(&format!("Neg: {} {}", mode, i), value, mode);
        }
    }

    fn assert_unsigned_neg_halts<I: IntegerType + UnwindSafe>(mode: Mode) {
        let candidate = Integer::<Circuit, I>::new(mode, UniformRand::rand(&mut test_rng()));
        let operation = std::panic::catch_unwind(|| candidate.neg());
        assert!(operation.is_err());
    }

    fn run_exhaustive_test<I: IntegerType + UnwindSafe + Neg<Output = I>>(mode: Mode)
    where
        RangeInclusive<I>: Iterator<Item = I>,
    {
        for value in I::MIN..=I::MAX {
            let name = format!("Neg: {}", mode);
            check_neg(&name, value, mode);
        }
    }

    test_integer_unary!(run_test, i8, neg);
    test_integer_unary!(run_test, i16, neg);
    test_integer_unary!(run_test, i32, neg);
    test_integer_unary!(run_test, i64, neg);
    test_integer_unary!(run_test, i128, neg);

    test_integer_unary!(assert_unsigned_neg_halts, u8, neg);
    test_integer_unary!(assert_unsigned_neg_halts, u16, neg);
    test_integer_unary!(assert_unsigned_neg_halts, u32, neg);
    test_integer_unary!(assert_unsigned_neg_halts, u64, neg);
    test_integer_unary!(assert_unsigned_neg_halts, u128, neg);

    test_integer_unary!(#[ignore], assert_unsigned_neg_halts, u8, neg, exhaustive);
    test_integer_unary!(#[ignore], run_exhaustive_test, i8, neg, exhaustive);
}
