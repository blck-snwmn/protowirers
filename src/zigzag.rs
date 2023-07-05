pub(crate) fn encode<T: ZigZag>(n: T) -> T::Output {
    n.encode()
}

// こういうdecode()も書けるが、decode文脈だと入力がu128なので、合わない。。。
// pub(crate) fn decode<T: ZigZag>(n: T::Output) -> T {
//     ZigZag::decode(n)
// }

pub(crate) fn decode(n: u128) -> i128 {
    let r = (n >> 1) as i128;
    let l = (n & 1) as i128;
    (r ^ -l)
}

// pub(crate) fn decode_ex<T, U, V>(n: T) -> V
// where
//     T: std::ops::BitAnd<Output = U> + std::ops::Shr<Output = U> + Copy + From<usize>,
//     U: std::ops::Neg<Output = U> + std::ops::BitXor<Output = V>,
// {
//     let x: T = 1.into();
//     let r = n >> x;
//     let l = n & x;
//     let l = -l;
//     let r = r ^ l;
//     r
// }

pub trait ZigZag: Sized {
    type Output;
    fn encode(&self) -> Self::Output;
    fn decode(n: Self::Output) -> Self;
}

macro_rules! zigzag_impl {
    ($T:ty, $OUT:ty, $SIFT:literal) => {
        impl ZigZag for $T {
            type Output = $OUT;
            fn encode(&self) -> Self::Output {
                ((self << 1) ^ (self >> $SIFT)) as Self::Output
            }
            fn decode(n: Self::Output) -> Self {
                let r = (n >> 1) as Self;
                let l = (n & 1) as Self;
                r ^ -l
            }
        }
    };
}

// TODO BITS定数が来たら、SHIFTは置き換える
zigzag_impl!(i32, u32, 31);
zigzag_impl!(i64, u64, 63);

// macro で実装されるのは以下。（参考までに残しておく）
// impl ZigZag for i32 {
//     type Output = u32;
//     fn encode(&self) -> Self::Output {
//         ((self << 1) ^ (self >> 31)) as u32
//     }
//     fn decode(n: Self::Output) -> Self {
//         let r = (n >> 1) as Self;
//         let l = (n & 1) as Self;
//         r ^ -l
//     }
// }

// impl ZigZag for i64 {
//     type Output = u64;
//     fn encode(&self) -> Self::Output {
//         ((self << 1) ^ (self >> 63)) as u64
//     }
//     fn decode(n: Self::Output) -> Self {
//         let r = (n >> 1) as Self;
//         let l = (n & 1) as Self;
//         r ^ -l
//     }
// }

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_encode() {
        assert_eq!(encode(0), 0_u32);
        assert_eq!(encode(-1), 1_u32);
        assert_eq!(encode(1), 2_u32);
        assert_eq!(encode(-2), 3_u32);
        assert_eq!(encode(2), 4_u32);
        assert_eq!(encode(2147483647), 4294967294_u32);
        assert_eq!(encode(-2147483648), 4294967295_u32);
        assert_eq!(encode(2147483647_i32), 4294967294);
        assert_eq!(encode(-2147483648_i32), 4294967295);
        assert_eq!(encode(-9223372036854775808_i64), 18446744073709551615);
        assert_eq!(encode(9223372036854775807_i64), 18446744073709551614);
    }
    #[test]
    fn test_decode_zigzag() {
        assert_eq!(decode(0), 0);
        assert_eq!(decode(1), -1);
        assert_eq!(decode(2), 1);
        assert_eq!(decode(3), -2);
        assert_eq!(decode(4), 2);
        assert_eq!(decode(4294967294), 2147483647);
        assert_eq!(decode(4294967295), -2147483648);
    }
}
