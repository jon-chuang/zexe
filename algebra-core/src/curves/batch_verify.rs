use crate::fields::FpParameters;
use crate::{
    cfg_chunks_mut,
    curves::{batch_bucketed_add_split, BatchGroupArithmeticSlice, BATCH_SIZE},
    AffineCurve, PrimeField, Vec,
};
use num_traits::{identities::Zero, Pow};

use core::fmt;
#[cfg(feature = "parallel")]
use rand::thread_rng;
use rand::Rng;

#[cfg(feature = "parallel")]
use rayon::prelude::*;

#[derive(Debug, Clone)]
pub struct VerificationError;

impl fmt::Display for VerificationError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Verification Error. Not in subgroup")
    }
}

fn verify_points<C: AffineCurve, R: Rng>(
    points: &[C],
    num_buckets: usize,
    _new_security_param: Option<usize>, // Only pass new_security_param if possibly recursing (future PRs)
    rng: &mut R,
) -> Result<(), VerificationError> {
    let mut bucket_assign = Vec::with_capacity(points.len());
    for _ in 0..points.len() {
        bucket_assign.push(rng.gen_range(0, num_buckets));
    }
    let mut buckets = batch_bucketed_add_split(num_buckets, points, &bucket_assign[..], 12);

    // We use the batch scalar mul to check the subgroup condition if
    // there are sufficient number of buckets
    let verification_failure = if num_buckets >= BATCH_SIZE {
        cfg_chunks_mut!(buckets, BATCH_SIZE).for_each(|e| {
            let length = e.len();
            e[..].batch_scalar_mul_in_place::<<C::ScalarField as PrimeField>::BigInt>(
                &mut vec![C::ScalarField::modulus().into(); length][..],
                4,
            );
        });
        !buckets.iter().all(|&p| p == C::zero())
    } else {
        !buckets
            .iter()
            .all(|&b| b.mul(C::ScalarField::modulus()) == C::Projective::zero())
    };
    if verification_failure {
        return Err(VerificationError);
    }
    Ok(())
}

fn run_rounds<C: AffineCurve, R: Rng>(
    points: &[C],
    num_buckets: usize,
    num_rounds: usize,
    new_security_param: Option<usize>,
    rng: &mut R,
) -> Result<(), VerificationError> {
    #[cfg(feature = "parallel")]
    if num_rounds > 2 {
        use std::sync::Arc;
        let ref_points = Arc::new(points.to_vec());
        let mut threads = vec![];
        for _ in 0..num_rounds {
            let ref_points_thread = ref_points.clone();
            // We only use std when a multicore environment is available
            threads.push(std::thread::spawn(
                move || -> Result<(), VerificationError> {
                    let mut rng = &mut thread_rng();
                    verify_points(
                        &ref_points_thread[..],
                        num_buckets,
                        new_security_param,
                        &mut rng,
                    )?;
                    Ok(())
                },
            ));
        }
        for thread in threads {
            thread.join().unwrap()?;
        }
    } else {
        for _ in 0..num_rounds {
            verify_points(points, num_buckets, new_security_param, rng)?;
        }
    }

    #[cfg(not(feature = "parallel"))]
    for _ in 0..num_rounds {
        verify_points(points, num_buckets, new_security_param, rng)?;
    }

    Ok(())
}

pub fn batch_verify_in_subgroup<C: AffineCurve, R: Rng>(
    points: &[C],
    security_param: usize,
    rng: &mut R,
) -> Result<(), VerificationError> {
    let (num_buckets, num_rounds, _) = get_max_bucket(
        security_param,
        points.len(),
        <C::ScalarField as PrimeField>::Params::MODULUS_BITS as usize,
    );
    run_rounds(points, num_buckets, num_rounds, None, rng)?;
    Ok(())
}

/// We get the greatest power of 2 number of buckets such that we minimise the
/// number of rounds while satisfying the constraint that
/// n_rounds * buckets * next_check_per_elem_cost < n
fn get_max_bucket(
    security_param: usize,
    n_elems: usize,
    next_check_per_elem_cost: usize,
) -> (usize, usize, usize) {
    let mut log2_num_buckets = 1;
    let num_rounds =
        |log2_num_buckets: usize| -> usize { (security_param - 1) / log2_num_buckets + 1 };

    while num_rounds(log2_num_buckets)
        * next_check_per_elem_cost
        * (2.pow(log2_num_buckets) as usize)
        < n_elems
        && num_rounds(log2_num_buckets) > 1
    {
        log2_num_buckets += 1;
    }
    (
        2.pow(log2_num_buckets) as usize, // number of buckets
        num_rounds(log2_num_buckets),     // number of rounds
        log2_num_buckets,                 // new security param
    )
}