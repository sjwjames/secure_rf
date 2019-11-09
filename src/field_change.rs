pub mod field_change{
    use crate::computing_party::computing_party::ComputingParty;
    use std::num::Wrapping;
    use num::{BigUint, Zero, FromPrimitive};
    use crate::or_xor::or_xor::{or_xor, or_xor_bigint};

    pub fn change_binary_to_decimal_field(binary_numbers: &Vec<u8>, ctx: &mut ComputingParty) -> Vec<Wrapping<u64>> {
        let mut dummy_list = vec![Wrapping(0u64); binary_numbers.len()];
        let mut output = Vec::new();
        let mut binary_int_list = Vec::new();
        for item in binary_numbers {
            binary_int_list.push(Wrapping(*item as u64));
        }
        if ctx.asymmetric_bit == 1 {
            output = or_xor(&binary_int_list, &dummy_list, ctx, 2);
        } else {
            output = or_xor(&dummy_list, &binary_int_list, ctx, 2);
        }
        output
    }

    pub fn change_binary_to_bigint_field(binary_numbers: &Vec<u8>, ctx: &mut ComputingParty) -> Vec<BigUint> {
        let mut binary_num_bigint = Vec::new();
        for item in binary_numbers.iter() {
            binary_num_bigint.push(item.to_biguint().unwrap());
        }
        let mut dummy_list = vec![BigUint::zero(); binary_numbers.len()];

        let mut output = Vec::new();
        if ctx.asymmetric_bit == 1 {
            output = or_xor_bigint(&binary_num_bigint, &dummy_list, ctx, &BigUint::from_usize(2).unwrap());
        } else {
            output = or_xor_bigint(&dummy_list, &binary_num_bigint, ctx, &BigUint::from_usize(2).unwrap());
        }
        output
    }
}