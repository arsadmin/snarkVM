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

mod commit;
mod commit_uncompressed;
mod hash;
mod hash_uncompressed;

use crate::{Blake2Xs, Commit, CommitUncompressed, Hash, HashUncompressed};
use snarkvm_curves::{AffineCurve, ProjectiveCurve};
use snarkvm_fields::{PrimeField, Zero};
use snarkvm_utilities::ToBits;

use anyhow::{bail, Result};
use itertools::Itertools;
use std::{borrow::Cow, sync::Arc};

/// Pedersen64 is an *additively-homomorphic* collision-resistant hash function that takes a 64-bit input.
pub type Pedersen64<G> = Pedersen<G, 64>;
/// Pedersen128 is an *additively-homomorphic* collision-resistant hash function that takes a 128-bit input.
pub type Pedersen128<G> = Pedersen<G, 128>;

/// Pedersen is a collision-resistant hash function that takes a variable-length input.
/// The Pedersen hash function does *not* behave like a random oracle, see Poseidon for one.
#[derive(Clone)]
pub struct Pedersen<G: AffineCurve, const NUM_BITS: u8> {
    /// The base window for the Pedersen hash.
    base_window: Arc<Vec<G::Projective>>,
    /// The random base window for the Pedersen commitment.
    random_base_window: Arc<Vec<G::Projective>>,
}

impl<G: AffineCurve, const NUM_BITS: u8> Pedersen<G, NUM_BITS> {
    /// Initializes a new instance of Pedersen with the given setup message.
    pub fn setup(message: &str) -> Self {
        // Construct an indexed message to attempt to sample a base.
        let (generator, _, _) = Blake2Xs::hash_to_curve::<G>(&format!("Aleo.Pedersen.Base.{message}"));
        let mut base_power = generator.to_projective();
        // Construct the window with the base.
        let mut base_window = vec![G::Projective::zero(); NUM_BITS as usize];
        for base in base_window.iter_mut().take(NUM_BITS as usize) {
            *base = base_power;
            base_power.double_in_place();
        }
        assert_eq!(base_window.len(), NUM_BITS as usize);

        // Compute the random base.
        let (generator, _, _) = Blake2Xs::hash_to_curve::<G>(&format!("Aleo.Pedersen.RandomBase.{message}"));
        let mut base = generator.to_projective();
        // Construct the window with the random base.
        let num_scalar_bits = G::ScalarField::size_in_bits();
        let mut random_base = Vec::with_capacity(num_scalar_bits);
        for _ in 0..num_scalar_bits {
            random_base.push(base);
            base.double_in_place();
        }
        assert_eq!(random_base.len(), num_scalar_bits);

        Self { base_window: Arc::new(base_window.to_vec()), random_base_window: Arc::new(random_base) }
    }

    /// Returns the base window.
    pub fn base_window(&self) -> &Arc<Vec<G::Projective>> {
        &self.base_window
    }

    /// Returns the random base window.
    pub fn random_base_window(&self) -> &Arc<Vec<G::Projective>> {
        &self.random_base_window
    }
}
