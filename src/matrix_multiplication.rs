pub mod matrix_multiplication{
    use crate::computing_party::computing_party::ComputingParty;
    use std::num::Wrapping;
    use std::thread;
    use crate::protocol::protocol::reveal_wrapping;
    use crate::utils::utils::truncate_local;

    pub fn batch_vectorized_matmul(a_matrix   : &Vec<Vec<Wrapping<u64>>>,
                                   b_matrices : &Vec<Vec<Vec<Wrapping<u64>>>>,
                                   e_matrix   : &Vec<Vec<Wrapping<u64>>>,
                                   v_matrices : &mut Vec<Vec<Vec<Wrapping<u64>>>>,
                                   z_matrices : &mut Vec<Vec<Vec<Wrapping<u64>>>>,
                                   ctx        : &mut ComputingParty ) -> Vec<Vec<Vec<Wrapping<u64>>>> {

        /* bounds checks...can be commented out */
        /* number of instances */
        assert_eq!(b_matrices.len(), v_matrices.len());
        assert_eq!(b_matrices.len(), z_matrices.len());

        /* m */
        assert_eq!(a_matrix.len(), z_matrices[0].len());
        assert_eq!(a_matrix.len(), e_matrix.len());

        /* n */
        assert_eq!(a_matrix[0].len(), b_matrices[0].len());
        assert_eq!(a_matrix[0].len(), v_matrices[0].len());
        assert_eq!(a_matrix[0].len(), e_matrix[0].len());

        /* r */
        assert_eq!(b_matrices[0][0].len(), v_matrices[0][0].len());
        assert_eq!(b_matrices[0][0].len(), z_matrices[0][0].len());
        /* end bounds checks */

        let instances = b_matrices.len();
        let m = a_matrix.len();
        let n = a_matrix[0].len();
        let r = b_matrices[0][0].len();

        /* get all b, v entrywise in two vectors and open shares of b - v */
        let mut b: Vec<Wrapping<u64>> = Vec::new();
        let mut v: Vec<Wrapping<u64>> = Vec::new();

        for instance in 0..instances {

            let v_matrix = v_matrices.pop().unwrap();

            for i in 0..n {
                for j in 0..r {

                    b.push( b_matrices[instance][i][j] );
                    v.push( v_matrix[i][j] );
                }
            }
        }

        let f = reveal_wrapping(&b.iter().zip(v.iter()).map(|(&bi, &vi)| bi - vi).collect(), ctx);

        // compute F, B tranposes.
        let mut f_matrices_transpose: Vec<Vec<Vec<Wrapping<u64>>>> = vec![
            vec![ vec![ Wrapping(0u64); n ] ; r ] ; instances ];

        let mut b_matrices_transpose: Vec<Vec<Vec<Wrapping<u64>>>> = vec![
            vec![ vec![ Wrapping(0u64); n ] ; r ] ; instances ];

        for instance in 0..instances {
            for i in 0..n {
                for j in 0..r {
                    f_matrices_transpose[instance][j][i] = f[ instance*(n*r) + i*(r) + j ];
                    b_matrices_transpose[instance][j][i] = b_matrices[instance][i][j];
                }
            }
        }

        let mut instance_handles: Vec<Vec<Vec<thread::JoinHandle<Wrapping<u64>>>>> = Vec::new();

        for instance in 0..instances {
            let z_matrix = z_matrices.pop().unwrap();
            let mut row_handles: Vec<Vec<thread::JoinHandle<Wrapping<u64>>>> = Vec::new();

            for i in 0..m {
                let mut dp_handles: Vec<thread::JoinHandle<Wrapping<u64>>> = Vec::new();

                for j in 0..r {
                    let a_row  = a_matrix[i].clone();
                    let e_row  = e_matrix[i].clone();
                    let b_col = b_matrices_transpose[instance][j].clone();
                    let f_col = f_matrices_transpose[instance][j].clone();
                    let z = z_matrix[i][j];

                    let decimal_precision = ctx.decimal_precision;
                    let attribute_count   = ctx.dt_data.attribute_count;
                    let asymmetric_bit    = ctx.asymmetric_bit;

                    let dp_handle = thread::spawn(move || {

                        // get dot product of i-th row of a_matrix w/ j-th col of b_matrix

                        let mut e_f = Wrapping(0u64);
                        let mut a_f = Wrapping(0u64);
                        let mut e_b = Wrapping(0u64);

                        for k in 0..n {

                            e_f += e_row[k] * f_col[k];
                            a_f += a_row[k] * f_col[k];
                            e_b += e_row[k] * b_col[k];
                        }

                        truncate_local(
                            - Wrapping(asymmetric_bit as u64) * e_f + a_f + e_b + z,
                            decimal_precision,
                            asymmetric_bit
                        )

                    });

                    dp_handles.push(dp_handle);
                }
                row_handles.push(dp_handles);
            }
            instance_handles.push(row_handles);
        }

        let mut output_matrices: Vec<Vec<Vec<Wrapping<u64>>>> = Vec::new();

        for instance in instance_handles {
            let mut output_matrix: Vec<Vec<Wrapping<u64>>> = Vec::new();

            for row in instance {
                let output_row: Vec<Wrapping<u64>> = row.into_iter().map(|x| x.join().unwrap()).collect();
                output_matrix.push(output_row);
            }
            output_matrices.push(output_matrix);
        }

        output_matrices
    }

}