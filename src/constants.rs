pub mod constants {
    //author Davis, email:daviscrailsback@gmail.com
    use std::num::Wrapping;

    // never change
    pub const MULT_ELEMS: usize = 2;
    pub const SIZEOF_U64: usize = 8;

    // can tweak batch size to optimize batch multiplication for different machines
    pub const BATCH_SIZE: usize = 4096;
    // how many mults can be done in one tx
    pub const REVEAL_BATCH_SIZE: usize = 2 * BATCH_SIZE;
    // how many reveals in one TX

    pub const BUF_SIZE: usize = BATCH_SIZE * MULT_ELEMS * SIZEOF_U64;
    pub const U64S_PER_TX: usize = 2 * BATCH_SIZE;
    pub const U8S_PER_TX: usize = 8 * U64S_PER_TX;
    pub const TI_BATCH_SIZE: usize = U64S_PER_TX / 3; // how many trplets in one tx

    pub const BINARY_PRIME: usize = 2;

    /* Simulated correlated randomness for debugging without the trusted initializer */
    pub const CR_0: (Wrapping<u64>, Wrapping<u64>, Wrapping<u64>) =
        (
            Wrapping(6833008916512791354),
            Wrapping(14547997512572730844),
            Wrapping(4912126131370984821)
        );

    pub const CR_1: (Wrapping<u64>, Wrapping<u64>, Wrapping<u64>) =
        (
            Wrapping(2741451256696586090),
            Wrapping(8847773937267267195),
            Wrapping(7432801596505970759)
        );

    pub const CR_BIN_0: (u64, u64, u64) =
        (
            16985030433743349914,
            16800953460621756407,
            13341120138288862608
        );

    pub const CR_BIN_1: (u64, u64, u64) =
        (
            9026346012512690793,
            5254509534812310812,
            4172330766887821171
        );
}