use ordered_float::OrderedFloat;

use crate::prelude::Num;

// MxN matrix: This type is just here for clarity, because the
// internal storage is reversed, such that the order of indices is
// as it would be in math, i.e. Matrix<M, N> has M rows and N columns.
type Matrix<const M: usize, const N: usize, F> = [[F; N]; M];

pub fn solve_system_of_equations<const M: usize, F: Num>(mut a: Matrix<M, { M + 1 }, F>) -> [F; M] {
    let n = M + 1;
    let mut h = 0;
    let mut k = 0;
    while h < M && k < n {
        let i_max = (h..M).max_by_key(|i| OrderedFloat(a[*i][k].abs())).unwrap();
        if a[i_max][k] == F::zero() {
            k += 1;
        } else {
            swap_rows(&mut a, h, i_max);
            for i in h + 1..M {
                let f = a[i][k] / a[h][k];
                a[i][k] = F::zero();
                for j in (k + 1)..n {
                    a[i][j] = a[i][j] - a[h][j] * f;
                }
            }
            h += 1;
            k += 1;
        }
    }
    backward_substitution(a)
}

fn swap_rows<const M: usize, const N: usize, F: Clone>(
    a: &mut Matrix<M, N, F>,
    r1: usize,
    r2: usize,
) {
    for column in 0..N {
        let temp = a[r1][column].clone();
        a[r1][column] = a[r2][column].clone();
        a[r2][column] = temp;
    }
}

fn backward_substitution<const M: usize, F: Num>(a: Matrix<M, { M + 1 }, F>) -> [F; M] {
    let mut result = [F::zero(); M];
    for i in (0..M).rev() {
        result[i] = a[i][M];
        for j in (i + 1)..M {
            result[i] = result[i] - a[i][j] * result[j];
        }
        result[i] = result[i] / a[i][i];
    }
    result
}

#[rustfmt::skip]
pub fn determinant3x3<F: Num>(
    a: Matrix<3, 3, F>,
    ) -> F {
      a[0][0] * a[1][1] * a[2][2]
    + a[0][1] * a[1][2] * a[2][0]
    + a[0][2] * a[1][0] * a[2][1]
    - a[0][2] * a[1][1] * a[2][0]
    - a[0][1] * a[1][0] * a[2][2]
    - a[0][0] * a[1][2] * a[2][1]
}

#[rustfmt::skip]
pub fn determinant4x4<F: Num>(
    a: Matrix<4, 4, F>,
) -> F {
      a[0][0] * determinant3x3([[a[1][1],a[1][2],a[1][3]],[a[2][1],a[2][2],a[2][3]],[a[3][1],a[3][2],a[3][3]]])
    - a[1][0] * determinant3x3([[a[0][1],a[0][2],a[0][3]],[a[2][1],a[2][2],a[2][3]],[a[3][1],a[3][2],a[3][3]]])
    + a[2][0] * determinant3x3([[a[0][1],a[0][2],a[0][3]],[a[1][1],a[1][2],a[1][3]],[a[3][1],a[3][2],a[3][3]]])
    - a[3][0] * determinant3x3([[a[0][1],a[0][2],a[0][3]],[a[1][1],a[1][2],a[1][3]],[a[2][1],a[2][2],a[2][3]]])
}

#[rustfmt::skip]
pub fn determinant5x5<F: Num>(
    a: Matrix<5, 5, F>
) -> F {
      a[0][0] * determinant4x4([[a[1][1], a[1][2], a[1][3], a[1][4]], [a[2][1], a[2][2], a[2][3], a[2][4]], [a[3][1], a[3][2], a[3][3], a[3][4]], [a[4][1], a[4][2], a[4][3], a[4][4]]])
    - a[1][0] * determinant4x4([[a[0][1], a[0][2], a[0][3], a[0][4]], [a[2][1], a[2][2], a[2][3], a[2][4]], [a[3][1], a[3][2], a[3][3], a[3][4]], [a[4][1], a[4][2], a[4][3], a[4][4]]])
    + a[2][0] * determinant4x4([[a[0][1], a[0][2], a[0][3], a[0][4]], [a[1][1], a[1][2], a[1][3], a[1][4]], [a[3][1], a[3][2], a[3][3], a[3][4]], [a[4][1], a[4][2], a[4][3], a[4][4]]])
    - a[3][0] * determinant4x4([[a[0][1], a[0][2], a[0][3], a[0][4]], [a[1][1], a[1][2], a[1][3], a[1][4]], [a[2][1], a[2][2], a[2][3], a[2][4]], [a[4][1], a[4][2], a[4][3], a[4][4]]])
    + a[4][0] * determinant4x4([[a[0][1], a[0][2], a[0][3], a[0][4]], [a[1][1], a[1][2], a[1][3], a[1][4]], [a[2][1], a[2][2], a[2][3], a[2][4]], [a[3][1], a[3][2], a[3][3], a[3][4]]])
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
                [
                    [1.0, 2.0, 4.0],
                    [5.0, 6.0, 7.0],
                    [8.0, 9.0, 10.0]
                ]
            ),
            -3.0,
        );
        assert_float_is_close(
            determinant3x3(
                [
                    [10.0, 9.0, 8.0],
                    [7.0, 6.0, 5.0],
                    [4.0, 2.0, 1.0]
                ]
            ),
            -3.0,
        );
    }

    #[test]
    #[rustfmt::skip]
    fn check_determinant4x4() {
        assert_float_is_close(
            determinant4x4(
                [
                    [1.0, 1.0, 4.0, 9.0],
                    [16.0, 25.0, 36.0, 49.0],
                    [64.0, 81.0, 100.0, 121.0],
                    [144.0, 169.0, 196.0, 225.0],
                ]
            ),
            -512.0,
        );
    }

    #[test]
    #[rustfmt::skip]
    fn check_determinant5x5() {
        assert_float_is_close(
            determinant5x5(
                [
                    [1.0, 2.0, 3.0, 4.0, 5.0],
                    [6.0, 7.0, 15.0, 16.0, 17.0],
                    [18.0, 19.0, 20.0, 21.0, 29.0],
                    [30.0, 31.0, 32.0, 33.0, 34.0],
                    [35.0, 43.0, 44.0, 45.0, 46.0],
                ]
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
