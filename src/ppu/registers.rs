use super::ppuctrl::PPUCtrl;
use super::ppumask::PPUMask;
use super::ppustatus::PPUStatus;


pub struct Registers {
    pub ppuctrl: PPUCtrl,        /* 0x2000 */
    pub ppumask: PPUMask,       /* 0x2001 */
    pub ppustatus: PPUStatus,   /* 0x2002 */
}

impl Registers {
    pub fn new() -> Self {
        let r = Registers {
            ppuctrl: PPUCtrl::new(),
            ppumask: PPUMask::new(),
            ppustatus: PPUStatus::new(),
        };
        r
    }
}