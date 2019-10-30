pub mod decision_tree {
    use crate::computing_party::computing_party::ComputingParty;
    use num::bigint::{BigInt, BigUint};
    use std::io::Bytes;
    use serde::{Serialize, Serializer};

    pub struct DecisionTreeData {
        pub attr_value_count: usize,
        pub class_value_count: usize,
        pub attribute_count: usize,
        pub instance_count: usize,
        pub attr_values: Vec<Vec<Vec<u64>>>,
        pub class_values: Vec<Vec<u64>>,
        pub attr_values_big_integer: Vec<Vec<Vec<BigUint>>>,
        pub class_values_big_integer: Vec<Vec<BigUint>>,
    }

    pub struct DecisionTreeTraining {
        pub max_depth: usize,
        pub alpha: BigInt,
        pub epsilon: f64,
        pub cutoff_transaction_set_size: usize,
        pub subset_transaction_bit_vector: Vec<u8>,
        pub attribute_bit_vector: Vec<u8>,
        pub prime: BigUint,
        pub dataset_size_prime: u64,
        pub dataset_size_bit_length: u64,
        pub bit_length: u64,
        pub big_int_ti_index: u64,
    }

    pub struct DecisionTreeShares{
        pub additive_triples:Vec<(u64,u64,u64)>,
        pub additive_bigint_triples:Vec<(BigUint,BigUint,BigUint)>,
        pub binary_triples:Vec<(u8)>,
        pub equality_shares:Vec<(BigUint)>
    }






    impl Clone for DecisionTreeData {
        fn clone(&self) -> Self {
            DecisionTreeData {
                attr_value_count: self.attr_value_count,
                class_value_count: self.class_value_count,
                attribute_count: self.attribute_count,
                instance_count: self.instance_count,
                attr_values: self.attr_values.clone(),
                class_values: self.class_values.clone(),
                attr_values_big_integer: self.attr_values_big_integer.clone(),
                class_values_big_integer: self.class_values_big_integer.clone(),
            }
        }
    }

    impl Clone for DecisionTreeTraining {
        fn clone(&self) -> Self {
            DecisionTreeTraining {
                max_depth: self.max_depth,
                alpha: self.alpha.clone(),
                epsilon: self.epsilon.clone(),
                cutoff_transaction_set_size: self.cutoff_transaction_set_size,
                subset_transaction_bit_vector: self.subset_transaction_bit_vector.clone(),
                attribute_bit_vector: self.attribute_bit_vector.clone(),
                prime: self.prime.clone(),
                dataset_size_prime: self.dataset_size_prime,
                dataset_size_bit_length: self.dataset_size_bit_length,
                bit_length: self.bit_length,
                big_int_ti_index: self.big_int_ti_index,
            }
        }
    }


    pub fn train(mut ctx: ComputingParty) {}


    fn find_common_class_index() {}
}