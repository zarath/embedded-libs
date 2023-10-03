#![no_std]
#![feature(array_chunks)]

#[cfg(feature = "libm")]
use core::f64::consts::PI;
use heapless::Vec;
#[cfg(feature = "libm")]
use libm::sin;

/// **PDM** modulation based on pseudo code from
/// <https://en.wikipedia.org/wiki/Pulse-density_modulation>
///
/// - **N** represents the number of pdb bits.
/// - **N_8** should be **N / 8 ** and is needed as rust currently not able
///  to divide generic constants.
///
///  # Arguments
///  
/// * curve - Takes a curve function as parameter. Curve function gets current position and the and
/// should return a f64 value between -1.0 an 1.0 - preferable starting and ending with -1.0 to
/// reduce crackle. Number of points have to be a multiple of 8!
///
/// # Example
/// ```
/// use heapless::Vec;
/// use pdm::{generate, square_idx};
/// let pdm: Vec<u8, 2> = generate::<16, 2>(square_idx::<4>);
///
/// assert_eq!(pdm[0], 0b00110011);
/// assert_eq!(pdm[1], 0b00110011);
/// ```
pub fn generate<const N: usize, const N_8: usize>(curve: fn(usize) -> f64) -> Vec<u8, N_8> {
    assert_eq!(N % 8, 0);
    let mut qe = 0.0;
    let x = (0..N)
        .map(curve)
        .map(|v| {
            qe += v;
            if qe > 0.0 {
                qe -= 1.0;
                1_u8
            } else {
                qe += 1.0;
                0_u8
            }
        })
        .collect::<Vec<u8, N>>()
        .array_chunks::<8>()
        .map(|x| x.iter().fold(0u8, |res, b| (res << 1) ^ *b))
        .collect::<Vec<u8, N_8>>();
    x
}

/// square wave function
/// returns -1.0 for the first half of each **N** points and 1.0 otherwise
#[inline]
pub const fn square_idx<const N: usize>(index: usize) -> f64 {
    if (index % N) < (N / 2) {
        -1.0
    } else {
        1.0
    }
}

/// sine wave function
#[cfg(feature = "libm")]
#[inline]
pub fn sine_idx<const N: usize>(index: usize) -> f64 {
    sin((index as f64) / (N as f64) * 2.0 * PI)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn square_wave() {
        let square: [u8; 4] = generate::<32, 4>(square_idx::<6>)
            .as_slice()
            .try_into()
            .unwrap();
        assert!(matches!(
            square,
            [0b00011100, 0b01110001, 0b11000111, 0b00011100]
        ));
    }

    #[test]
    #[cfg(feature = "libm")]
    fn sine_wave() {
        let sine: [u8; 4] = generate::<32, 4>(sine_idx::<32>)
            .as_slice()
            .try_into()
            .unwrap();
        assert!(matches!(
            sine,
            [0b01110111, 0b11111101, 0b10010000, 0b00000010]
        ));
    }
}
