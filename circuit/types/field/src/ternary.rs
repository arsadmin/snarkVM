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

impl<E: Environment> Ternary for Field<E> {
    type Boolean = Boolean<E>;
    type Output = Self;

    /// Returns `first` if `condition` is `true`, otherwise returns `second`.
    fn ternary(condition: &Self::Boolean, first: &Self, second: &Self) -> Self::Output {
        // Constant `condition`
        if condition.is_constant() {
            match condition.eject_value() {
                true => first.clone(),
                false => second.clone(),
            }
        }
        // Constant `first` and `second`
        else if first.is_constant() && second.is_constant() {
            let not_condition = Field::from_boolean(&!condition);
            let condition = Field::from_boolean(condition);
            (condition * first) + (not_condition * second)
        }
        // Variables
        else {
            // Initialize the witness.
            let witness = witness!(|condition, first, second| match condition {
                true => first,
                false => second,
            });

            //
            // Ternary Enforcement
            // -------------------------------------------------------
            //    witness = condition * a + (1 - condition) * b
            // => witness = b + condition * (a - b)
            // => condition * (a - b) = witness - b
            //
            //
            // Assumption
            // -------------------------------------------------------
            // If a == b, either values suffices as a valid witness,
            // and we may forgo the cases below. Else, we consider
            // the following four cases.
            //
            //
            // Case 1: condition = 0 AND witness = a (dishonest)
            // -------------------------------------------------------
            // 0 * (a - b) = a - b
            //           0 = a - b
            // => if a != b, as LHS != RHS, the witness is incorrect.
            //
            //
            // Case 2: condition = 0 AND witness = b (honest)
            // -------------------------------------------------------
            // 0 * (a - b) = b - b
            //           0 = 0
            // => as LHS == RHS, the witness is correct.
            //
            //
            // Case 3: condition = 1 AND witness = a (honest)
            // -------------------------------------------------------
            // 1 * (a - b) = a - b
            //       a - b = a - b
            // => as LHS == RHS, the witness is correct.
            //
            //
            // Case 4: condition = 1 AND witness = b (dishonest)
            // -------------------------------------------------------
            // 1 * (a - b) = b - b
            //       a - b = 0
            // => if a != b, as LHS != RHS, the witness is incorrect.
            //
            E::enforce(|| (condition, (first - second), (&witness - second)));

            witness
        }
    }
}

impl<E: Environment> Metrics<dyn Ternary<Boolean = Boolean<E>, Output = Field<E>>> for Field<E> {
    type Case = (Mode, Mode, Mode);

    fn count(case: &Self::Case) -> Count {
        match case {
            (Mode::Constant, _, _)
            | (Mode::Public, Mode::Constant, Mode::Constant)
            | (Mode::Private, Mode::Constant, Mode::Constant) => Count::is(0, 0, 0, 0),
            _ => Count::is(0, 0, 1, 1),
        }
    }
}

impl<E: Environment> OutputMode<dyn Ternary<Boolean = Boolean<E>, Output = Field<E>>> for Field<E> {
    type Case = (CircuitType<Boolean<E>>, Mode, Mode);

    fn output_mode(parameter: &Self::Case) -> Mode {
        match parameter.0.mode().is_constant() {
            true => match &parameter.0 {
                CircuitType::Constant(circuit) => match circuit.eject_value() {
                    true => parameter.1,
                    false => parameter.2,
                },
                _ => E::halt("Circuit is required to determine output mode."),
            },
            false => Mode::Private,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use snarkvm_circuit_environment::Circuit;
    use snarkvm_utilities::{test_rng, UniformRand};

    fn check_ternary(
        name: &str,
        expected: <Circuit as Environment>::BaseField,
        condition: Boolean<Circuit>,
        a: Field<Circuit>,
        b: Field<Circuit>,
    ) {
        Circuit::scope(name, || {
            let case = format!("({} ? {} : {})", condition.eject_value(), a.eject_value(), b.eject_value());
            let candidate = Field::ternary(&condition, &a, &b);
            assert_eq!(expected, candidate.eject_value(), "{case}");
            assert_count!(Ternary(Boolean, Field, Field) => Field, &(condition.eject_mode(), a.eject_mode(), b.eject_mode()));
            assert_output_mode!(Ternary(Boolean, Field, Field) => Field, &(CircuitType::from(&condition), a.eject_mode(), b.eject_mode()), candidate);
        });
    }

    #[test]
    fn test_constant_condition() {
        let first: <Circuit as Environment>::BaseField = UniformRand::rand(&mut test_rng());
        let second: <Circuit as Environment>::BaseField = UniformRand::rand(&mut test_rng());

        // false ? Constant : Constant
        let expected = second;
        let condition = Boolean::<Circuit>::new(Mode::Constant, false);
        let a = Field::<Circuit>::new(Mode::Constant, first);
        let b = Field::<Circuit>::new(Mode::Constant, second);
        check_ternary("false ? Constant : Constant", expected, condition, a, b);

        // false ? Constant : Public
        let expected = second;
        let condition = Boolean::<Circuit>::new(Mode::Constant, false);
        let a = Field::<Circuit>::new(Mode::Constant, first);
        let b = Field::<Circuit>::new(Mode::Public, second);
        check_ternary("false ? Constant : Public", expected, condition, a, b);

        // false ? Public : Constant
        let expected = second;
        let condition = Boolean::<Circuit>::new(Mode::Constant, false);
        let a = Field::<Circuit>::new(Mode::Public, first);
        let b = Field::<Circuit>::new(Mode::Constant, second);
        check_ternary("false ? Public : Constant", expected, condition, a, b);

        // false ? Public : Public
        let expected = second;
        let condition = Boolean::<Circuit>::new(Mode::Constant, false);
        let a = Field::<Circuit>::new(Mode::Public, first);
        let b = Field::<Circuit>::new(Mode::Public, second);
        check_ternary("false ? Public : Public", expected, condition, a, b);

        // false ? Public : Private
        let expected = second;
        let condition = Boolean::<Circuit>::new(Mode::Constant, false);
        let a = Field::<Circuit>::new(Mode::Public, first);
        let b = Field::<Circuit>::new(Mode::Private, second);
        check_ternary("false ? Public : Private", expected, condition, a, b);

        // false ? Private : Private
        let expected = second;
        let condition = Boolean::<Circuit>::new(Mode::Constant, false);
        let a = Field::<Circuit>::new(Mode::Private, first);
        let b = Field::<Circuit>::new(Mode::Private, second);
        check_ternary("false ? Private : Private", expected, condition, a, b);

        // true ? Constant : Constant
        let expected = first;
        let condition = Boolean::<Circuit>::new(Mode::Constant, true);
        let a = Field::<Circuit>::new(Mode::Constant, first);
        let b = Field::<Circuit>::new(Mode::Constant, second);
        check_ternary("true ? Constant : Constant", expected, condition, a, b);

        // true ? Constant : Public
        let expected = first;
        let condition = Boolean::<Circuit>::new(Mode::Constant, true);
        let a = Field::<Circuit>::new(Mode::Constant, first);
        let b = Field::<Circuit>::new(Mode::Public, second);
        check_ternary("true ? Constant : Public", expected, condition, a, b);

        // true ? Public : Constant
        let expected = first;
        let condition = Boolean::<Circuit>::new(Mode::Constant, true);
        let a = Field::<Circuit>::new(Mode::Public, first);
        let b = Field::<Circuit>::new(Mode::Constant, second);
        check_ternary("true ? Public : Constant", expected, condition, a, b);

        // true ? Public : Public
        let expected = first;
        let condition = Boolean::<Circuit>::new(Mode::Constant, true);
        let a = Field::<Circuit>::new(Mode::Public, first);
        let b = Field::<Circuit>::new(Mode::Public, second);
        check_ternary("true ? Public : Public", expected, condition, a, b);

        // true ? Public : Private
        let expected = first;
        let condition = Boolean::<Circuit>::new(Mode::Constant, true);
        let a = Field::<Circuit>::new(Mode::Public, first);
        let b = Field::<Circuit>::new(Mode::Private, second);
        check_ternary("true ? Public : Private", expected, condition, a, b);

        // true ? Private : Private
        let expected = first;
        let condition = Boolean::<Circuit>::new(Mode::Constant, true);
        let a = Field::<Circuit>::new(Mode::Private, first);
        let b = Field::<Circuit>::new(Mode::Private, second);
        check_ternary("true ? Private : Private", expected, condition, a, b);
    }

    #[test]
    fn test_public_condition_and_constant_inputs() {
        let first: <Circuit as Environment>::BaseField = UniformRand::rand(&mut test_rng());
        let second: <Circuit as Environment>::BaseField = UniformRand::rand(&mut test_rng());

        // false ? Constant : Constant
        let expected = second;
        let condition = Boolean::<Circuit>::new(Mode::Public, false);
        let a = Field::<Circuit>::new(Mode::Constant, first);
        let b = Field::<Circuit>::new(Mode::Constant, second);
        check_ternary("false ? Constant : Constant", expected, condition, a, b);

        // true ? Constant : Constant
        let expected = first;
        let condition = Boolean::<Circuit>::new(Mode::Public, true);
        let a = Field::<Circuit>::new(Mode::Constant, first);
        let b = Field::<Circuit>::new(Mode::Constant, second);
        check_ternary("true ? Constant : Constant", expected, condition, a, b);
    }

    #[test]
    fn test_public_condition_and_mixed_inputs() {
        let first: <Circuit as Environment>::BaseField = UniformRand::rand(&mut test_rng());
        let second: <Circuit as Environment>::BaseField = UniformRand::rand(&mut test_rng());

        // false ? Constant : Public
        let expected = second;
        let condition = Boolean::<Circuit>::new(Mode::Public, false);
        let a = Field::<Circuit>::new(Mode::Constant, first);
        let b = Field::<Circuit>::new(Mode::Public, second);
        check_ternary("false ? Constant : Public", expected, condition, a, b);

        // false ? Public : Constant
        let expected = second;
        let condition = Boolean::<Circuit>::new(Mode::Public, false);
        let a = Field::<Circuit>::new(Mode::Public, first);
        let b = Field::<Circuit>::new(Mode::Constant, second);
        check_ternary("false ? Public : Constant", expected, condition, a, b);

        // true ? Constant : Public
        let expected = first;
        let condition = Boolean::<Circuit>::new(Mode::Public, true);
        let a = Field::<Circuit>::new(Mode::Constant, first);
        let b = Field::<Circuit>::new(Mode::Public, second);
        check_ternary("true ? Constant : Public", expected, condition, a, b);

        // true ? Public : Constant
        let expected = first;
        let condition = Boolean::<Circuit>::new(Mode::Public, true);
        let a = Field::<Circuit>::new(Mode::Public, first);
        let b = Field::<Circuit>::new(Mode::Constant, second);
        check_ternary("true ? Public : Constant", expected, condition, a, b);
    }

    #[test]
    fn test_private_condition_and_constant_inputs() {
        let first: <Circuit as Environment>::BaseField = UniformRand::rand(&mut test_rng());
        let second: <Circuit as Environment>::BaseField = UniformRand::rand(&mut test_rng());

        // false ? Constant : Constant
        let expected = second;
        let condition = Boolean::<Circuit>::new(Mode::Private, false);
        let a = Field::<Circuit>::new(Mode::Constant, first);
        let b = Field::<Circuit>::new(Mode::Constant, second);
        check_ternary("false ? Constant : Constant", expected, condition, a, b);

        // true ? Constant : Constant
        let expected = first;
        let condition = Boolean::<Circuit>::new(Mode::Private, true);
        let a = Field::<Circuit>::new(Mode::Constant, first);
        let b = Field::<Circuit>::new(Mode::Constant, second);
        check_ternary("true ? Constant : Constant", expected, condition, a, b);
    }

    #[test]
    fn test_private_condition_and_mixed_inputs() {
        let first: <Circuit as Environment>::BaseField = UniformRand::rand(&mut test_rng());
        let second: <Circuit as Environment>::BaseField = UniformRand::rand(&mut test_rng());

        // false ? Constant : Public
        let expected = second;
        let condition = Boolean::<Circuit>::new(Mode::Private, false);
        let a = Field::<Circuit>::new(Mode::Constant, first);
        let b = Field::<Circuit>::new(Mode::Public, second);
        check_ternary("false ? Constant : Public", expected, condition, a, b);

        // false ? Public : Constant
        let expected = second;
        let condition = Boolean::<Circuit>::new(Mode::Private, false);
        let a = Field::<Circuit>::new(Mode::Public, first);
        let b = Field::<Circuit>::new(Mode::Constant, second);
        check_ternary("false ? Public : Constant", expected, condition, a, b);

        // true ? Constant : Public
        let expected = first;
        let condition = Boolean::<Circuit>::new(Mode::Private, true);
        let a = Field::<Circuit>::new(Mode::Constant, first);
        let b = Field::<Circuit>::new(Mode::Public, second);
        check_ternary("true ? Constant : Public", expected, condition, a, b);

        // true ? Public : Constant
        let expected = first;
        let condition = Boolean::<Circuit>::new(Mode::Private, true);
        let a = Field::<Circuit>::new(Mode::Public, first);
        let b = Field::<Circuit>::new(Mode::Constant, second);
        check_ternary("true ? Public : Constant", expected, condition, a, b);
    }

    #[test]
    fn test_public_condition_and_variable_inputs() {
        let first: <Circuit as Environment>::BaseField = UniformRand::rand(&mut test_rng());
        let second: <Circuit as Environment>::BaseField = UniformRand::rand(&mut test_rng());

        // false ? Public : Public
        let expected = second;
        let condition = Boolean::<Circuit>::new(Mode::Public, false);
        let a = Field::<Circuit>::new(Mode::Public, first);
        let b = Field::<Circuit>::new(Mode::Public, second);
        check_ternary("false ? Public : Public", expected, condition, a, b);

        // false ? Public : Private
        let expected = second;
        let condition = Boolean::<Circuit>::new(Mode::Public, false);
        let a = Field::<Circuit>::new(Mode::Public, first);
        let b = Field::<Circuit>::new(Mode::Private, second);
        check_ternary("false ? Public : Private", expected, condition, a, b);

        // false ? Private : Public
        let expected = second;
        let condition = Boolean::<Circuit>::new(Mode::Public, false);
        let a = Field::<Circuit>::new(Mode::Private, first);
        let b = Field::<Circuit>::new(Mode::Public, second);
        check_ternary("false ? Private : Public", expected, condition, a, b);

        // false ? Private : Private
        let expected = second;
        let condition = Boolean::<Circuit>::new(Mode::Public, false);
        let a = Field::<Circuit>::new(Mode::Private, first);
        let b = Field::<Circuit>::new(Mode::Private, second);
        check_ternary("false ? Private : Private", expected, condition, a, b);

        // true ? Public : Public
        let expected = first;
        let condition = Boolean::<Circuit>::new(Mode::Public, true);
        let a = Field::<Circuit>::new(Mode::Public, first);
        let b = Field::<Circuit>::new(Mode::Public, second);
        check_ternary("true ? Public : Public", expected, condition, a, b);

        // true ? Public : Private
        let expected = first;
        let condition = Boolean::<Circuit>::new(Mode::Public, true);
        let a = Field::<Circuit>::new(Mode::Public, first);
        let b = Field::<Circuit>::new(Mode::Private, second);
        check_ternary("true ? Public : Private", expected, condition, a, b);

        // true ? Private : Public
        let expected = first;
        let condition = Boolean::<Circuit>::new(Mode::Public, true);
        let a = Field::<Circuit>::new(Mode::Private, first);
        let b = Field::<Circuit>::new(Mode::Public, second);
        check_ternary("true ? Private : Public", expected, condition, a, b);

        // true ? Private : Private
        let expected = first;
        let condition = Boolean::<Circuit>::new(Mode::Public, true);
        let a = Field::<Circuit>::new(Mode::Private, first);
        let b = Field::<Circuit>::new(Mode::Private, second);
        check_ternary("true ? Private : Private", expected, condition, a, b);
    }

    #[test]
    fn test_private_condition_and_variable_inputs() {
        let first: <Circuit as Environment>::BaseField = UniformRand::rand(&mut test_rng());
        let second: <Circuit as Environment>::BaseField = UniformRand::rand(&mut test_rng());

        // false ? Public : Public
        let expected = second;
        let condition = Boolean::<Circuit>::new(Mode::Private, false);
        let a = Field::<Circuit>::new(Mode::Public, first);
        let b = Field::<Circuit>::new(Mode::Public, second);
        check_ternary("false ? Public : Public", expected, condition, a, b);

        // false ? Public : Private
        let expected = second;
        let condition = Boolean::<Circuit>::new(Mode::Private, false);
        let a = Field::<Circuit>::new(Mode::Public, first);
        let b = Field::<Circuit>::new(Mode::Private, second);
        check_ternary("false ? Public : Private", expected, condition, a, b);

        // false ? Private : Public
        let expected = second;
        let condition = Boolean::<Circuit>::new(Mode::Private, false);
        let a = Field::<Circuit>::new(Mode::Private, first);
        let b = Field::<Circuit>::new(Mode::Public, second);
        check_ternary("false ? Private : Public", expected, condition, a, b);

        // false ? Private : Private
        let expected = second;
        let condition = Boolean::<Circuit>::new(Mode::Private, false);
        let a = Field::<Circuit>::new(Mode::Private, first);
        let b = Field::<Circuit>::new(Mode::Private, second);
        check_ternary("false ? Private : Private", expected, condition, a, b);

        // true ? Public : Public
        let expected = first;
        let condition = Boolean::<Circuit>::new(Mode::Private, true);
        let a = Field::<Circuit>::new(Mode::Public, first);
        let b = Field::<Circuit>::new(Mode::Public, second);
        check_ternary("true ? Public : Public", expected, condition, a, b);

        // true ? Public : Private
        let expected = first;
        let condition = Boolean::<Circuit>::new(Mode::Private, true);
        let a = Field::<Circuit>::new(Mode::Public, first);
        let b = Field::<Circuit>::new(Mode::Private, second);
        check_ternary("true ? Public : Private", expected, condition, a, b);

        // true ? Private : Public
        let expected = first;
        let condition = Boolean::<Circuit>::new(Mode::Private, true);
        let a = Field::<Circuit>::new(Mode::Private, first);
        let b = Field::<Circuit>::new(Mode::Public, second);
        check_ternary("true ? Private : Public", expected, condition, a, b);

        // true ? Private : Private
        let expected = first;
        let condition = Boolean::<Circuit>::new(Mode::Private, true);
        let a = Field::<Circuit>::new(Mode::Private, first);
        let b = Field::<Circuit>::new(Mode::Private, second);
        check_ternary("true ? Private : Private", expected, condition, a, b);
    }
}
