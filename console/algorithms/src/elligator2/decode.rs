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

impl<
    G: AffineCurve<Coordinates = (BaseField<G>, BaseField<G>)>,
    P: MontgomeryParameters<BaseField = BaseField<G>> + TwistedEdwardsParameters<BaseField = BaseField<G>>,
> Elligator2<G, P>
{
    /// Returns the decoded field element, given the encoded affine group element and sign.
    pub fn decode(group: &G, sign_high: bool) -> Result<BaseField<G>> {
        ensure!(Self::D.legendre().is_qnr(), "D on the twisted Edwards curve must be a quadratic nonresidue");
        ensure!(!group.is_zero(), "Inputs to Elligator2 must be nonzero (inverses will fail)");
        ensure!(group.is_on_curve(), "Inputs to Elligator2 must be on the twisted Edwards curve");

        // Compute the coefficients for the Weierstrass form: v^2 == u^3 + A * u^2 + B * u.
        let (montgomery_b_inverse, a, b) = match Self::MONTGOMERY_B.inverse() {
            Some(b_inverse) => (b_inverse, Self::MONTGOMERY_A * b_inverse, b_inverse.square()),
            None => bail!("Montgomery B must be invertible in order to use Elligator2"),
        };

        let x = group.to_x_coordinate();
        let y = group.to_y_coordinate();

        // Ensure that x != -A.
        ensure!(x != -a, "Elligator2 failed: x == -A");

        // Ensure that if y is 0, then x is 0.
        if y.is_zero() {
            ensure!(x.is_zero(), "Elligator2 failed: y == 0 but x != 0");
        }

        // Convert the twisted Edwards element (x, y) to the Weierstrass element (u, v)
        let (u, v) = {
            let one = BaseField::<G>::one();

            let numerator = one + y;
            let denominator = one - y;

            // Compute u = (1 + y) / (1 - y).
            let u = numerator * denominator.inverse().ok_or_else(|| anyhow!("Elligator2 failed: (1 - y) == 0"))?;

            // Compute v = (1 + y) / ((1 - y) * x).
            let v = numerator
                * (denominator * x).inverse().ok_or_else(|| anyhow!("Elligator2 failed: x * (1 - y) == 0"))?;

            // Ensure (u, v) is a valid Montgomery element on: B * v^2 == u^3 + A * u^2 + u
            let u2 = u.square();
            ensure!(
                Self::MONTGOMERY_B * v.square() == (u2 * u) + (Self::MONTGOMERY_A * u2) + u,
                "Elligator2 failed: B * v^2 != u^3 + A * u^2 + u"
            );

            let u = u * montgomery_b_inverse;
            let v = v * montgomery_b_inverse;

            // Ensure (u, v) is a valid Weierstrass element on: v^2 == u^3 + A * u^2 + B * u
            let u2 = u.square();
            ensure!(v.square() == (u2 * u) + (a * u2) + (b * u), "Elligator2 failed: v^2 != u^3 + A * u^2 + B * u");

            (u, v)
        };

        // Ensure -D * u * (u + A) is a residue.
        let du = Self::D * u;
        let u_plus_a = u + a;
        ensure!((-du * u_plus_a).legendre().is_qr(), "Elligator2 failed: -D * u * (u + A) is not a quadratic residue");

        let v_reconstructed = v.square().sqrt().ok_or_else(|| anyhow!("Elligator2 failed: cannot square root v^2"))?;
        let exists_in_sqrt_fq2 = v_reconstructed == v;

        let element = match exists_in_sqrt_fq2 {
            // Let element = sqrt(-u / ((u + A) * D)).
            true => -u * (u_plus_a * Self::D).inverse().ok_or_else(|| anyhow!("Elligator2 failed: (u+A) * D == 0"))?,
            // Let element = sqrt(-(u + A) / Du)).
            false => -u_plus_a * du.inverse().ok_or_else(|| anyhow!("Elligator2 failed: D * u == 0"))?,
        }
        .sqrt()
        .ok_or_else(|| anyhow!("Elligator2 failed: cannot compute the square root for the element"))?;

        match sign_high {
            true => Ok(cmp::max(element, -element)),
            false => Ok(cmp::min(element, -element)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use snarkvm_curves::edwards_bls12::{EdwardsAffine, EdwardsParameters};
    use snarkvm_utilities::{test_rng, UniformRand};

    pub(crate) const ITERATIONS: usize = 10000;

    #[test]
    fn test_encode_and_decode() -> Result<()> {
        let rng = &mut test_rng();

        let mut high_ctr = 0usize;
        let mut low_ctr = 0usize;

        for _ in 0..ITERATIONS {
            let expected = UniformRand::rand(rng);

            let (encoded, sign_high) =
                Elligator2::<EdwardsAffine, EdwardsParameters>::encode_without_cofactor_clear(&expected)?;
            let decoded = Elligator2::<EdwardsAffine, EdwardsParameters>::decode(&encoded, sign_high)?;
            assert_eq!(expected, decoded);

            match sign_high {
                true => high_ctr += 1,
                false => low_ctr += 1,
            }
        }

        println!("Sign high: {}, sign low: {}", high_ctr, low_ctr);
        Ok(())
    }

    #[test]
    fn test_zero_fails() {
        let encode = Elligator2::<EdwardsAffine, EdwardsParameters>::encode(&Zero::zero());
        assert!(encode.is_err());

        let decode = Elligator2::<EdwardsAffine, EdwardsParameters>::decode(&Zero::zero(), true);
        assert!(decode.is_err());
        let decode = Elligator2::<EdwardsAffine, EdwardsParameters>::decode(&Zero::zero(), false);
        assert!(decode.is_err());
    }
}
