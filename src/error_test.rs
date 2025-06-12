#[cfg(test)]
mod tests {
    use crate::decode::DecodeError;
    use crate::error::ProtowiresError;
    use crate::parser::{ParseError, Parser};
    use crate::wire::{TypeLengthDelimited, TypeVairant, WireDataLengthDelimited, WireDataVarint};

    #[test]
    fn test_value_too_large_error() {
        let result = Parser::<u32>::parse(
            &WireDataVarint::new((u32::MAX as u128) + 1),
            TypeVairant::Uint32,
        );

        match result {
            Err(ProtowiresError::TryFromInt(_)) => {
                // u32の範囲を超える値はTryFromIntErrorとして処理される
            }
            _ => panic!("Expected TryFromInt error"),
        }
    }

    #[test]
    fn test_parse_error_type_mismatch() {
        // Stringはvariantではなく、lengthDelimitedタイプなので、
        // WireDataLengthDelimited経由でテストする必要がある
        let wire_data = WireDataLengthDelimited::new(vec![0x41, 0x42]);
        // Bytesタイプを期待しているところにWireStringを渡す
        let result: Result<Vec<u8>, ProtowiresError> =
            Parser::<Vec<u8>>::parse(&wire_data, TypeLengthDelimited::WireString);

        match result {
            Err(ProtowiresError::Parse(ParseError::UnexpectTypeError { want, got })) => {
                assert!(want.contains("Bytes"));
                assert!(got.contains("WireString"));
            }
            _ => panic!("Expected Parse error with type mismatch"),
        }
    }

    #[test]
    fn test_decode_error_unexpected_format() {
        use crate::decode::decode_variants_slice;

        let invalid_bytes = &[
            0b10000000, 0b10000100, 0b10101111, 0b01011111, 0b00000100, 0b10000110,
        ];
        let result = decode_variants_slice(invalid_bytes);

        match result {
            Err(ProtowiresError::Decode(DecodeError::UnexpectFormat)) => {
                // UnexpectFormat variant has no associated message
            }
            _ => panic!("Expected Decode error with UnexpectFormat"),
        }
    }

    #[test]
    fn test_io_error_propagation() {
        use std::io::{Cursor, Read};

        // Create a cursor that will fail on read
        let mut cursor = Cursor::new(vec![]);
        let mut buf = [0u8; 10];

        // Force EOF error
        cursor.set_position(100);
        let io_result = cursor.read_exact(&mut buf);

        // Convert to our error type
        let our_result: Result<(), ProtowiresError> = io_result.map_err(Into::into);

        match our_result {
            Err(ProtowiresError::Io(_)) => {
                // IO error properly propagated
            }
            _ => panic!("Expected IO error"),
        }
    }

    #[test]
    fn test_utf8_error() {
        use crate::parser::Parser;
        use crate::wire::{TypeLengthDelimited, WireDataLengthDelimited};

        // Invalid UTF-8 sequence
        let invalid_utf8 = vec![0xFF, 0xFE, 0xFD];
        let wire_data = WireDataLengthDelimited::new(invalid_utf8);
        let result = Parser::<String>::parse(&wire_data, TypeLengthDelimited::WireString);

        match result {
            Err(ProtowiresError::FromUtf8(_)) => {
                // UTF-8 error properly caught
            }
            _ => panic!("Expected UTF-8 error"),
        }
    }

    #[test]
    fn test_error_display() {
        let err = ProtowiresError::ValueTooLarge {
            value: 1000,
            max: 100,
        };

        let msg = err.to_string();
        assert!(msg.contains("value too large"));
        assert!(msg.contains("1000"));
        assert!(msg.contains("100"));
    }

    #[test]
    fn test_decode_error_variants() {
        let err1 = DecodeError::UnexpectFormat;
        let err2 = DecodeError::UnexpectedWireDataValue(99);

        let proto_err1: ProtowiresError = err1.into();
        let proto_err2: ProtowiresError = err2.into();

        match proto_err1 {
            ProtowiresError::Decode(DecodeError::UnexpectFormat) => {
                // UnexpectFormat variant has no associated message
            }
            _ => panic!("Unexpected error variant"),
        }

        match proto_err2 {
            ProtowiresError::Decode(DecodeError::UnexpectedWireDataValue(val)) => {
                assert_eq!(val, 99);
            }
            _ => panic!("Unexpected error variant"),
        }
    }
}
