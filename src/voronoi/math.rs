use crate::prelude::Float;

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
}
