use super::*;

pub(super) struct Wheel(u8);

impl Wheel {
    pub fn new() -> Self {
        Wheel(0)
    }

    fn wheel(mut wheel_pos: u8) -> (u8, u8, u8) {
        wheel_pos = 255 - wheel_pos;
        if wheel_pos < 85 {
            return (255 - wheel_pos * 3, 0, wheel_pos * 3);
        }
        if wheel_pos < 170 {
            wheel_pos -= 85;
            return (0, wheel_pos * 3, 255 - wheel_pos * 3);
        }
        wheel_pos -= 170;
        (wheel_pos * 3, 255 - wheel_pos * 3, 0)
    }
}

impl LEDMode for Wheel {
    fn next_matrix(&mut self, _last: LEDMatrix) -> Option<LEDMatrix> {
        let rgb = Wheel::wheel(self.0).into();
        self.0 = if self.0 < 255 { self.0 + 1 } else { 0 };
        Some(LEDMatrix {
            keys: [[rgb; 7]; 3],
            thumb: [rgb; 5],
            underglow: [rgb; 6],
        })
    }
}
