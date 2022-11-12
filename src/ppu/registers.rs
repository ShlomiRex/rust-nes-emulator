use super::ppuctrl::PPUCtrl;
use super::ppumask::PPUMask;
use super::ppustatus::PPUStatus;

pub struct Registers {
    pub ppuctrl: PPUCtr,
    pub ppumask: PPUMask,
    pub ppustatus: PPUStatus,
}

impl Registers {
    pub fn new() -> Self {
        let mut r = Registers {
            ppuctrl: PPUCtr(0),
            ppumask: PPUMask(0),
            ppustatus: PPUStatus(0),
        };
        r
    }
}