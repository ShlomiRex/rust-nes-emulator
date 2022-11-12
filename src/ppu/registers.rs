use super::ppuctrl::PPUCtrl;
use super::ppumask::PPUMask;
use super::ppustatus::PPUStatus;

pub struct Registers {
    pub ppuctrl: PPUCtr,        /* 0x2000 */
    pub ppumask: PPUMask,       /* 0x2001 */
    pub ppustatus: PPUStatus,   /* 0x2002 */
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