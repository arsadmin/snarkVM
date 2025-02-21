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

mod bytes;
mod serialize;
mod string;
mod try_from;

use crate::{ComputeKey, PrivateKey};
use snarkvm_console_network::Network;
use snarkvm_fields::PrimeField;
use snarkvm_utilities::{
    io::{Read, Result as IoResult, Write},
    FromBytes,
    FromBytesDeserializer,
    ToBytes,
    ToBytesSerializer,
};

use anyhow::{anyhow, bail, Error};
use base58::{FromBase58, ToBase58};
use core::{fmt, ops::Deref, str::FromStr};
use serde::{de, Deserialize, Deserializer, Serialize, Serializer};

/// The account view key used to decrypt records and ciphertext.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct ViewKey<N: Network>(N::Scalar);

impl<N: Network> Deref for ViewKey<N> {
    type Target = N::Scalar;

    /// Returns the account view key as a scalar.
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use snarkvm_console_network::Testnet3;
    use snarkvm_utilities::test_crypto_rng;

    use anyhow::Result;

    type CurrentNetwork = Testnet3;

    const ITERATIONS: u64 = 1000;

    #[test]
    fn test_deref() -> Result<()> {
        for _ in 0..ITERATIONS {
            // Sample a new address.
            let private_key = PrivateKey::<CurrentNetwork>::new(&mut test_crypto_rng())?;
            let expected = ViewKey::try_from(private_key)?;

            // Check the scalar representation.
            let candidate = *expected;
            assert_eq!(expected, ViewKey(candidate));
        }
        Ok(())
    }
}
