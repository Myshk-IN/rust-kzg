use crate::consts::{expand_root_of_unity, SCALE2_ROOT_OF_UNITY, SCALE_FACTOR};
use blst::{blst_fr_add, blst_fr_cneg, blst_fr_from_uint64, blst_fr_inverse, blst_fr_mul, blst_uint64_from_fr, blst_fr_sqr, blst_fr_sub, blst_fr_eucl_inverse, blst_fr};
use kzg::{G1, G2, TFFTSettings, TFr, TPoly};

pub struct Fr(blst::blst_fr);

impl TFr for Fr {
    fn default() -> Self {
        Fr(blst_fr::default())
    }

    fn zero() -> Self {
        Fr::from_u64(0)
    }

    fn one() -> Self {
        Fr::from_u64(1)
    }

    fn rand() -> Fr {
        let val: [u64; 4] = rand::random();
        let ret: Fr = Fr::default();
        unsafe {
            blst_fr_from_uint64(&ret as *const Fr as *mut blst_fr, val.as_ptr());
        }

        ret
    }

    fn from_u64_arr(u: &[u64; 4]) -> Self {
        let ret = Fr::default();
        unsafe {
            blst_fr_from_uint64(&ret as *const Fr as *mut blst_fr, u.as_ptr());
        }

        ret
    }

    fn from_u64(val: u64) -> Self {
        Fr::from_u64_arr(&[val, 0, 0, 0])
    }

    fn is_one(&self) -> bool {
        let mut val: [u64; 4] = [0; 4];
        unsafe {
            blst_uint64_from_fr(val.as_mut_ptr(), self as *const Fr as *const blst_fr);
        }
        return val[0] == 1 && val[1] == 0 && val[2] == 0 && val[3] == 0;
    }

    fn is_zero(&self) -> bool {
        let mut val: [u64; 4] = [0; 4];
        unsafe {
            blst_uint64_from_fr(val.as_mut_ptr(), self as *const Fr as *const blst_fr);
        }
        return val[0] == 0 && val[1] == 0 && val[2] == 0 && val[3] == 0;
    }

    fn sqr(&self) -> Self {
        let ret = Fr::default();
        unsafe {
            blst_fr_sqr(&ret as *const Fr as *mut blst_fr, self as *const Fr as *const blst_fr);
        }

        ret
    }

    // fn pow(&self, n: usize) -> Self {
    //     //fr_t tmp = *a;
    //     let mut tmp: Fr = self.clone();
    //
    //     //*out = fr_one;
    //     let mut out = Fr::one();
    //     let mut n2 = n;
    //
    //     unsafe {
    //         loop {
    //             if n2 & 1 == 1 {
    //                 blst_fr_mul(&out as *const Fr as *mut blst_fr, &out as *const Fr as *mut blst_fr, &tmp as *const Fr as *mut blst_fr);
    //             }
    //             n2 = n2 >> 1;
    //             if n == 0 {
    //                 break;
    //             }
    //             blst_fr_sqr(&tmp as *const Fr as *mut blst_fr, &tmp as *const Fr as *mut blst_fr);
    //         }
    //     }
    //
    //     out
    // }

    fn mul(&self, b: &Fr) -> Self {
        let ret = Fr::default();
        unsafe {
            blst_fr_mul(&ret as *const Fr as *mut blst_fr, self as *const Fr as *const blst_fr, b as *const Fr as *const blst_fr);
        }

        ret
    }

    fn add(&self, b: &Fr) -> Self {
        let ret = Fr::default();
        unsafe {
            blst_fr_add(&ret as *const Fr as *mut blst_fr, self as *const Fr as *const blst_fr, b as *const Fr as *const blst_fr);
        }

        ret
    }

    fn sub(&self, b: &Fr) -> Self {
        let ret = Fr::default();
        unsafe {
            blst_fr_sub(&ret as *const Fr as *mut blst_fr, self as *const Fr as *const blst_fr, b as *const Fr as *mut blst_fr);
        }

        ret
    }

    fn eucl_inverse(&self) -> Self {
        let ret = Fr::default();
        unsafe {
            blst_fr_eucl_inverse(&ret as *const Fr as *mut blst_fr, self as *const Fr as *const blst_fr);
        }

        return ret;
    }

    fn negate(&self) -> Self {
        let ret = Fr::default();
        unsafe {
            blst_fr_cneg(&ret as *const Fr as *mut blst_fr, self as *const Fr as *const blst_fr, true);
        }

        ret
    }

    fn inverse(&self) -> Self {
        let ret = Fr::default();
        unsafe {
            blst_fr_inverse(&ret as *const Fr as *mut blst_fr, self as *const Fr as *const blst_fr);
        }

        ret
    }

    fn equals(&self, b: &Fr) -> bool {
        let mut val_a: [u64; 4] = [0; 4];
        let mut val_b: [u64; 4] = [0; 4];

        unsafe {
            blst_uint64_from_fr(val_a.as_mut_ptr(), self as *const Fr as *const blst_fr);
            blst_uint64_from_fr(val_b.as_mut_ptr(), b as *const Fr as *mut blst_fr);
        }

        return val_a[0] == val_b[0]
            && val_a[1] == val_b[1]
            && val_a[2] == val_b[2]
            && val_a[3] == val_b[3];
    }

    fn destroy(&self) {}
}

pub struct Poly {
    pub coeffs: Vec<Fr>,
}

impl TPoly<Fr> for Poly {
    fn new(size: usize) -> Self {
        Poly { coeffs: vec![Fr::default(); size] }
    }

    fn get_coeff_at(&self, i: usize) -> Fr where Fr: Sized {
        self.coeffs[i]
    }

    fn set_coeff_at(&mut self, i: usize, x: &Fr) {
        self.coeffs[i] = x.clone()
    }

    fn get_coeffs(&self) -> &[Fr] {
        &self.coeffs
    }

    fn len(&self) -> usize {
        self.coeffs.len()
    }

    fn eval(&self, x: &Fr) -> Fr {
        if self.coeffs.len() == 0 {
            return Fr::zero();
        } else if x.is_zero() {
            return self.coeffs[0].clone();
        }

        let mut ret = self.coeffs[self.coeffs.len() - 1].clone();
        let mut i = self.coeffs.len() - 2;
        loop {
            let temp = ret.mul(&x);
            ret = temp.add(&self.coeffs[i]);

            if i == 0 {
                break;
            }
            i -= 1;
        }

        return ret;
    }

    fn scale(&mut self) {
        let scale_factor = Fr::from_u64(SCALE_FACTOR);
        let inv_factor = scale_factor.inverse();

        let mut factor_power = Fr::one();
        for i in 0..self.coeffs.len() {
            factor_power = factor_power.mul(&inv_factor);
            self.coeffs[i] = self.coeffs[i].mul(&factor_power);
        }
    }

    fn unscale(&mut self) {
        let scale_factor = Fr::from_u64(SCALE_FACTOR);

        let mut factor_power = Fr::one();
        for i in 0..self.coeffs.len() {
            factor_power = factor_power.mul(&scale_factor);
            self.coeffs[i] = self.coeffs[i].mul(&factor_power);
        }
    }

    fn destroy(&self) {}
}

impl Clone for Poly {
    fn clone(&self) -> Self {
        Poly { coeffs: self.coeffs.clone() }
    }
}

pub struct FFTSettings {
    pub max_width: usize,
    pub root_of_unity: Fr,
    pub expanded_roots_of_unity: Vec<Fr>,
    pub reverse_roots_of_unity: Vec<Fr>,
}

impl TFFTSettings<Fr> for FFTSettings {
    fn new(scale: usize) -> Result<FFTSettings, String> {
        FFTSettings::from_scale(scale)
    }

    fn get_max_width(&self) -> usize {
        self.max_width
    }

    fn get_expanded_roots_of_unity_at(&self, i: usize) -> Fr where Fr: Sized {
        self.expanded_roots_of_unity[i]
    }

    fn get_expanded_roots_of_unity(&self) -> &[Fr] {
        &self.expanded_roots_of_unity
    }

    fn get_reverse_roots_of_unity_at(&self, i: usize) -> Fr where Fr: Sized {
        self.reverse_roots_of_unity[i]
    }

    fn get_reversed_roots_of_unity(&self) -> &[Fr] {
        &self.reverse_roots_of_unity
    }

    fn destroy(&self) {}
}

impl Clone for Fr {
    fn clone(&self) -> Self {
        Fr(self.0.clone())
    }
}

impl Copy for Fr {}

impl FFTSettings {
    /// Create FFTSettings with roots of unity for a selected scale. Resulting roots will have a magnitude of 2 ^ max_scale.
    pub fn from_scale(max_scale: usize) -> Result<FFTSettings, String> {
        if max_scale >= SCALE2_ROOT_OF_UNITY.len() {
            return Err(String::from("Scale is expected to be within root of unity matrix row size"));
        }

        // max_width = 2 ^ max_scale
        let max_width: usize = 1 << max_scale;
        let root_of_unity = Fr::from_u64_arr(&SCALE2_ROOT_OF_UNITY[max_scale]);

        // create max_width of roots & store them reversed as well
        let expanded_roots_of_unity = expand_root_of_unity(&root_of_unity, max_width).unwrap();
        let mut reverse_roots_of_unity = expanded_roots_of_unity.clone();
        reverse_roots_of_unity.reverse();

        Ok(FFTSettings {
            max_width,
            root_of_unity,
            expanded_roots_of_unity,
            reverse_roots_of_unity,
        })
    }
}

impl Clone for FFTSettings {
    fn clone(&self) -> Self {
        let mut output = FFTSettings::from_scale(0).unwrap();
        output.max_width = self.max_width;
        output.root_of_unity = self.root_of_unity.clone();
        output.expanded_roots_of_unity = self.expanded_roots_of_unity.clone();
        output.reverse_roots_of_unity = self.reverse_roots_of_unity.clone();
        output
    }
}

pub struct KZGSettings {
    pub fs: FFTSettings,
    // Both secret_g1 and secret_g2 have the same number of elements
    pub secret_g1: Vec<G1>,
    pub secret_g2: Vec<G2>,
}
