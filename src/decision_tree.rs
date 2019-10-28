pub mod decision_tree {
    use crate::computing_party::computing_party::ComputingParty;
    use num_bigint::{BigInt, BigUint};
    use std::io::Bytes;

    pub struct DecisionTreeTraining {
        pub max_depth:usize,
        pub alpha:BigInt,
        pub epsilon:f32,
        pub cutoff_transaction_set_size:usize,
        pub attr_count:usize,
        pub dataset_size:usize,
        pub attr_value_count:usize,
        pub class_value_count:usize,
        pub attr_values:Vec<Vec<Vev<u64>>>,
        pub class_values:Vec<Vec<Vev<u64>>>,
        pub attr_values_big_integer:Vec<Vec<Vev<BigUint>>>,
        pub class_values_big_integer:Vec<Vec<Vev<BigUint>>>,
        pub level_counter:usize,
        pub subset_transaction_bit_vector:Vec<u64>,
        pub attribute_bit_vector:Vec<u64>,

        pub prime:BigUint,
        pub dataset_size_prime:u64,
        pub dataset_size_bit_length:u64,
        pub bit_length:u64,
        pub big_int_ti_index:u64,
    }

    pub fn train(mut ctx: ComputingParty) {

    }

    pub fn init(ctx: ComputingParty)->DecisionTreeTraining{

    }

    fn find_common_class_index(){

    }
}