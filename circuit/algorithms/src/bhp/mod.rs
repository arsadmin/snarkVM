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

mod hasher;
use hasher::BHPHasher;

mod commit;
mod commit_uncompressed;
mod hash;
mod hash_uncompressed;

#[cfg(all(test, console))]
use snarkvm_circuit_types::environment::assert_scope;

use crate::{Commit, CommitUncompressed, Hash, HashUncompressed};
use snarkvm_circuit_types::prelude::*;

/// BHP256 is a collision-resistant hash function that takes a 256-bit input.
pub type BHP256<G> = BHP<G, 3, 57>; // Supports inputs up to 261 bits (1 u8 + 1 Fq).
/// BHP512 is a collision-resistant hash function that takes a 512-bit input.
pub type BHP512<G> = BHP<G, 6, 43>; // Supports inputs up to 522 bits (2 u8 + 2 Fq).
/// BHP768 is a collision-resistant hash function that takes a 768-bit input.
pub type BHP768<G> = BHP<G, 15, 23>; // Supports inputs up to 783 bits (3 u8 + 3 Fq).
/// BHP1024 is a collision-resistant hash function that takes a 1024-bit input.
pub type BHP1024<G> = BHP<G, 8, 54>; // Supports inputs up to 1044 bits (4 u8 + 4 Fq).

/// The BHP chunk size (this implementation is for a 3-bit BHP).
const BHP_CHUNK_SIZE: usize = 3;

/// BHP is a collision-resistant hash function that takes a variable-length input.
/// The BHP hash function does *not* behave like a random oracle, see Poseidon for one.
///
/// ## Design
/// The BHP hash function splits the given input into blocks, and processes them iteratively.
///
/// The first iteration is initialized as follows:
/// ```text
/// DIGEST_0 = BHP([ 0...0 || DOMAIN || LENGTH(INPUT) || INPUT[0..BLOCK_SIZE] ]);
/// ```
/// Each subsequent iteration is initialized as follows:
/// ```text
/// DIGEST_N+1 = BHP([ DIGEST_N[0..DATA_BITS] || INPUT[(N+1)*BLOCK_SIZE..(N+2)*BLOCK_SIZE] ]);
/// ```
pub struct BHP<E: Environment, const NUM_WINDOWS: u8, const WINDOW_SIZE: u8> {
    /// The domain separator for the BHP hash function.
    domain: Vec<Boolean<E>>,
    /// The internal BHP hasher used to process one iteration.
    hasher: BHPHasher<E, NUM_WINDOWS, WINDOW_SIZE>,
}

#[cfg(console)]
impl<E: Environment, const NUM_WINDOWS: u8, const WINDOW_SIZE: u8> Inject for BHP<E, NUM_WINDOWS, WINDOW_SIZE> {
    type Primitive = console::BHP<E::Affine, NUM_WINDOWS, WINDOW_SIZE>;

    /// Initializes a new instance of a BHP circuit with the given BHP variant.
    fn new(_mode: Mode, bhp: Self::Primitive) -> Self {
        // Ensure the given domain is within the allowed size in bits.
        let num_bits = bhp.domain().len();
        let max_bits = E::BaseField::size_in_data_bits() - 64; // 64 bits encode the length.
        if num_bits > max_bits {
            E::halt(format!("Domain cannot exceed {max_bits} bits, found {num_bits} bits"))
        } else if num_bits != max_bits {
            E::halt(format!("Domain was not padded to {max_bits} bits, found {num_bits} bits"))
        }

        // Initialize the domain.
        let domain = Vec::constant(bhp.domain().to_vec());

        // Initialize the BHP hasher.
        let hasher = BHPHasher::<E, NUM_WINDOWS, WINDOW_SIZE>::constant(bhp);

        Self { domain, hasher }
    }
}

#[cfg(all(test, console))]
mod tests {
    use super::*;
    use snarkvm_circuit_types::{environment::Circuit, Eject};
    use snarkvm_curves::{AffineCurve, ProjectiveCurve};

    use anyhow::Result;

    const ITERATIONS: usize = 10;
    const MESSAGE: &str = "BHPCircuit0";

    #[test]
    fn test_setup_constant() -> Result<()> {
        for _ in 0..ITERATIONS {
            let native = console::BHP::<<Circuit as Environment>::Affine, 8, 32>::setup(MESSAGE)?;
            let circuit = BHP::<Circuit, 8, 32>::new(Mode::Constant, native.clone());

            native.bases().iter().zip(circuit.hasher.bases().iter()).for_each(|(native_bases, circuit_bases)| {
                native_bases.iter().zip(circuit_bases).for_each(|(native_base, circuit_base_lookups)| {
                    // Check the first circuit base (when converted back to twisted Edwards) matches the native one.
                    let (circuit_x, circuit_y) = {
                        let (x_bases, y_bases) = circuit_base_lookups;
                        // Convert the Montgomery point to a twisted Edwards point.
                        let edwards_x = &x_bases[0] / &y_bases[0]; // 1 constraint
                        let edwards_y = (&x_bases[0] - Field::one()) / (&x_bases[0] + Field::one());
                        (edwards_x, edwards_y)
                    };
                    assert_eq!(native_base.to_affine().to_x_coordinate(), circuit_x.eject_value());
                    assert_eq!(native_base.to_affine().to_y_coordinate(), circuit_y.eject_value());
                })
            });
        }
        Ok(())
    }
}
