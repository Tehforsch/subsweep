use crate::prelude::Float;

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
    #[rustfmt::skip]
    let det = a11 * a22 * a33
            + a12 * a23 * a31
            + a13 * a21 * a32
            - a13 * a22 * a31
            - a12 * a21 * a33
            - a11 * a23 * a32;
    det
}

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
    #[rustfmt::skip]
    let det = a11 * determinant3x3(a22,a23,a24,a32,a33,a34,a42,a43,a44)
            - a21 * determinant3x3(a12,a13,a14,a32,a33,a34,a42,a43,a44)
            + a31 * determinant3x3(a12,a13,a14,a22,a23,a24,a42,a43,a44)
            - a41 * determinant3x3(a12,a13,a14,a22,a23,a24,a32,a33,a34);
    det
}

#[cfg(test)]
mod tests {
    use super::determinant3x3;
    use super::determinant4x4;
    use crate::test_utils::assert_float_is_close;

    #[test]
    fn check_determinant3x3() {
        assert_float_is_close(
            determinant3x3(1.0, 2.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0),
            -3.0,
        );
        assert_float_is_close(
            determinant3x3(10.0, 9.0, 8.0, 7.0, 6.0, 5.0, 4.0, 2.0, 1.0),
            -3.0,
        );
    }

    #[test]
    fn check_determinant4x4() {
        assert_float_is_close(
            determinant4x4(
                1.0, 1.0, 4.0, 9.0, 16.0, 25.0, 36.0, 49.0, 64.0, 81.0, 100.0, 121.0, 144.0, 169.0,
                196.0, 225.0,
            ),
            -512.0,
        );
    }
}
