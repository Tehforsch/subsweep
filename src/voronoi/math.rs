use ordered_float::OrderedFloat;

use crate::prelude::Float;

// MxN matrix: This type is just here for clarity, because the
// internal storage is reversed, such that the order of indices is
// as it would be in math, i.e. Matrix<M, N> has M rows and N columns.
type Matrix<const M: usize, const N: usize> = [[Float; N]; M];

fn swap_rows<const M: usize, const N: usize>(a: &mut Matrix<M, N>, r1: usize, r2: usize) {
    for column in 0..N {
        let temp = a[r1][column];
        a[r1][column] = a[r2][column];
        a[r2][column] = temp;
    }
}

pub fn solve_system_of_equations<const M: usize>(mut a: Matrix<M, { M + 1 }>) -> [Float; M] {
    let n = M + 1;
    let mut h = 0;
    let mut k = 0;
    while h < M && k < n {
        let i_max = (h..M).max_by_key(|i| OrderedFloat(a[*i][k].abs())).unwrap();
        if a[i_max][k] == 0.0 {
            k += 1;
        } else {
            swap_rows(&mut a, h, i_max);
            for i in h + 1..M {
                let f = a[i][k] / a[h][k];
                a[i][k] = 0.0;
                for j in (k + 1)..n {
                    a[i][j] -= a[h][j] * f;
                }
            }
            h += 1;
            k += 1;
        }
    }
    backward_substitution(a)
}

pub fn backward_substitution<const M: usize>(a: Matrix<M, { M + 1 }>) -> [Float; M] {
    let mut result = [0.0; M];
    for i in (0..M).rev() {
        debug_assert!(a[i][i].abs() > 0.0);
        result[i] = a[i][M];
        for j in (i + 1)..M {
            result[i] -= a[i][j] * result[j];
        }
        result[i] /= a[i][i];
    }
    result
}

#[rustfmt::skip]
pub fn determinant3x3(
    a11: Float,
    a12: Float,
    a13: Float,
    a21: Float,
    a22: Float,
    a23: Float,
    a31: Float,
    a32: Float,
    a33: Float,
) -> Float {
      a11 * a22 * a33
    + a12 * a23 * a31
    + a13 * a21 * a32
    - a13 * a22 * a31
    - a12 * a21 * a33
    - a11 * a23 * a32
}

#[rustfmt::skip]
pub fn determinant4x4(
    a11: Float,
    a12: Float,
    a13: Float,
    a14: Float,
    a21: Float,
    a22: Float,
    a23: Float,
    a24: Float,
    a31: Float,
    a32: Float,
    a33: Float,
    a34: Float,
    a41: Float,
    a42: Float,
    a43: Float,
    a44: Float,
) -> Float {
      a11 * determinant3x3(a22,a23,a24,a32,a33,a34,a42,a43,a44)
    - a21 * determinant3x3(a12,a13,a14,a32,a33,a34,a42,a43,a44)
    + a31 * determinant3x3(a12,a13,a14,a22,a23,a24,a42,a43,a44)
    - a41 * determinant3x3(a12,a13,a14,a22,a23,a24,a32,a33,a34)
}

#[rustfmt::skip]
pub fn determinant5x5(
    a11: Float,
    a12: Float,
    a13: Float,
    a14: Float,
    a15: Float,
    a21: Float,
    a22: Float,
    a23: Float,
    a24: Float,
    a25: Float,
    a31: Float,
    a32: Float,
    a33: Float,
    a34: Float,
    a35: Float,
    a41: Float,
    a42: Float,
    a43: Float,
    a44: Float,
    a45: Float,
    a51: Float,
    a52: Float,
    a53: Float,
    a54: Float,
    a55: Float,
) -> Float {
      a11 * determinant4x4(a22, a23, a24, a25, a32, a33, a34, a35, a42, a43, a44, a45, a52, a53, a54, a55)
    - a21 * determinant4x4(a12, a13, a14, a15, a32, a33, a34, a35, a42, a43, a44, a45, a52, a53, a54, a55)
    + a31 * determinant4x4(a12, a13, a14, a15, a22, a23, a24, a25, a42, a43, a44, a45, a52, a53, a54, a55)
    - a41 * determinant4x4(a12, a13, a14, a15, a22, a23, a24, a25, a32, a33, a34, a35, a52, a53, a54, a55)
    + a51 * determinant4x4(a12, a13, a14, a15, a22, a23, a24, a25, a32, a33, a34, a35, a42, a43, a44, a45)
}

#[cfg(test)]
mod tests {
    use super::determinant3x3;
    use super::determinant4x4;
    use super::determinant5x5;
    use crate::test_utils::assert_float_is_close;

    // All of the following are completely made up matrices selected purely by the criteria of
    // not having zero determinant (I felt like that tested the code more somehow)

    #[test]
    #[rustfmt::skip]
    fn check_determinant3x3() {
        assert_float_is_close(
            determinant3x3(
                1.0, 2.0, 4.0,
                5.0, 6.0, 7.0,
                8.0, 9.0, 10.0
            ),
            -3.0,
        );
        assert_float_is_close(
            determinant3x3(
                10.0, 9.0, 8.0,
                7.0, 6.0, 5.0,
                4.0, 2.0, 1.0
            ),
            -3.0,
        );
    }

    #[test]
    #[rustfmt::skip]
    fn check_determinant4x4() {
        assert_float_is_close(
            determinant4x4(
                1.0, 1.0, 4.0, 9.0,
                16.0, 25.0, 36.0, 49.0,
                64.0, 81.0, 100.0, 121.0,
                144.0, 169.0, 196.0, 225.0,
            ),
            -512.0,
        );
    }

    #[test]
    #[rustfmt::skip]
    fn check_determinant5x5() {
        assert_float_is_close(
            determinant5x5(
                1.0, 2.0, 3.0, 4.0, 5.0,
                6.0, 7.0, 15.0, 16.0, 17.0,
                18.0, 19.0, 20.0, 21.0, 29.0,
                30.0, 31.0, 32.0, 33.0, 34.0,
                35.0, 43.0, 44.0, 45.0, 46.0,
            ),
            -9947.0,
        );
    }

    #[test]
    #[rustfmt::skip]
    fn solve_system_of_equations() {
        let res = super::solve_system_of_equations(
            [
                [2.0, 0.0, 0.0, 2.0],
                [0.0, 3.0, 0.0, 3.0],
                [0.0, 0.0, 4.0, 4.0],
            ]);
        assert_float_is_close(res[0], 1.0);
        assert_float_is_close(res[1], 1.0);
        assert_float_is_close(res[2], 1.0);
        let res = super::solve_system_of_equations(
            [
                [1.0, 0.0, 0.0, 1.0],
                [0.0, 0.0, 1.0, 1.0],
                [0.0, 1.0, 0.0, 1.0],
            ]);
        assert_float_is_close(res[0], 1.0);
        assert_float_is_close(res[1], 1.0);
        assert_float_is_close(res[2], 1.0);
        let res = super::solve_system_of_equations(
            [
                [2.0, 0.0, 0.0, 9.0],
                [0.0, 0.0, 3.0, -5.0],
                [0.0, 9.0, 0.0, -18.0],
            ]);
        assert_float_is_close(res[0], 4.5);
        assert_float_is_close(res[1], -2.0);
        assert_float_is_close(res[2], -5.0 / 3.0);
        let res = super::solve_system_of_equations(
            [
                [1.0, 2.0, 4.0, 5.0],
                [5.0, 6.0, 7.0, 10.0],
                [8.0, 9.0, 10.0, 15.0]
            ]);
        assert_float_is_close(res[0], 5.0 / 3.0);
        assert_float_is_close(res[1], -5.0 / 3.0);
        assert_float_is_close(res[2], 5.0 / 3.0);
    }
}
