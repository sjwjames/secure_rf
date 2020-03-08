pub mod discretize {
    use crate::bit_decomposition::bit_decomposition::batch_log_decomp;
    use std::num::Wrapping;
    use crate::computing_party::computing_party::ComputingParty;
    use crate::multiplication::multiplication::batch_multiply;
    use crate::utils::utils::{truncate_local, u64_to_byte_array, byte_array_to_u64};
    use crate::constants::constants::BUF_SIZE;
    use std::io::{Write, Read};
    use crate::constants::constants;

    const BATCH_SIZE_REVEAL: usize = constants::REVEAL_BATCH_SIZE;

    pub fn minmax(x_list: &Vec<Wrapping<u64>>,
                  ctx: &mut ComputingParty) -> (Wrapping<u64>, Wrapping<u64>) {
        let asymmetric_bit = Wrapping(ctx.asymmetric_bit as u64);

        let mut n = x_list.len();
        let mut pairs = n / 2;

        let mut l_operands: Vec<Wrapping<u64>> = Vec::new();
        let mut r_operands: Vec<Wrapping<u64>> = Vec::new();

        for i in 0..pairs {
            l_operands.push(x_list[2 * i]);
            r_operands.push(x_list[2 * i + 1]);
        }

        // println!("l_ops: {:5?}", reveal(&l_operands, ctx, ctx.decimal_precision, true));
        // println!("r_ops: {:5?}", reveal(&r_operands, ctx, ctx.decimal_precision, true));

        let l_geq_r = batch_compare(&l_operands, &r_operands, ctx);
        let l_geq_r: Vec<u64> = reveal(&l_operands, ctx, ctx.decimal_precision, true, ctx.debug_output)
            .iter()
            .zip(&reveal(&r_operands, ctx, ctx.decimal_precision, true, ctx.debug_output))
            .map(|(&l, &r)| if (l >= r) && ctx.asymmetric_bit == 1 { 1u64 } else { 0u64 })
            .collect();

        let mut l_geq_r = xor_share_to_additive(&l_geq_r, ctx, 1);
        let mut l_lt_r: Vec<Wrapping<u64>> = l_geq_r.iter().map(|x| -x + asymmetric_bit).collect();

        // println!("l_geq_r: {:?}", reveal_wrapping(&l_geq_r, ctx));
        // println!("l_lt_r : {:?}", reveal_wrapping(&l_lt_r, ctx));

        let mut values = l_operands.clone();
        values.append(&mut l_operands.clone());
        values.append(&mut r_operands.clone());
        values.append(&mut r_operands.clone());

        let mut assignments = l_geq_r.clone();
        assignments.append(&mut l_lt_r.clone());
        assignments.append(&mut l_geq_r.clone());
        assignments.append(&mut l_lt_r.clone());

        let min_max_pairs = batch_multiply(&values, &assignments, ctx);

        let mut mins: Vec<Wrapping<u64>> = Vec::new();
        let mut maxs: Vec<Wrapping<u64>> = Vec::new();


        for i in 0..pairs {
            maxs.push(min_max_pairs[i] + min_max_pairs[i + 3 * pairs]);
            mins.push(min_max_pairs[i + pairs] + min_max_pairs[i + 2 * pairs]);
        }

        if n % 2 == 1 {
            maxs.push(x_list[n - 1]);
            mins.push(x_list[n - 1]);
        }

        // println!("n={} -- min of pairs: {:5?}", n, reveal(&mins, ctx, ctx.decimal_precision, true ));
        // println!("n={} -- max of pairs: {:5?}", n, reveal(&maxs, ctx, ctx.decimal_precision, true ));

        n = (n / 2) + (n % 2);
        pairs = (n / 2);


        while n > 1 {


            // println!("_____________________________________________");

            // println!("n: {}, pairs: {}", n, pairs);

            let mut l_operands: Vec<Wrapping<u64>> = Vec::new();
            let mut r_operands: Vec<Wrapping<u64>> = Vec::new();

            for i in 0..pairs {
                l_operands.push(mins[2 * i]);
                r_operands.push(mins[2 * i + 1]);
            }

            for i in 0..pairs {
                l_operands.push(maxs[2 * i]);
                r_operands.push(maxs[2 * i + 1]);
            }

            // println!("l_ops: {:5?}", reveal(&l_operands, ctx, ctx.decimal_precision, true));
            // println!("r_ops: {:5?}", reveal(&r_operands, ctx, ctx.decimal_precision, true));

            let l_geq_r = batch_compare(&l_operands, &r_operands, ctx);
            let l_geq_r: Vec<u64> = reveal(&l_operands, ctx, ctx.decimal_precision, true, true)
                .iter()
                .zip(&reveal(&r_operands, ctx, ctx.decimal_precision, true, true))
                .map(|(&l, &r)| if (l >= r) && ctx.asymmetric_bit == 1 { 1u64 } else { 0u64 })
                .collect();

            let mut l_geq_r = xor_share_to_additive(&l_geq_r, ctx, 1);
            let mut l_lt_r: Vec<Wrapping<u64>> = l_geq_r.iter().map(|x| -x + asymmetric_bit).collect();

            // println!("l_geq_r: {:?}", reveal_wrapping(&l_geq_r, ctx));
            // println!("l_lt_r : {:?}", reveal_wrapping(&l_lt_r, ctx));

            let mut values = r_operands[..(pairs)].to_vec();
            values.append(&mut l_operands[(pairs)..].to_vec());
            values.append(&mut l_operands[..(pairs)].to_vec());
            values.append(&mut r_operands[(pairs)..].to_vec());

            let mut assignments = l_geq_r.clone();
            assignments.append(&mut l_lt_r.clone());

            let min_max_pairs = batch_multiply(&values, &assignments, ctx);

            // println!("min_max_pairs: {:5?}", reveal(&min_max_pairs, ctx, ctx.decimal_precision, true));

            let mut new_mins: Vec<Wrapping<u64>> = Vec::new();
            let mut new_maxs: Vec<Wrapping<u64>> = Vec::new();

            for i in 0..pairs {
                new_mins.push(min_max_pairs[i] + min_max_pairs[i + 2 * pairs]);
                new_maxs.push(min_max_pairs[i + pairs] + min_max_pairs[i + 3 * pairs]);
            }

            if n % 2 == 1 {
                new_mins.push(mins[n - 1]);
                new_maxs.push(maxs[n - 1]);
            }

            mins = new_mins;
            maxs = new_maxs;

            // println!("min of pairs: {:5?}", reveal(&mins, ctx, ctx.decimal_precision, true ));
            // println!("max of pairs: {:5?}", reveal(&maxs, ctx, ctx.decimal_precision, true ));


            n = (n / 2) + (n % 2);
            pairs = (n / 2);
        }

        (mins[0], maxs[0])
    }

    pub fn batch_compare(x_list: &Vec<Wrapping<u64>>,
                         y_list: &Vec<Wrapping<u64>>,
                         ctx: &mut ComputingParty) -> Vec<u64> {
        let diff_dc = batch_log_decomp(
            &x_list.iter().zip(y_list.iter()).map(|(&x, &y)| (x - y)).collect(),
            (ctx.integer_precision + ctx.decimal_precision + 1) as usize, 5, ctx);

        diff_dc.iter().map(|z| (z >> 63) ^ ctx.asymmetric_bit as u64).collect()
    }

    pub fn discretize(x_list: &Vec<Wrapping<u64>>,
                      buckets: usize,
                      ctx: &mut ComputingParty) -> Vec<Wrapping<u64>> {
        let n = x_list.len();
        let (min, max) = minmax(&x_list, ctx);

        // println!("min: {}, max: {}",
        // 	reveal(&vec![min], ctx, ctx.decimal_precision, true)[0],
        // 	reveal(&vec![max], ctx, ctx.decimal_precision, true)[0]);

        let range = max - min;
        let mut height_markers: Vec<Wrapping<u64>> = Vec::new();
        for i in 1..buckets {
            let height_ratio = (i as f64) / (buckets as f64);
            let height_ratio_ring = Wrapping((height_ratio * 2f64.powf(ctx.decimal_precision as f64)) as u64);
            //	println!("height_ratio: {}, height_ratio_ring: {}", height_ratio,height_ratio_ring);
            height_markers.push(
                min + truncate_local(
                    height_ratio_ring * range,
                    ctx.decimal_precision,
                    ctx.asymmetric_bit,
                ));
        }

        //println!("height_markers: {:5?}", reveal(&height_markers, ctx, ctx.decimal_precision, true));

        let mut l_operands: Vec<Wrapping<u64>> = Vec::new();
        for i in 0..n {
            for j in 0..(buckets - 1) {
                l_operands.push(x_list[i]);
            }
        }

        let mut r_operands: Vec<Wrapping<u64>> = Vec::new();
        for i in 0..n {
            for j in 0..(buckets - 1) {
                r_operands.push(height_markers[j]);
            }
        }

        let l_geq_r = batch_compare(&l_operands, &r_operands, ctx);
        let l_geq_r: Vec<u64> = reveal(&l_operands, ctx, ctx.decimal_precision, true, true).iter()
            .zip(&reveal(&r_operands, ctx, ctx.decimal_precision, true, true))
            .map(|(&l, &r)| if (l >= r) && ctx.asymmetric_bit == 1 { 1u64 } else { 0u64 })
            .collect();
        let l_geq_r: Vec<Wrapping<u64>> = xor_share_to_additive(&l_geq_r, ctx, 1)
            .iter()
            .map(|x| Wrapping(x.0 << ctx.decimal_precision as u64))
            .collect();

        let mut x_discrete = vec![Wrapping(0u64); n];
        for i in 0..n {
            for j in 0..(buckets - 1) {
                x_discrete[i] += l_geq_r[(buckets - 1) * i + j];
            }
        }

        x_discrete
    }

    pub fn xor_share_to_additive(x_list_u64: &Vec<u64>, ctx: &mut ComputingParty, size: usize)
                                 -> Vec<Wrapping<u64>> {
        let len = x_list_u64.len();
        let mut x_additive_list = vec![Wrapping(0u64); len];

        for i in 0..len {
            let mut bin_list = vec![0u64; size];

            for j in 0..size {
                bin_list[j] = (x_list_u64[i] >> j as u64) & 1u64;
            }

            let bin_list_ring = binary_vector_to_ring(&bin_list, ctx);

            let mut bitselect = Wrapping(1u64);
            for j in 0..size {
                x_additive_list[i] += bitselect * bin_list_ring[j];
                bitselect <<= 1;
            }
        }

        x_additive_list
    }

    pub fn binary_vector_to_ring(x_list_u64: &Vec<u64>,
                                 ctx: &mut ComputingParty) -> Vec<Wrapping<u64>> {
        let len = (*x_list_u64).len();

        let mut x_list: Vec<Wrapping<u64>> = vec![Wrapping(0); len];
        let dummy: Vec<Wrapping<u64>> = vec![Wrapping(0); len];
        let mut x_list_ring: Vec<Wrapping<u64>> = vec![Wrapping(0); len];
        let mut product_list: Vec<Wrapping<u64>>;

        for i in 0..len {
            x_list[i] = Wrapping(x_list_u64[i]);
        }

        if (*ctx).asymmetric_bit == 1 {
            product_list = batch_multiply(&x_list, &dummy, ctx);
        } else {
            product_list = batch_multiply(&dummy, &x_list, ctx);
        }

        for i in 0..len {
            x_list_ring[i] = x_list[i] - Wrapping(2) * product_list[i];
        }

        x_list_ring
    }


    pub fn reveal(x_list: &Vec<Wrapping<u64>>,
                  ctx: &mut ComputingParty,
                  decimal_precision: u32,
                  to_real: bool,
                  show_recvd: bool,
    ) -> Vec<f64> {
        let len = (*x_list).len();
        let mut x_combined: Vec<Wrapping<u64>> = vec![Wrapping(0); len];
        let mut x_revealed: Vec<f64> = vec![0.0; len];
        let mut remainder = len.clone();
        let mut index = 0;

        while remainder > BATCH_SIZE_REVEAL {

            //if ctx.debug_output { println!("[index={}][remainder={}]", index, remainder); }

            let mut x_sublist = [Wrapping(0); BATCH_SIZE_REVEAL];
            x_sublist.clone_from_slice(&(x_list[BATCH_SIZE_REVEAL * index..BATCH_SIZE_REVEAL * (index + 1)]));

            let x_combined_sublist = reveal_submodule(x_sublist, BATCH_SIZE_REVEAL, ctx, show_recvd);

            x_combined[BATCH_SIZE_REVEAL * index..BATCH_SIZE_REVEAL * (index + 1)].clone_from_slice(&x_combined_sublist);

            remainder -= BATCH_SIZE_REVEAL;
            index += 1;
        }

        //if ctx.debug_output {println!("[index={}][remainder={}]", index, remainder);}

        let mut x_sublist = [Wrapping(0); BATCH_SIZE_REVEAL];
        x_sublist[0..remainder].clone_from_slice(&(x_list[BATCH_SIZE_REVEAL * index..]));

        let x_combined_sublist = reveal_submodule(x_sublist, BATCH_SIZE_REVEAL, ctx, show_recvd);

        x_combined[BATCH_SIZE_REVEAL * index..].clone_from_slice(&(x_combined_sublist[..remainder]));

        if to_real {
            for i in 0..len {
                let x = x_combined[i];

                if (x.0 >> 63) == 0 {// TODO replace 63 with named constant RINGSIZE-1

                    x_revealed[i] = (x.0 as f64) / (2u64.pow(decimal_precision) as f64);
                } else {
                    x_revealed[i] = -1.0 * ((-x).0 as f64) / (2u64.pow(decimal_precision) as f64);
                }
            }
        } else {
            for i in 0..len {
                x_revealed[i] = x_combined[i].0 as f64;
            }
        }


        x_revealed
    }


    pub fn reveal_submodule(x_list: [Wrapping<u64>; BATCH_SIZE_REVEAL],
                            tx_len: usize,
                            ctx: &mut ComputingParty,
                            show_recvd: bool) -> [Wrapping<u64>; BATCH_SIZE_REVEAL]
    {
        let mut x_revealed = [Wrapping(0); BATCH_SIZE_REVEAL];

        let mut o_stream = ctx.o_stream.try_clone()
            .expect("failed cloning tcp o_stream");

        let mut in_stream = ctx.in_stream.try_clone()
            .expect("failed cloning tcp o_stream");

        let mut tx_buf = [0u8; BUF_SIZE];
        let mut rx_buf = [0u8; BUF_SIZE];

        for i in 0..tx_len {
            let x = x_list[i].0;
            tx_buf[8 * i..8 * (i + 1)].clone_from_slice(&u64_to_byte_array(x));
        }

        if ctx.asymmetric_bit == 1 {
            let mut bytes_written = 0;
            while bytes_written < BUF_SIZE {
                let current_bytes = o_stream.write(&tx_buf[bytes_written..]).unwrap();
                bytes_written += current_bytes;
            }

            let mut bytes_read = 0;
            while bytes_read < BUF_SIZE {
                let current_bytes = match in_stream.read(&mut rx_buf[bytes_read..]) {
                    Ok(size) => size,
                    Err(_) => panic!("couldn't read"),
                };
                bytes_read += current_bytes;
            }
        } else {
            let mut bytes_read = 0;
            while bytes_read < BUF_SIZE {
                let current_bytes = match in_stream.read(&mut rx_buf[bytes_read..]) {
                    Ok(size) => size,
                    Err(_) => panic!("couldn't read"),
                };
                bytes_read += current_bytes;
            }

            let mut bytes_written = 0;
            while bytes_written < BUF_SIZE {
                let current_bytes = o_stream.write(&tx_buf[bytes_written..]).unwrap();
                bytes_written += current_bytes;
            }
        }

        if show_recvd {
            println!("Received: {:?}", rx_buf[..8].to_vec());
        }

        for i in 0..tx_len {
            let mut x_other_buf = [0 as u8; 8];
            x_other_buf.clone_from_slice(&rx_buf[8 * i..8 * (i + 1)]);
            let x_other = Wrapping(byte_array_to_u64(x_other_buf));

            x_revealed[i] = x_list[i] + x_other;
        }

        x_revealed
    }
}