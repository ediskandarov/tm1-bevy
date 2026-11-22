//! Original C arithmetic with explicit PS1 scalar layouts.

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct Matrix {
    pub rotation: [[i16; 3]; 3],
    pub translation: [i32; 3],
}
#[repr(C)]
#[derive(Clone, Copy)]
struct ShortVector {
    xyz: [i16; 3],
    pad: i16,
}

unsafe extern "C" {
    fn mathMulVec(matrix: *const Matrix, input: *const [i32; 3], out: *mut [i32; 3]);
    fn mathMulTransVec(matrix: *const Matrix, input: *const [i32; 3], out: *mut [i32; 3]);
    fn mathMulVecLong(matrix: *const Matrix, input: *const [i32; 3], out: *mut [i32; 3]);
    fn mathMulTransVecShort(matrix: *const Matrix, input: *const ShortVector, out: *mut [i32; 3]);
    fn mathNormalizeVec(input: *const [i32; 3], out: *mut [i32; 3]);
}

pub fn multiply(matrix: &Matrix, input: [i32; 3], transpose: bool) -> [i32; 3] {
    let mut out = [0; 3];
    unsafe {
        if transpose {
            mathMulTransVec(matrix, &input, &mut out);
        } else {
            mathMulVec(matrix, &input, &mut out);
        }
    }
    out
}

pub fn multiply_long(matrix: &Matrix, input: [i32; 3]) -> [i32; 3] {
    let mut out = [0; 3];
    unsafe {
        mathMulVecLong(matrix, &input, &mut out);
    }
    out
}

pub fn multiply_short_transpose(matrix: &Matrix, input: [i16; 3]) -> [i32; 3] {
    let mut out = [0; 3];
    let input = ShortVector { xyz: input, pad: 0 };
    unsafe {
        mathMulTransVecShort(matrix, &input, &mut out);
    }
    out
}

/// The SDK's negative-square-root path reads before its lookup table. Until
/// that behavior is captured, reject vectors whose squared norm overflows.
pub fn normalize(input: [i32; 3]) -> Option<[i32; 3]> {
    let squared: u64 = input
        .iter()
        .map(|&v| (i64::from(v) * i64::from(v)) as u64)
        .sum();
    if squared > i32::MAX as u64 {
        return None;
    }
    let mut out = [0; 3];
    unsafe {
        mathNormalizeVec(&input, &mut out);
    }
    Some(out)
}

/// Reconstruct the PsyQ table mathematically; no executable data is embedded.
pub fn square_root(value: u32) -> Option<u32> {
    if value > i32::MAX as u32 {
        return None;
    }
    if value == 0 {
        return Some(0);
    }
    let even = value.leading_zeros() & !1;
    let exponent = (31 - even) / 2;
    let normalized = if even >= 24 {
        value << (even - 24)
    } else {
        value >> (24 - even)
    };
    let table_value = (normalized << 18).isqrt();
    Some((table_value << exponent) >> 12)
}

#[unsafe(no_mangle)]
extern "C" fn SquareRoot0(value: i32) -> i32 {
    // Only reached through normalize(), which checks the complete input domain.
    square_root(value as u32).expect("unvalidated negative SquareRoot0 input") as i32
}
