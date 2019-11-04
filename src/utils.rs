pub mod utils{
    use num::bigint::{BigUint, ToBigUint, ToBigInt, RandBigInt};
    use num::integer::*;
    use std::ops::Sub;

    pub fn big_uint_subtract(x:&BigUint,y:&BigUint,big_int_prime:&BigUint)->BigUint{
        let result = x.to_bigint().unwrap().sub(y.to_bigint().unwrap()).mod_floor(&(big_int_prime.to_bigint().unwrap())).to_biguint().unwrap();
        result
    }

    pub fn big_uint_clone(x:&BigUint)->BigUint{
        let mut result = BigUint::from_bytes_le(&(x.to_bytes_le().clone()));
        result
    }

    pub fn big_uint_vec_clone(list:&Vec<BigUint>)->Vec<BigUint>{
        let mut result = Vec::new();
        for item in list.iter(){
            result.push(big_uint_clone(item));
        }
        result
    }
}