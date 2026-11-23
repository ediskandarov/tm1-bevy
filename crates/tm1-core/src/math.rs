//! Original C arithmetic with explicit PS1 scalar layouts.

mod tables;
use tables::{ROOT, ROTATION, TABLES};

#[derive(Clone, Copy)]
pub struct TrigTables;

impl TrigTables {
    pub(crate) fn as_ptr(&self) -> *const i16 {
        TABLES.as_ptr()
    }
    pub fn sine(&self, angle: i32) -> i32 {
        let phase = (angle & 4095) as usize;
        let i = phase & 1023;
        let v = i32::from(TABLES[if phase & 1024 == 0 { i } else { 1024 - i }]);
        if phase & 2048 == 0 { v } else { -v }
    }
    pub fn rotation_yxz(&self, angles: [i16; 3]) -> Matrix {
        unsafe extern "C" {
            fn tm1_rotation_yxz(table: *const i16, angles: *const i16, matrix: *mut Matrix);
        }
        let table = &TABLES[ROTATION..];
        let mut matrix = Matrix::default();
        unsafe { tm1_rotation_yxz(table.as_ptr(), angles.as_ptr(), &mut matrix) };
        matrix
    }

    pub fn sin_cos(&self, angle: i64) -> [i16; 2] {
        let i = ROTATION + 2 * (angle.unsigned_abs() as usize & 4095);
        [
            if angle < 0 { -TABLES[i] } else { TABLES[i] },
            TABLES[i + 1],
        ]
    }
}

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

/// Run the original vector routine, including its signed 32-bit sum of squares.
pub fn normalize(input: [i32; 3]) -> Option<[i32; 3]> {
    let mut out = [0; 3];
    unsafe {
        mathNormalizeVec(&input, &mut out);
    }
    Some(out)
}

/// The SDK counts leading sign bits. Negative inputs occur when a game's
/// wrapping sum of squares crosses the signed 32-bit boundary.
fn sdk_square_root(value: i32) -> u32 {
    let magnitude = if value < 0 { !value } else { value };
    if magnitude == 0 {
        return 0;
    }
    let even = magnitude.leading_zeros() & !1;
    let normalized = if even >= 24 {
        value << (even - 24)
    } else {
        value >> (24 - even)
    };
    let coefficient = TABLES[(ROOT as i32 + normalized - 64) as usize] as u32;
    (coefficient << ((31 - even) / 2)) >> 12
}

/// Interpret all 32 bits as the input register of the JP SquareRoot0 routine.
pub fn square_root(value: u32) -> Option<u32> {
    Some(sdk_square_root(value as i32))
}

#[unsafe(no_mangle)]
extern "C" fn SquareRoot0(value: i32) -> i32 {
    sdk_square_root(value) as i32
}
