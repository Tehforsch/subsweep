use std::cmp::Ordering;

use array_init::from_iter;
use num::FromPrimitive;
use num::Signed;
use num::Zero;

use super::precision_types::FloatError;
use super::precision_types::PrecisionError;
use super::precision_types::PrecisionFloat;
use super::precision_types::DETERMINANT_3X3_EPSILON;
use super::precision_types::DETERMINANT_4X4_EPSILON;
use super::precision_types::DETERMINANT_5X5_EPSILON;
use super::traits::Num;

// MxN matrix: This type is just here for clarity, because the
// internal storage is reversed, such that the order of indices is
// as it would be in math, i.e. Matrix<M, N> has M rows and N columns.
type Matrix<const M: usize, const N: usize, F> = [[F; N]; M];

pub fn solve_system_of_equations<const M: usize, F: Num>(mut a: Matrix<M, { M + 1 }, F>) -> [F; M] {
    let n = M + 1;
    let mut h = 0;
    let mut k = 0;
    while h < M && k < n {
        let i_max = (h..M)
            .max_by(|i, j| a[*i][k].abs().partial_cmp(&a[*j][k].abs()).unwrap())
            .unwrap();
        if a[i_max][k] == F::zero() {
            k += 1;
        } else {
            swap_rows(&mut a, h, i_max);
            for i in h + 1..M {
                let f = a[i][k].clone() / a[h][k].clone();
                a[i][k] = F::zero();
                for j in (k + 1)..n {
                    a[i][j] = a[i][j].clone() - a[h][j].clone() * f.clone();
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
    let mut result = array_init::array_init(|_| F::zero());
    for i in (0..M).rev() {
        result[i] = a[i][M].clone();
        for j in (i + 1)..M {
            result[i] = result[i].clone() - a[i][j].clone() * result[j].clone();
        }
        result[i] = result[i].clone() / a[i][i].clone();
    }
    result
}

fn lift_matrix<const D: usize>(m: Matrix<D, D, f64>) -> Matrix<D, D, PrecisionFloat> {
    let iter = m.into_iter().map(|row| {
        let x: [PrecisionFloat; D] = from_iter(
            row.into_iter()
                .map(|x| PrecisionFloat::from_f64(x).unwrap()),
        )
        .unwrap();
        x
    });
    let arr: Matrix<D, D, PrecisionFloat> = from_iter(iter).unwrap();
    arr
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Sign {
    Positive,
    Negative,
    Zero,
}

impl std::ops::Mul<Sign> for Sign {
    type Output = Self;

    fn mul(self, rhs: Sign) -> Self::Output {
        match self {
            Sign::Positive => match rhs {
                Sign::Positive => Sign::Positive,
                Sign::Negative => Sign::Negative,
                Sign::Zero => Sign::Zero,
            },
            Sign::Negative => match rhs {
                Sign::Positive => Sign::Negative,
                Sign::Negative => Sign::Positive,
                Sign::Zero => Sign::Zero,
            },
            Sign::Zero => Sign::Zero,
        }
    }
}

impl Sign {
    pub fn of<T: Zero + PartialOrd>(val: T) -> Self {
        match val.partial_cmp(&T::zero()).unwrap() {
            Ordering::Less => Sign::Negative,
            Ordering::Equal => Sign::Zero,
            Ordering::Greater => Sign::Positive,
        }
    }

    pub fn try_from_val<T: Zero + PartialOrd + Signed + FloatError>(
        val: &T,
        epsilon: f64,
    ) -> Result<Self, PrecisionError> {
        PrecisionError::check(val, epsilon)?;
        Ok(match val.partial_cmp(&T::zero()).unwrap() {
            Ordering::Less => Sign::Negative,
            Ordering::Equal => Sign::Zero,
            Ordering::Greater => Sign::Positive,
        })
    }

    pub fn is_positive(self) -> bool {
        matches!(self, Sign::Positive)
    }

    pub fn is_negative(self) -> bool {
        matches!(self, Sign::Negative)
    }

    pub fn panic_if_zero<U: std::fmt::Display>(&self, arg: impl Fn() -> U) -> &Self {
        if let Self::Zero = self {
            panic!("{}", arg())
        }
        self
    }
}

fn compare_result_against_entries<const D: usize>(
    val: f64,
    m: &Matrix<D, D, f64>,
    epsilon: f64,
) -> Result<f64, PrecisionError> {
    for row in m.iter() {
        for entry in row.iter() {
            PrecisionError::check(&(val / entry), epsilon)?;
        }
    }
    Ok(val)
}

// I would use a HRTB here, but these arent stable for non-lifetime bindings
// as far as I can tell.
fn determine_sign_with_arbitrary_precision_if_necessary<const D: usize>(
    m: Matrix<D, D, f64>,
    f: fn(Matrix<D, D, f64>) -> f64,
    f_arbitrary_precision: fn(Matrix<D, D, PrecisionFloat>) -> PrecisionFloat,
    epsilon: f64,
) -> Sign {
    let val = f(m);
    match compare_result_against_entries(val, &m, epsilon) {
        Ok(val) => Sign::of(val),
        Err(_) => {
            let m = lift_matrix(m);
            Sign::of(f_arbitrary_precision(m))
        }
    }
}

pub fn determinant3x3_sign(a: Matrix<3, 3, f64>) -> Sign {
    determine_sign_with_arbitrary_precision_if_necessary(
        a,
        determinant3x3::<f64>,
        determinant3x3::<PrecisionFloat>,
        DETERMINANT_3X3_EPSILON,
    )
}

pub fn determinant4x4_sign(a: Matrix<4, 4, f64>) -> Sign {
    determine_sign_with_arbitrary_precision_if_necessary(
        a,
        determinant4x4::<f64>,
        determinant4x4::<PrecisionFloat>,
        DETERMINANT_4X4_EPSILON,
    )
}

pub fn determinant5x5_sign(a: Matrix<5, 5, f64>) -> Sign {
    determine_sign_with_arbitrary_precision_if_necessary(
        a,
        determinant5x5::<f64>,
        determinant5x5::<PrecisionFloat>,
        DETERMINANT_5X5_EPSILON,
    )
}

#[rustfmt::skip]
pub fn determinant3x3<F: Num>(
    a: Matrix<3, 3, F>,
    ) -> F {
    let [[a00, a01, a02], [a10, a11, a12], [a20, a21, a22]] = a;
      a00.clone() * a11.clone() * a22.clone()
    + a01.clone() * a12.clone() * a20.clone()
    + a02.clone() * a10.clone() * a21.clone()
    - a02 * a11 * a20
    - a01 * a10 * a22
    - a00 * a12 * a21
}

#[rustfmt::skip]
pub fn determinant4x4<F: Num>(
    a: Matrix<4, 4, F>,
) -> F {
      a[0][0].clone() * determinant3x3([[a[1][1].clone(),a[1][2].clone(),a[1][3].clone()],[a[2][1].clone(),a[2][2].clone(),a[2][3].clone()],[a[3][1].clone(),a[3][2].clone(),a[3][3].clone()]])
    - a[1][0].clone() * determinant3x3([[a[0][1].clone(),a[0][2].clone(),a[0][3].clone()],[a[2][1].clone(),a[2][2].clone(),a[2][3].clone()],[a[3][1].clone(),a[3][2].clone(),a[3][3].clone()]])
    + a[2][0].clone() * determinant3x3([[a[0][1].clone(),a[0][2].clone(),a[0][3].clone()],[a[1][1].clone(),a[1][2].clone(),a[1][3].clone()],[a[3][1].clone(),a[3][2].clone(),a[3][3].clone()]])
    - a[3][0].clone() * determinant3x3([[a[0][1].clone(),a[0][2].clone(),a[0][3].clone()],[a[1][1].clone(),a[1][2].clone(),a[1][3].clone()],[a[2][1].clone(),a[2][2].clone(),a[2][3].clone()]])
}

#[rustfmt::skip]
pub fn determinant5x5<F: Num>(
    a: Matrix<5, 5, F>
) -> F {
      a[0][0].clone() * determinant4x4([[a[1][1].clone(), a[1][2].clone(), a[1][3].clone(), a[1][4].clone()], [a[2][1].clone(), a[2][2].clone(), a[2][3].clone(), a[2][4].clone()], [a[3][1].clone(), a[3][2].clone(), a[3][3].clone(), a[3][4].clone()], [a[4][1].clone(), a[4][2].clone(), a[4][3].clone(), a[4][4].clone()]])
    - a[1][0].clone() * determinant4x4([[a[0][1].clone(), a[0][2].clone(), a[0][3].clone(), a[0][4].clone()], [a[2][1].clone(), a[2][2].clone(), a[2][3].clone(), a[2][4].clone()], [a[3][1].clone(), a[3][2].clone(), a[3][3].clone(), a[3][4].clone()], [a[4][1].clone(), a[4][2].clone(), a[4][3].clone(), a[4][4].clone()]])
    + a[2][0].clone() * determinant4x4([[a[0][1].clone(), a[0][2].clone(), a[0][3].clone(), a[0][4].clone()], [a[1][1].clone(), a[1][2].clone(), a[1][3].clone(), a[1][4].clone()], [a[3][1].clone(), a[3][2].clone(), a[3][3].clone(), a[3][4].clone()], [a[4][1].clone(), a[4][2].clone(), a[4][3].clone(), a[4][4].clone()]])
    - a[3][0].clone() * determinant4x4([[a[0][1].clone(), a[0][2].clone(), a[0][3].clone(), a[0][4].clone()], [a[1][1].clone(), a[1][2].clone(), a[1][3].clone(), a[1][4].clone()], [a[2][1].clone(), a[2][2].clone(), a[2][3].clone(), a[2][4].clone()], [a[4][1].clone(), a[4][2].clone(), a[4][3].clone(), a[4][4].clone()]])
    + a[4][0].clone() * determinant4x4([[a[0][1].clone(), a[0][2].clone(), a[0][3].clone(), a[0][4].clone()], [a[1][1].clone(), a[1][2].clone(), a[1][3].clone(), a[1][4].clone()], [a[2][1].clone(), a[2][2].clone(), a[2][3].clone(), a[2][4].clone()], [a[3][1].clone(), a[3][2].clone(), a[3][3].clone(), a[3][4].clone()]])
}

#[cfg(test)]
mod tests {
    use std::fs::File;
    use std::path::Path;

    use crate::test_utils::assert_float_is_close;
    use crate::voronoi::math::utils::determinant3x3_sign;
    use crate::voronoi::math::utils::determinant5x5_sign;
    use crate::voronoi::math::utils::lift_matrix;
    use crate::voronoi::math::utils::Matrix;
    use crate::voronoi::math::utils::Sign;

    // All of the following are completely made up matrices selected purely by the criteria of
    // not having zero determinant (I felt like that tested the code more somehow)

    #[test]
    #[rustfmt::skip]
    fn determinant3x3() {
        assert_float_is_close(
            super::determinant3x3(
                [
                    [1.0, 2.0, 4.0],
                    [5.0, 6.0, 7.0],
                    [8.0, 9.0, 10.0]
                ]
            ),
            -3.0,
        );
        assert_float_is_close(
            super::determinant3x3(
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
    fn determinant4x4() {
        assert_float_is_close(
            super::determinant4x4(
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
    fn determinant5x5() {
        assert_float_is_close(
            super::determinant5x5(
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

    #[test]
    fn matrix_with_zero_determinant_precision() {
        let matrix: Matrix<3, 3, f64> = [
            [
                7.041529113171147e-9,
                7.041529113171147e-9,
                7.041529113171147e-9,
            ],
            [
                -0.013275610231885723,
                -4.576711463239629e-246,
                7.041529113212176e-9,
            ],
            [
                7.041529113171147e-9,
                7.041529113171147e-9,
                7.041529113171147e-9,
            ],
        ];
        assert_eq!(
            determinant3x3_sign(matrix),
            Sign::of(super::determinant3x3(lift_matrix(matrix)))
        );
    }

    #[test]
    fn critical_matrices_precision_5x5() {
        let matrices_file = Path::new(file!())
            .parent()
            .unwrap()
            .join("../../../tests/data/critical_matrices_5x5");
        let matrices_file = File::open(matrices_file).unwrap();
        let matrices = serde_yaml::from_reader::<_, Vec<Vec<Vec<f64>>>>(matrices_file).unwrap();
        let matrices = matrices.into_iter().map(|m| vec_as_matrix::<5, 5>(m));
        for matrix in matrices {
            assert_eq!(
                determinant5x5_sign(matrix),
                Sign::of(super::determinant5x5(lift_matrix(matrix)))
            );
        }
    }

    fn vec_as_matrix<const M: usize, const N: usize>(v: Vec<Vec<f64>>) -> Matrix<M, N, f64> {
        let mut m: Matrix<M, N, f64> = [[0.0; N]; M];
        assert_eq!(m.len(), M);
        for i in 0..M {
            assert_eq!(m[i].len(), N);
            for j in 0..N {
                m[i][j] = v[i][j];
            }
        }
        m
    }
}
