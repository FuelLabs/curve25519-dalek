#![allow(non_snake_case)]

use crate::backend::serial::curve_models::ProjectiveNielsPoint;
use crate::edwards::EdwardsPoint;
use crate::scalar::Scalar;
use crate::traits::Identity;
use crate::window::LookupTable;

#[cfg(not(all(target_os = "zkvm", target_vendor = "succinct")))]
/// Perform constant-time, variable-base scalar multiplication.
#[rustfmt::skip] // keep alignment of explanatory comments
pub(crate) fn mul(point: &EdwardsPoint, scalar: &Scalar) -> EdwardsPoint {
    // Construct a lookup table of [P,2P,3P,4P,5P,6P,7P,8P]
    let lookup_table = LookupTable::<ProjectiveNielsPoint>::from(point);
    // Setting s = scalar, compute
    //
    //    s = s_0 + s_1*16^1 + ... + s_63*16^63,
    //
    // with `-8 ≤ s_i < 8` for `0 ≤ i < 63` and `-8 ≤ s_63 ≤ 8`.
    // This decomposition requires s < 2^255, which is guaranteed by Scalar invariant #1.
    let scalar_digits = scalar.as_radix_16();
    // Compute s*P as
    //
    //    s*P = P*(s_0 +   s_1*16^1 +   s_2*16^2 + ... +   s_63*16^63)
    //    s*P =  P*s_0 + P*s_1*16^1 + P*s_2*16^2 + ... + P*s_63*16^63
    //    s*P = P*s_0 + 16*(P*s_1 + 16*(P*s_2 + 16*( ... + P*s_63)...))
    //
    // We sum right-to-left.

    // Unwrap first loop iteration to save computing 16*identity
    let mut tmp2;
    let mut tmp3 = EdwardsPoint::identity();
    let mut tmp1 = &tmp3 + &lookup_table.select(scalar_digits[63]);
    // Now tmp1 = s_63*P in P1xP1 coords
    for i in (0..63).rev() {
        tmp2 = tmp1.as_projective(); // tmp2 =    (prev) in P2 coords
        tmp1 = tmp2.double();        // tmp1 =  2*(prev) in P1xP1 coords
        tmp2 = tmp1.as_projective(); // tmp2 =  2*(prev) in P2 coords
        tmp1 = tmp2.double();        // tmp1 =  4*(prev) in P1xP1 coords
        tmp2 = tmp1.as_projective(); // tmp2 =  4*(prev) in P2 coords
        tmp1 = tmp2.double();        // tmp1 =  8*(prev) in P1xP1 coords
        tmp2 = tmp1.as_projective(); // tmp2 =  8*(prev) in P2 coords
        tmp1 = tmp2.double();        // tmp1 = 16*(prev) in P1xP1 coords
        tmp3 = tmp1.as_extended();   // tmp3 = 16*(prev) in P3 coords
        tmp1 = &tmp3 + &lookup_table.select(scalar_digits[i]);
        // Now tmp1 = s_i*P + 16*(prev) in P1xP1 coords
    }
    tmp1.as_extended()
}

#[cfg(all(target_os = "zkvm", target_vendor = "succinct"))]
use sp1_lib::{ed25519::Ed25519AffinePoint, utils::AffinePoint};
#[cfg(all(target_os = "zkvm", target_vendor = "succinct"))]
/// Perform constant-time, variable-base scalar multiplication.
///
/// Accelerated with SP1's EdAdd syscall.
#[allow(non_snake_case)]
pub(crate) fn mul(point: &EdwardsPoint, scalar: &Scalar) -> EdwardsPoint {
    let mut ed_point: Ed25519AffinePoint = (*point).into();
    ed_point.mul_assign(&sp1_lib::utils::bytes_to_words_le(scalar.as_bytes())).expect("Scalar multiplication failed");
    ed_point.into()
}
