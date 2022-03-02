use core::cmp;

use super::*;

pub(super) struct FadeAfterRelease {
    key: RGB8,
    bg: RGB8,
    current: LEDMatrix,
}

impl FadeAfterRelease {
    pub fn new() -> Self {
        FadeAfterRelease {
            key: (255, 0, 0).into(),
            bg: (0, 128, 200).into(),
            current: LEDMatrix::default(),
        }
    }

    pub fn key_release(&mut self, i: usize, j: usize) {
        match i {
            0..=2 => self.current.keys[i][j] = self.key.clone(),
            3 => {
                if j > 1 {
                    self.current.thumb[j - 2] = self.key.clone()
                }
            }
            _ => (),
        }
    }
}

fn converge_rgb(current: RGB8, target: RGB8) -> RGB8 {
    let RGB8 {
        r: current_r,
        g: current_g,
        b: current_b,
    } = current;
    let RGB8 {
        r: target_r,
        g: target_g,
        b: target_b,
    } = target;

    RGB8 {
        r: converge(current_r, target_r),
        g: converge(current_g, target_g),
        b: converge(current_b, target_b),
    }
}

fn converge(current: u8, target: u8) -> u8 {
    if current > target {
        cmp::max(target, current.saturating_sub((current - target) / 10))
    } else {
        cmp::min(target, current.saturating_add((target - current) / 10))
    }
}

impl LEDMode for FadeAfterRelease {
    fn next_matrix(&mut self, last: LEDMatrix) -> Option<LEDMatrix> {
        let mut next = last.clone();

        for i in 0..next.keys.len() {
            for j in 0..next.keys[i].len() {
                let RGB8 { r, g, b } = self.current.keys[i][j];
                if r > 0 || g > 0 || b > 0 {
                    next.keys[i][j] = self.current.keys[i][j];
                } else {
                    next.keys[i][j] = converge_rgb(next.keys[i][j], self.bg);
                }
            }
        }
        for i in 0..next.thumb.len() {
            let RGB8 { r, g, b } = self.current.thumb[i];
            if r > 0 || g > 0 || b > 0 {
                next.thumb[i] = self.current.thumb[i];
            } else {
                next.thumb[i] = converge_rgb(next.thumb[i], self.bg);
            }
        }

        for led in next.underglow.iter_mut() {
            *led = self.bg;
        }

        self.current = LEDMatrix::default();
        Some(next)
    }
}
