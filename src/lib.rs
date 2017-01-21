#![feature(test)]
extern crate test;

extern crate rand;

use rand::Rng;

/// The Bernoulli distribution
///
/// Bernoulli distribution is the probability distribution of a random variable which takes the
/// value (bit) `1` with success probability of `p` and the value `0` with failure probability of
/// `q = 1 − p`.
///
/// The distribution assumes unbiased underlying Rng.
///
/// This distribution is implemented as a Rng wrapper, rather than an impl of `Sample` because the
/// distribution keeps the random samples from the underlying Rng between calls to generate a
/// random number.
pub struct BernoulliRng<R> {
    p: f64,
    low: f64,
    high: f64,
    rng: R,
    shift: u8,
    bits: u64
}

impl<R: Rng> BernoulliRng<R> {
    pub fn new(rng: R, probability: f64) -> Result<BernoulliRng<R>, Error> {
        if probability.is_sign_negative() || probability > 1.0 {
            return Err(Error::InvalidProbability);
        }
        Ok(BernoulliRng {
            p: probability,
            low: 0.0,
            high: 1.0,
            rng: rng,
            shift: 0,
            bits: 0
        })
    }

    #[inline]
    fn next_bit(&mut self) -> bool {
        if self.shift == 0 {
            self.bits = self.rng.next_u64();
            self.shift = 64;
        }
        let bit = (self.bits & 1) == 1;
        self.bits >>= 1;
        self.shift -= 1;
        bit
    }
}

pub enum Error {
    InvalidProbability
}

impl ::std::error::Error for Error {
    fn description(&self) -> &str {
        match self {
            &Error::InvalidProbability => "invalid probability specified",
        }
    }
}

impl ::std::fmt::Display for Error {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            &Error::InvalidProbability => write!(f, "invalid probability specified")
        }
    }
}

impl ::std::fmt::Debug for Error {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            &Error::InvalidProbability => write!(f, "invalid probability specified")
        }
    }
}

impl<R: Rng> Rng for BernoulliRng<R> {
    fn next_u32(&mut self) -> u32 {
        let (mut ret, mut i, mut high, mut low) = (0, 0, self.high, self.low);
        let (p, p_recip, pinv_recip) = (self.p, self.p.recip(), (1.0 - self.p).recip());
        while i != 32 {
            if high < p {
                ret = ret << 1 | 1;
                i += 1;
                low *= p_recip;
                high *= p_recip;
            } else if low > p {
                ret = ret << 1;
                i += 1;
                low = (low - p) * pinv_recip;
                high = (high - p) * pinv_recip;
            } else {
                let mid = 0.5 * (low + high);
                if self.next_bit() {
                    low = mid;
                } else {
                    high = mid;
                }
            }
        }
        self.high = high;
        self.low = low;
        ret
    }
}

#[test]
fn it_works() {
    let mut v = 0;
    let mut rng = rand::thread_rng();
    let mut distr = if let Ok(v) = BernoulliRng::new(rng, 0.75) { v } else { panic!() };
    for i in 0..10000 {
        let o = distr.next_u32();
        v += o.count_ones();
    }
    println!("{}/{}", v, 320000);

}

#[bench]
fn it_works_fast(b: &mut test::Bencher) {
    let mut rng = rand::thread_rng();
    let mut distr = if let Ok(v) = BernoulliRng::new(rng, 0.75) { v } else { panic!() };
    b.iter(||{
        for i in 0..10000 {
            distr.next_u32();
        }
    });
}

#[bench]
fn it_works_faster(b: &mut test::Bencher) {
    let mut rng = rand::thread_rng();
    b.iter(||{
        for i in 0..10000 {
            rng.next_u32();
        }
    });
}
