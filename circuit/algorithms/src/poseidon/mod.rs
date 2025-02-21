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

mod hash;
mod hash_many;
mod hash_to_scalar;
mod prf;

#[cfg(all(test, console))]
use snarkvm_circuit_types::environment::assert_scope;

use crate::{Hash, HashMany, HashToScalar, PRF};
use snarkvm_circuit_types::{environment::prelude::*, Field, Scalar};

/// Poseidon2 is a cryptographic hash function of input rate 2.
pub type Poseidon2<E> = Poseidon<E, 2>;
/// Poseidon4 is a cryptographic hash function of input rate 4.
pub type Poseidon4<E> = Poseidon<E, 4>;
/// Poseidon8 is a cryptographic hash function of input rate 8.
pub type Poseidon8<E> = Poseidon<E, 8>;

const CAPACITY: usize = 1;

/// The mode structure for duplex sponges.
#[derive(PartialEq, Eq, Clone, Debug)]
pub enum DuplexSpongeMode {
    /// The sponge is currently absorbing data.
    Absorbing {
        /// The next position of the state to be XOR-ed when absorbing.
        next_absorb_index: usize,
    },
    /// The sponge is currently squeezing data out.
    Squeezing {
        /// The next position of the state to be outputted when squeezing.
        next_squeeze_index: usize,
    },
}

#[derive(Clone)]
pub struct Poseidon<E: Environment, const RATE: usize> {
    /// The domain separator for the Poseidon hash function.
    domain: Field<E>,
    /// The number of rounds in a full-round operation.
    full_rounds: usize,
    /// The number of rounds in a partial-round operation.
    partial_rounds: usize,
    /// The exponent used in S-boxes.
    alpha: Field<E>,
    /// The additive round keys. These are added before each MDS matrix application to make it an affine shift.
    /// They are indexed by `ark[round_number][state_element_index]`
    ark: Vec<Vec<Field<E>>>,
    /// The Maximally Distance Separating (MDS) matrix.
    mds: Vec<Vec<Field<E>>>,
}

#[cfg(console)]
impl<E: Environment, const RATE: usize> Inject for Poseidon<E, RATE> {
    type Primitive = console::Poseidon<E::BaseField, RATE>;

    fn new(_mode: Mode, poseidon: Self::Primitive) -> Self {
        // Initialize the domain separator.
        let domain = Field::constant(poseidon.domain());

        // Initialize the Poseidon parameters.
        let parameters = poseidon.parameters();
        let full_rounds = parameters.full_rounds;
        let partial_rounds = parameters.partial_rounds;
        let alpha = Field::constant(E::BaseField::from(parameters.alpha as u128));
        // Cache the bits for the field element.
        alpha.to_bits_le();
        let ark = parameters
            .ark
            .iter()
            .take(full_rounds + partial_rounds)
            .map(|round| round.iter().take(RATE + 1).cloned().map(Field::constant).collect())
            .collect();
        let mds = parameters
            .mds
            .iter()
            .take(RATE + 1)
            .map(|round| round.iter().take(RATE + 1).cloned().map(Field::constant).collect())
            .collect();

        Self { domain, full_rounds, partial_rounds, alpha, ark, mds }
    }
}
