use super::ppuctrl::PPUCtrl;
use super::ppumask::PPUMask;
use super::ppustatus::PPUStatus;


pub struct Registers {
    pub ppuctrl: PPUCtrl,       /* 0x2000 */
    pub ppumask: PPUMask,       /* 0x2001 */
    pub ppustatus: PPUStatus,   /* 0x2002 */
}

impl Registers {
    pub fn new() -> Self {
        let r = Registers {
            ppuctrl: PPUCtrl {flags: 0},
            ppumask: PPUMask {flags: 0},
            ppustatus: PPUStatus {flags: 0}
        };
        r
    }
}