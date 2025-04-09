//! Interpolator
//! Ported from the TypeScript implementation from Wokwi
//! https://github.com/wokwi/rp2040js/blob/main/src/interpolator.ts

#[derive(Default)]
pub struct InterpolatorConfig {
    pub shift: u32,
    pub mask_lsb: u32,
    pub mask_msb: u32,
    pub signed: bool,
    pub cross_input: bool,
    pub cross_result: bool,
    pub add_raw: bool,
    pub force_msb: u32,
    pub blend: bool,
    pub clamp: bool,
    pub over_f0: bool,
    pub over_f1: bool,
    pub over_f: bool,
}

impl InterpolatorConfig {
    fn new(value: u32) -> Self {
        Self {
            shift: (value >> 0) & 0b11111,
            mask_lsb: (value >> 5) & 0b11111,
            mask_msb: (value >> 10) & 0b11111,
            signed: ((value >> 15) & 1) != 0,
            cross_input: ((value >> 16) & 1) != 0,
            cross_result: ((value >> 17) & 1) != 0,
            add_raw: ((value >> 18) & 1) != 0,
            force_msb: (value >> 19) & 0b11,
            blend: ((value >> 21) & 1) != 0,
            clamp: ((value >> 22) & 1) != 0,
            over_f0: ((value >> 23) & 1) != 0,
            over_f1: ((value >> 24) & 1) != 0,
            over_f: ((value >> 25) & 1) != 0,
        }
    }

    fn to_u32(&self) -> u32 {
        ((self.shift & 0b11111) << 0)
            | ((self.mask_lsb & 0b11111) << 5)
            | ((self.mask_msb & 0b11111) << 10)
            | (((self.signed as u32) & 1) << 15)
            | (((self.cross_input as u32) & 1) << 16)
            | (((self.cross_result as u32) & 1) << 17)
            | (((self.add_raw as u32) & 1) << 18)
            | ((self.force_msb & 0b11) << 19)
            | (((self.blend as u32) & 1) << 21)
            | (((self.clamp as u32) & 1) << 22)
            | (((self.over_f0 as u32) & 1) << 23)
            | (((self.over_f1 as u32) & 1) << 24)
            | (((self.over_f as u32) & 1) << 25)
    }
}

#[derive(Default)]
pub struct Interpolator<const N: usize> {
    pub accum: [u32; 2],
    pub base: [u32; 3],
    pub ctrl: [u32; 2],
    pub result: [u32; 3],
    pub sm_result: [u32; 2],
}

impl<const N: usize> Interpolator<N> {
    pub fn new() -> Self {
        let mut res = Self::default();
        res.update();
        res
    }

    pub fn update(&mut self) {
        let mut ctrl0 = InterpolatorConfig::new(self.ctrl[0]);
        let mut ctrl1 = InterpolatorConfig::new(self.ctrl[1]);

        let do_clamp = ctrl0.clamp && N == 1;
        let do_blend = ctrl0.blend && N == 0;

        ctrl0.clamp = do_clamp;
        ctrl0.blend = do_blend;
        ctrl1.clamp = false;
        ctrl1.blend = false;
        ctrl1.over_f0 = false;
        ctrl1.over_f1 = false;
        ctrl1.over_f = false;

        let input0 = if ctrl0.cross_input {
            self.accum[1] as i32
        } else {
            self.accum[0] as i32
        };
        let input1 = if ctrl1.cross_input {
            self.accum[0] as i32
        } else {
            self.accum[1] as i32
        };

        let msbmask0: u32 = if ctrl0.mask_msb == 31 {
            0xffffffff
        } else {
            (1 << (ctrl0.mask_msb + 1)) - 1
        };
        let msbmask1: u32 = if ctrl1.mask_msb == 31 {
            0xffffffff
        } else {
            (1 << (ctrl1.mask_msb + 1)) - 1
        };
        let mask0 = msbmask0 & !((1 << ctrl0.mask_lsb) - 1);
        let mask1 = msbmask1 & !((1 << ctrl1.mask_lsb) - 1);

        let uresult0 = (input0 as u32 >> ctrl0.shift) & mask0;
        let uresult1 = (input1 as u32 >> ctrl1.shift) & mask1;

        let overf0 = ((input0 as u32 >> ctrl0.shift) & !msbmask0) != 0;
        let overf1 = ((input1 as u32 >> ctrl1.shift) & !msbmask1) != 0;
        let overf = overf0 || overf1;

        let sextmask0 = if (uresult0 & (1 << ctrl0.mask_msb)) != 0 {
            u32::MAX << ctrl0.mask_msb
        } else {
            0
        };
        let sextmask1 = if (uresult1 & (1 << ctrl1.mask_msb)) != 0 {
            u32::MAX << ctrl1.mask_msb
        } else {
            0
        };

        let sresult0 = uresult0 | sextmask0;
        let sresult1 = uresult1 | sextmask1;

        let result0 = if ctrl0.signed { sresult0 } else { uresult0 };
        let result1 = if ctrl1.signed { sresult1 } else { uresult1 };

        let addresult0 = self.base[0].wrapping_add(if ctrl0.add_raw {
            input0 as u32
        } else {
            result0
        });
        let addresult1 = self.base[1].wrapping_add(if ctrl1.add_raw {
            input1 as u32
        } else {
            result1
        });
        let addresult2 = self.base[2]
            .wrapping_add(result0 as u32)
            .wrapping_add(if do_blend { 0 } else { result1 as u32 });

        let uclamp0 = if result0 < self.base[0] {
            self.base[0]
        } else {
            if result0 > self.base[1] {
                self.base[1]
            } else {
                result0
            }
        };

        let sclamp0 = if (result0 as i32) < (self.base[0] as i32) {
            self.base[0]
        } else {
            if (result0 as i32) > (self.base[1] as i32) {
                self.base[1]
            } else {
                result0
            }
        };

        let clamp0 = if ctrl0.signed { sclamp0 } else { uclamp0 };

        let alpha1 = (result1 & 0xff) as f64;

        let ublend1 = self.base[0].wrapping_add(
            (alpha1 * (self.base[1].wrapping_sub(self.base[0]) as f64) / 256.0).floor() as i32
                as u32,
        );
        let sblend1 = (self.base[0] as i32).wrapping_add(
            (((alpha1 * ((self.base[1] as i32).wrapping_sub(self.base[0] as i32)) as f64) / 256.)
                .floor()) as i32,
        );
        let blend1 = if ctrl1.signed {
            sblend1 as u32
        } else {
            ublend1
        };

        self.sm_result[0] = result0 as u32;
        self.sm_result[1] = result1 as u32;

        self.result[0] = if do_blend {
            alpha1.floor() as u32
        } else {
            let res = if do_clamp { clamp0 } else { addresult0 };
            res | (ctrl0.force_msb << 28)
        };

        self.result[1] = (if do_blend { blend1 } else { addresult1 }) | (ctrl0.force_msb << 28);
        self.result[2] = addresult2;

        ctrl0.over_f0 = overf0;
        ctrl0.over_f1 = overf1;
        ctrl0.over_f = overf;
        self.ctrl[0] = ctrl0.to_u32();
        self.ctrl[1] = ctrl1.to_u32();
    }

    pub fn writeback(&mut self) {
        let ctrl0 = InterpolatorConfig::new(self.ctrl[0]);
        let ctrl1 = InterpolatorConfig::new(self.ctrl[1]);

        self.accum[0] = if ctrl0.cross_result {
            self.result[1]
        } else {
            self.result[0]
        };
        self.accum[1] = if ctrl1.cross_result {
            self.result[0]
        } else {
            self.result[1]
        };

        self.update();
    }

    pub fn set_base01(&mut self, value: u32) {
        let ctrl0 = InterpolatorConfig::new(self.ctrl[0]);
        let ctrl1 = InterpolatorConfig::new(self.ctrl[1]);

        let do_blend = ctrl0.blend && N == 0;

        let input0 = (value & 0xffff) as i32;
        let input1 = ((value >> 16) & 0xffff) as i32;

        let sextmask0 = if input0 & (1 << 15) != 0 { -1 << 15 } else { 0 };
        let sextmask1 = if input1 & (1 << 15) != 0 { -1 << 15 } else { 0 };

        let base0 = if if do_blend { ctrl1.signed } else { ctrl0.signed } {
            input0 | sextmask0
        } else {
            input0
        };
        let base1 = if ctrl1.signed {
            input1 | sextmask1
        } else {
            input1
        };

        self.base[0] = base0 as u32;
        self.base[1] = base1 as u32;

        self.update();
    }
}
