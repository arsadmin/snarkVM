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

mod decode;
mod encode;

use snarkvm_curves::{AffineCurve, MontgomeryParameters, TwistedEdwardsParameters};
use snarkvm_fields::{Field, LegendreSymbol, One, SquareRootField, Zero};

use anyhow::{anyhow, bail, ensure, Result};
use core::{cmp, marker::PhantomData, ops::Neg};

type BaseField<G> = <G as AffineCurve>::BaseField;

pub struct Elligator2<
    G: AffineCurve<Coordinates = (BaseField<G>, BaseField<G>)>,
    P: MontgomeryParameters<BaseField = BaseField<G>> + TwistedEdwardsParameters<BaseField = BaseField<G>>,
>(PhantomData<(G, P)>);
