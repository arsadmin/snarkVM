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

impl<N: Network> Visibility<N> for Plaintext<N> {
    /// Returns the number of field elements to encode `self`.
    fn size_in_fields(&self) -> Result<u16> {
        // Compute the number of bits.
        let num_bits = self.to_bits_le().len() + 1; // 1 extra bit for the terminus indicator.
        // Compute the ceiling division of the number of bits by the number of bits in a field element.
        let num_fields = (num_bits + N::Field::size_in_data_bits() - 1) / N::Field::size_in_data_bits();
        // Ensure the number of field elements does not exceed the maximum allowed size.
        match num_fields <= N::MAX_DATA_SIZE_IN_FIELDS as usize {
            // Return the number of field elements.
            true => Ok(num_fields as u16),
            false => bail!("Plaintext is too large to encode in field elements."),
        }
    }
}
