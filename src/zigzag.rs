use anyhow::Result;

pub(crate) fn encode<T: ZigZag>(n: T) -> T::Output {
    n.encode()
}

/// Varint(u128) を ZigZag 復号し i128 を返します（範囲チェックなしの内部用）。
pub(crate) fn decode_raw(n: u128) -> i128 {
    let r = (n >> 1) as i128;
    let l = (n & 1) as i128;
    r ^ -l
}

/// ZigZag 復号後に目標型へ変換します。範囲外の場合は Err を返します。
pub(crate) fn decode<T>(n: u128) -> Result<T>
where
    T: std::convert::TryFrom<i128>,
    <T as std::convert::TryFrom<i128>>::Error: std::error::Error + Send + Sync + 'static,
{
    let v = decode_raw(n);
    let r = std::convert::TryFrom::try_from(v)?;
    Ok(r)
}

/// ZigZag のエンコードを提供するトレイト（decode は自由関数として提供）。
pub trait ZigZag: Sized {
    type Output;
    fn encode(&self) -> Self::Output;

    // 理由: `ZigZag` に `decode` を持たせない
    // - デコードの起点は常に Varint の `u128` 値。
    //   まず `u128` を ZigZag 復号して `i128` を得るのが自然。
    //   そのため decode は `decode_raw(u128) -> i128` と
    //   `decode<T: TryFrom<i128>>(u128) -> Result<T>` の自由関数で提供する。
    // - `ZigZag::decode(Output)` を追加すると、
    //   `u128` → `Output(u32/u64)` の安全な前段変換やジェネリクスが必要になり、
    //   読みやすさ・保守性を下げる。
    // - 結論: トレイトはエンコード専用（`encode(&self)` のみ）。デコードは自由関数で扱う。
}

macro_rules! zigzag_impl {
    ($T:ty, $OUT:ty) => {
        impl ZigZag for $T {
            type Output = $OUT;
            fn encode(&self) -> Self::Output {
                ((self << 1) ^ (self >> (<$OUT>::BITS - 1))) as Self::Output
            }
        }
    };
}

zigzag_impl!(i32, u32);
zigzag_impl!(i64, u64);

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
        assert_eq!(decode_raw(0), 0);
        assert_eq!(decode_raw(1), -1);
        assert_eq!(decode_raw(2), 1);
        assert_eq!(decode_raw(3), -2);
        assert_eq!(decode_raw(4), 2);
        assert_eq!(decode_raw(4294967294), 2147483647);
        assert_eq!(decode_raw(4294967295), -2147483648);
    }
}
