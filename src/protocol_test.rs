pub mod protocol_test {
    use crate::computing_party::computing_party::ComputingParty;
    use std::num::Wrapping;
    use crate::multiplication::multiplication::multiplication_byte;
    use crate::utils::utils::reveal_byte_result;

    pub fn test_multi_byte(ctx: &mut ComputingParty) {
        for i in 0..2 {
            for j in 0..2 {
                for m in 0..2 {
                    for n in 0..2 {
                        let mut result = 0;
                        if ctx.party_id == 0 {
                            result = multiplication_byte(i, j, ctx);
                        } else {
                            result = multiplication_byte(m, n, ctx);
                        }
                        let result_revealed =reveal_byte_result(result,ctx);
                        assert_eq!(result_revealed, (i^m) * (j^n), "we are testing multiplication_byte with {} and {}", (i^m), (j^n));
                    }
                }
            }
        }
    }
}