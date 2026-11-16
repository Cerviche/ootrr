use crate::tex::image::Img;

const ROOT3: f32 = 1.732_050_8;
const LUMA: [f32; 3] = [0.2126, 0.7152, 0.0722];

fn curve(v: f32) -> f32 {
    if v <= 0.003_130_8 { v * 12.92 } else { 1.055 * v.powf(1.0 / 2.4) - 0.055 }
}

fn uncurve(v: f32) -> f32 {
    if v <= 0.040_45 { v / 12.92 } else { ((v + 0.055) / 1.055).powf(2.4) }
}

pub fn lin(c: [u8; 3]) -> [f32; 3] {
    c.map(|v| uncurve(v as f32 / 255.0))
}

pub fn srgb(l: [f32; 3]) -> [u8; 3] {
    l.map(|v| (curve(v.clamp(0.0, 1.0)) * 255.0).round() as u8)
}

pub fn luma(l: [f32; 3]) -> f32 {
    LUMA[0] * l[0] + LUMA[1] * l[1] + LUMA[2] * l[2]
}

pub fn hue(c: [f32; 3]) -> (f32, f32) {
    let a: f32 = c[0] - (c[1] + c[2]) / 2.0;
    let b: f32 = ROOT3 / 2.0 * (c[1] - c[2]);
    (b.atan2(a).to_degrees().rem_euclid(360.0), a.hypot(b))
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Tint {
    pub to: [u8; 3],
    pub mid: f32,
    pub span: f32,
    pub floor: f32,
    pub full: f32,
    pub from: f32,
}

impl Tint {
    pub fn new(to: [u8; 3]) -> Tint {
        Tint { to, mid: 0.0, span: 360.0, floor: 10.0, full: 40.0, from: 0.0 }
    }

    pub fn band(mut self, mid: f32, span: f32) -> Tint {
        self.mid = mid;
        self.span = span;
        self
    }

    pub fn chroma(mut self, floor: f32, full: f32) -> Tint {
        self.floor = floor;
        self.full = full;
        self
    }

    pub fn weight(&self, c: [u8; 3]) -> f32 {
        let (h, chroma): (f32, f32) = hue([c[0] as f32, c[1] as f32, c[2] as f32]);
        let by_chroma: f32 = ((chroma - self.floor) / (self.full - self.floor)).clamp(0.0, 1.0);
        if self.span >= 180.0 {
            return by_chroma;
        }
        let far: f32 = ((h - self.mid + 540.0).rem_euclid(360.0) - 180.0).abs();
        let by_hue: f32 = ((self.span * 1.5 - far) / (self.span * 0.5)).clamp(0.0, 1.0);
        by_hue * by_chroma
    }

    pub fn fit(mut self, img: &Img) -> Option<Tint> {
        let (mut sum, mut wsum): (f64, f64) = (0.0, 0.0);
        for p in img.px.chunks_exact(4) {
            let c: [u8; 3] = [p[0], p[1], p[2]];
            let w: f64 = self.weight(c) as f64 * p[3].max(1) as f64;
            sum += w * luma(lin(c)) as f64;
            wsum += w;
        }
        if wsum <= 0.0 || sum <= 0.0 {
            return None;
        }
        self.from = (sum / wsum) as f32;
        Some(self)
    }

    pub fn rgb(&self, c: [u8; 3]) -> [u8; 3] {
        let w: f32 = self.weight(c);
        if w <= 0.0 || self.from <= 0.0 {
            return c;
        }
        let l: [f32; 3] = lin(c);
        let k: f32 = luma(l) / self.from;
        let t: [f32; 3] = lin(self.to);
        srgb([0, 1, 2].map(|i| l[i] + w * (t[i] * k - l[i])))
    }

    pub fn param(&self, v: [f32; 3], from: f32) -> [f32; 3] {
        if from <= 0.0 {
            return v;
        }
        let k: f32 = luma(v.map(|x| x.max(0.0))) / from;
        lin(self.to).map(|t| t * k)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn flat(c: [u8; 3], n: usize) -> Img {
        let mut a: Img = Img::new(n, 1);
        for x in 0..n {
            a.put(x, 0, [c[0], c[1], c[2], 255]);
        }
        a
    }

    #[test]
    fn a_fitted_tint_moves_the_mean_colour_onto_the_target() {
        for to in [[255, 0, 0], [12, 200, 90], [250, 250, 250], [5, 5, 5], [128, 64, 200]] {
            let t: Tint = Tint::new(to).chroma(3.0, 12.0).fit(&flat([64, 82, 42], 4)).unwrap();
            let got: [u8; 3] = t.rgb([64, 82, 42]);
            for i in 0..3 {
                assert!((got[i] as i32 - to[i] as i32).abs() <= 2, "{to:?} {got:?}");
            }
        }
    }

    #[test]
    fn shading_is_kept_as_brightness() {
        let mut img: Img = flat([60, 90, 40], 2);
        img.put(1, 0, [30, 45, 20, 255]);
        let t: Tint = Tint::new([200, 40, 40]).chroma(3.0, 12.0).fit(&img).unwrap();
        let (a, b): ([u8; 3], [u8; 3]) = (t.rgb([60, 90, 40]), t.rgb([30, 45, 20]));
        assert!(luma(lin(a)) > luma(lin(b)) * 3.0, "{a:?} {b:?}");
        assert!(a[0] > a[1] && b[0] > b[1]);
    }

    #[test]
    fn greys_and_colours_outside_the_band_are_left() {
        let t: Tint = Tint::new([255, 0, 255]).band(90.0, 40.0).fit(&flat([64, 82, 42], 2)).unwrap();
        assert_eq!(t.rgb([120, 120, 120]), [120, 120, 120]);
        assert_eq!(t.rgb([170, 60, 40]), [170, 60, 40]);
        assert_ne!(t.rgb([64, 82, 42]), [64, 82, 42]);
    }

    #[test]
    fn a_low_chroma_floor_reaches_dull_cloth() {
        let dull: [u8; 3] = [53, 63, 46];
        let hard: Tint = Tint::new([255, 0, 0]).band(100.0, 45.0).fit(&flat([90, 140, 40], 1)).unwrap();
        let soft: Tint = Tint::new([255, 0, 0]).band(100.0, 45.0).chroma(3.0, 12.0).fit(&flat(dull, 1)).unwrap();
        assert!(hard.weight(dull) < 0.3);
        assert_eq!(soft.weight(dull), 1.0);
        let got: [u8; 3] = soft.rgb(dull);
        assert!(got[0] > 200 && got[1] < 10 && got[2] < 10, "{got:?}");
    }

    #[test]
    fn nothing_to_fit_gives_no_tint() {
        assert!(Tint::new([1, 2, 3]).fit(&flat([128, 128, 128], 3)).is_none());
    }

    #[test]
    fn a_parameter_takes_the_target_at_its_own_brightness() {
        let t: Tint = Tint::new([255, 128, 0]);
        let first: [f32; 3] = [0.46, 0.31, 0.86];
        let got: [f32; 3] = t.param(first, luma(first));
        let want: [f32; 3] = lin([255, 128, 0]);
        for i in 0..3 {
            assert!((got[i] - want[i]).abs() < 1e-4);
        }
        let half: [f32; 3] = t.param([0.23, 0.155, 0.43], luma(first));
        assert!((half[0] - want[0] / 2.0).abs() < 1e-3);
    }
}
