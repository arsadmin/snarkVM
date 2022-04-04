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

use crate::{
    fft::{
        domain::{FFTPrecomputation, IFFTPrecomputation},
        DensePolynomial,
        EvaluationDomain,
        Evaluations as EvaluationsOnDomain,
    },
    snark::marlin::{
        ahp::{indexer::Circuit, verifier},
        AHPError,
        MarlinMode,
    },
};
use snarkvm_fields::PrimeField;
use snarkvm_r1cs::SynthesisError;

/// State for the AHP prover.
pub struct State<'a, F: PrimeField, MM: MarlinMode> {
    pub(super) index: &'a Circuit<F, MM>,

    /// A domain that is sized for the public input.
    pub(super) input_domain: EvaluationDomain<F>,

    /// A domain that is sized for the number of constraints.
    pub(super) constraint_domain: EvaluationDomain<F>,

    /// A domain that is sized for the number of non-zero elements in A.
    pub(super) non_zero_a_domain: EvaluationDomain<F>,
    /// A domain that is sized for the number of non-zero elements in B.
    pub(super) non_zero_b_domain: EvaluationDomain<F>,
    /// A domain that is sized for the number of non-zero elements in C.
    pub(super) non_zero_c_domain: EvaluationDomain<F>,
    /// Query bound b
    pub(super) zk_bound: Option<usize>,

    pub(super) padded_public_variables: Vec<Vec<F>>,
    pub(super) private_variables: Vec<Vec<F>>,
    /// Az.
    pub(super) z_a: Option<Vec<Vec<F>>>,
    /// Bz.
    pub(super) z_b: Option<Vec<Vec<F>>>,

    pub(super) x_poly: Vec<DensePolynomial<F>>,
    pub(super) first_round_oracles: Option<super::FirstOracles<'a, F>>,
    pub(super) mz_poly_randomizer: Option<Vec<F>>,

    /// The challenges sent by the verifier in the first round
    pub(super) verifier_first_message: Option<verifier::FirstMessage<F>>,

    /// Polynomials involved in the holographic sumcheck.
    pub(super) lhs_polynomials: Option<[DensePolynomial<F>; 3]>,
    /// Polynomials involved in the holographic sumcheck.
    pub(super) sums: Option<[F; 3]>,
}

impl<'a, F: PrimeField, MM: MarlinMode> State<'a, F, MM> {
    pub fn initialize(
        padded_public_input: Vec<Vec<F>>,
        private_variables: Vec<Vec<F>>,
        zk_bound: impl Into<Option<usize>>,
        index: &'a Circuit<F, MM>,
    ) -> Result<Self, AHPError> {
        let index_info = &index.index_info;
        let constraint_domain =
            EvaluationDomain::new(index_info.num_constraints).ok_or(SynthesisError::PolynomialDegreeTooLarge)?;

        let non_zero_a_domain =
            EvaluationDomain::new(index_info.num_non_zero_a).ok_or(SynthesisError::PolynomialDegreeTooLarge)?;
        let non_zero_b_domain =
            EvaluationDomain::new(index_info.num_non_zero_b).ok_or(SynthesisError::PolynomialDegreeTooLarge)?;
        let non_zero_c_domain =
            EvaluationDomain::new(index_info.num_non_zero_c).ok_or(SynthesisError::PolynomialDegreeTooLarge)?;

        let input_domain =
            EvaluationDomain::new(padded_public_input[0].len()).ok_or(SynthesisError::PolynomialDegreeTooLarge)?;

        let x_poly = padded_public_input
            .iter()
            .map(|padded_public_input| {
                EvaluationsOnDomain::from_vec_and_domain(padded_public_input.clone(), input_domain).interpolate()
            })
            .collect();

        Ok(Self {
            padded_public_variables: padded_public_input,
            x_poly,
            private_variables,
            zk_bound: zk_bound.into(),
            index,
            input_domain,
            constraint_domain,
            non_zero_a_domain,
            non_zero_b_domain,
            non_zero_c_domain,
            z_a: None,
            z_b: None,
            first_round_oracles: None,
            mz_poly_randomizer: None,
            verifier_first_message: None,
            lhs_polynomials: None,
            sums: None,
        })
    }

    /// Get the public input.
    pub fn public_input(&self, i: usize) -> Vec<F> {
        super::ConstraintSystem::unformat_public_input(&self.padded_public_variables[i])
    }

    /// Get the padded public input.
    pub fn padded_public_input(&self, i: usize) -> &[F] {
        &self.padded_public_variables[i]
    }

    pub fn fft_precomputation(&self) -> &FFTPrecomputation<F> {
        &self.index.fft_precomputation
    }

    pub fn ifft_precomputation(&self) -> &IFFTPrecomputation<F> {
        &self.index.ifft_precomputation
    }
}
