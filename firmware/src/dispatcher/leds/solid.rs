use super::*;

#[derive(Copy, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Solid(u8, u8, u8);

impl Solid {
    pub fn new() -> Self {
        Solid(0, 128, 200)
    }

    pub fn update(&mut self, new: (u8, u8, u8)) {
        let (r, g, b) = new;
        self.0 = r;
        self.1 = g;
        self.2 = b;
    }

    pub fn decrement_red(&mut self) {
        self.0 = self.0.saturating_sub(1);
    }
    pub fn decrement_green(&mut self) {
        self.1 = self.1.saturating_sub(1);
    }
    pub fn decrement_blue(&mut self) {
        self.2 = self.2.saturating_sub(1);
    }

    pub fn increment_red(&mut self) {
        self.0 = self.0.saturating_add(1);
    }
    pub fn increment_green(&mut self) {
        self.1 = self.1.saturating_add(1);
    }
    pub fn increment_blue(&mut self) {
        self.2 = self.2.saturating_add(1);
    }

    pub fn red(&self) -> u8 {
        self.0
    }
    pub fn green(&self) -> u8 {
        self.1
    }
    pub fn blue(&self) -> u8 {
        self.2
    }
}

impl LEDMode for Solid {
    fn next_matrix(&mut self, _last: LEDMatrix) -> Option<LEDMatrix> {
        let rgb = (self.0, self.1, self.2).into();
        Some(LEDMatrix {
            keys: [[rgb; 7]; 3],
            thumb: [rgb; 5],
            underglow: [rgb; 6],
        })
    }
}
