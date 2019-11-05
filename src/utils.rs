pub mod utils {
    use num::bigint::{BigUint, ToBigUint, ToBigInt, RandBigInt};
    use num::integer::*;
    use std::ops::Sub;
    use std::num::Wrapping;
    use crate::computing_party::computing_party::ComputingParty;

    pub fn big_uint_subtract(x: &BigUint, y: &BigUint, big_int_prime: &BigUint) -> BigUint {
        let result = x.to_bigint().unwrap().sub(y.to_bigint().unwrap()).mod_floor(&(big_int_prime.to_bigint().unwrap())).to_biguint().unwrap();
        result
    }

    pub fn big_uint_clone(x: &BigUint) -> BigUint {
        let mut result = BigUint::from_bytes_le(&(x.to_bytes_le().clone()));
        result
    }

    pub fn big_uint_vec_clone(list: &Vec<BigUint>) -> Vec<BigUint> {
        let mut result = Vec::new();
        for item in list.iter() {
            result.push(big_uint_clone(item));
        }
        result
    }

    pub fn truncate_local(x: Wrapping<u64>,
                          decimal_precision: u32,
                          asymmetric_bit: u8) -> Wrapping<u64> {
        if asymmetric_bit == 0 {
            return -Wrapping((-x).0 >> decimal_precision);
        }

        Wrapping(x.0 >> decimal_precision)
    }

    pub enum ShareType {
        AdditiveShare,
        AdditiveBigIntShare,
        BinaryShare,
        EqualityShare,
    }

    pub fn increment_current_share_index(ctx: &mut ComputingParty, share_type: ShareType) {

        match share_type {
            ShareType::AdditiveShare => {
                let mut count = ctx.dt_shares.current_additive_index.lock().unwrap();
                *count+=1;
            }
            ShareType::AdditiveBigIntShare => {
                let mut count = ctx.dt_shares.current_additive_bigint_index.lock().unwrap();
                *count+=1;
            }
            ShareType::BinaryShare => {
                let mut count = ctx.dt_shares.current_binary_index.lock().unwrap();
                *count+=1;
            }
            ShareType::EqualityShare => {
                let mut count = ctx.dt_shares.current_equality_index.lock().unwrap();
                *count+=1;
            }
        }
    }
}