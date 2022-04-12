#[cfg(test)]
mod test_encode {
    use crate::{
        abi_type::{AbiType, AbiValue},
        biguint_ext::BigUintExt,
    };
    use algonaut_core::Address;
    use num_bigint::{BigUint, RandBigInt};
    use rand::Rng;
    use std::{convert::TryInto, ops::Shl};

    #[test]
    fn test_encode_int() {
        let mut rng = rand::thread_rng();

        for size in (8..512).step_by(8) {
            let upper_limit: BigUint = BigUint::from(1u8).shl(size);
            let uint_type = AbiType::uint(size).unwrap();

            for _ in 0..1000 {
                let random_int: BigUint = rng.gen_biguint(size.clone().try_into().unwrap());

                let expected = BigUint::from(random_int.clone())
                    .to_bytes_be_padded(size / 8)
                    .unwrap();

                let encoded_res = uint_type.encode(AbiValue::Int(random_int));
                assert!(encoded_res.is_ok(), "encoding from uint type fail");
                let encoded = encoded_res.unwrap();

                assert_eq!(expected, encoded, "encode uint not match with expected");
            }

            let largest: BigUint = upper_limit - BigUint::from(1u8);
            let largest_bytes = largest.to_bytes_be();

            let largest_encoded_res = uint_type.encode(AbiValue::Int(largest));
            assert!(largest_encoded_res.is_ok(), "largest uint encode error");
            let largest_encoded = largest_encoded_res.unwrap();

            assert_eq!(
                largest_bytes, largest_encoded,
                "encode uint largest do not match with expected"
            );
        }
    }

    #[test]
    fn test_encode_ufixed() {
        let mut rng = rand::thread_rng();

        for size in (8..512).step_by(8) {
            let upper_limit: BigUint = BigUint::from(1u8).shl(size);

            for precision in 1..160 {
                let ufixed_type_res = AbiType::ufixed(size, precision);
                assert!(ufixed_type_res.is_ok(), "make ufixed type fail");
                let ufixed_type = ufixed_type_res.unwrap();

                for _ in 0..20 {
                    let random_int: BigUint = rng.gen_biguint(size.clone().try_into().unwrap());

                    let expected = BigUint::from(random_int.clone())
                        .to_bytes_be_padded(size / 8)
                        .unwrap();

                    let encoded_res = ufixed_type.encode(AbiValue::Int(random_int));
                    assert!(encoded_res.is_ok(), "encoding from ufixed type fail");
                    let encoded = encoded_res.unwrap();

                    assert_eq!(expected, encoded, "encode ufixed not match with expected");
                }

                // (2^[bitSize] - 1) / (10^[precision]) test
                let largest: BigUint = upper_limit.clone() - BigUint::from(1u8);
                let largest_bytes = largest.to_bytes_be();

                let largest_encoded_res = ufixed_type.encode(AbiValue::Int(largest));
                assert!(largest_encoded_res.is_ok(), "largest ufixed encode error");
                let largest_encoded = largest_encoded_res.unwrap();

                assert_eq!(
                    largest_bytes, largest_encoded,
                    "encode ufixed largest do not match with expected"
                );
            }
        }
    }

    #[test]
    fn test_encode_address() {
        let mut rng = rand::thread_rng();

        let upper_limit: BigUint = BigUint::from(1u8).shl(256u16) - BigUint::from(1u8);
        let upper_encoded = BigUint::from(upper_limit).to_bytes_be_padded(32).unwrap();

        for _ in 0..1000 {
            let mut rand: BigUint = rng.gen_biguint(256);

            while rand >= BigUint::from(1u8).shl(256) {
                rand = rng.gen_biguint(256);
                let addr_encode = rand.to_bytes_be_padded(32).unwrap();
                assert_eq!(
                    AbiType::address()
                        .encode(AbiValue::Address(Address(
                            addr_encode.clone().try_into().unwrap()
                        )))
                        .unwrap(),
                    addr_encode
                );
            }
            assert_eq!(
                AbiType::address()
                    .encode(AbiValue::Address(Address(
                        upper_encoded.clone().try_into().unwrap()
                    )))
                    .unwrap(),
                upper_encoded
            );
        }
    }

    #[test]
    fn test_encode_bool() {
        assert_eq!(
            AbiType::bool().encode(AbiValue::Bool(false)).unwrap(),
            &[0x00]
        );
        assert_eq!(
            AbiType::bool().encode(AbiValue::Bool(true)).unwrap(),
            &[0x80]
        );
    }

    #[test]
    fn test_encode_byte() {
        for i in 0..=u8::MAX {
            assert_eq!(AbiType::byte().encode(AbiValue::Byte(i)).unwrap(), &[i]);
        }
    }

    #[test]
    fn test_encode_string() {
        let rng = rand::thread_rng();
        for length in 1..=400 {
            for _ in 0..10 {
                let gen_string: String = rng
                    .clone()
                    .sample_iter::<char, _>(rand::distributions::Standard)
                    .take(length)
                    .collect();

                let mut str_bytes = gen_string.as_bytes().to_vec();
                let mut header = BigUint::from(str_bytes.len())
                    .to_bytes_be_padded(2)
                    .unwrap();
                let mut gen_bytes = vec![];
                gen_bytes.append(&mut header);
                gen_bytes.append(&mut str_bytes);

                assert_eq!(
                    gen_bytes,
                    AbiType::string()
                        .encode(AbiValue::String(gen_string))
                        .unwrap(),
                );
            }
        }
    }

    #[test]
    fn test_encode_bool_array0() {
        let inputs = &[true, false, false, true, true];
        let input_values: Vec<AbiValue> = inputs.into_iter().map(|b| AbiValue::Bool(*b)).collect();

        let expected: &[u8] = &[0b10011000];

        assert_eq!(
            AbiType::static_array(AbiType::bool(), 5)
                .encode(AbiValue::Array(input_values))
                .unwrap(),
            expected
        );
    }

    #[test]
    fn test_encode_bool_array1() {
        let inputs = &[
            false, false, false, true, true, false, true, false, true, false, true,
        ];
        let input_values: Vec<AbiValue> = inputs.into_iter().map(|b| AbiValue::Bool(*b)).collect();

        let expected: &[u8] = &[0b00011010, 0b10100000];

        assert_eq!(
            expected,
            AbiType::static_array(AbiType::bool(), 11)
                .encode(AbiValue::Array(input_values))
                .unwrap(),
        );
    }

    #[test]
    fn test_encode_bool_array2() {
        let inputs = &[
            false, false, false, true, true, false, true, false, true, false, true,
        ];
        let input_values: Vec<AbiValue> = inputs.into_iter().map(|b| AbiValue::Bool(*b)).collect();

        let expected: &[u8] = &[0x00, 0x0B, 0b00011010, 0b10100000];

        assert_eq!(
            expected,
            AbiType::dynamic_array(AbiType::bool())
                .encode(AbiValue::Array(input_values))
                .unwrap(),
        );
    }

    #[test]
    fn test_uint_array() {
        let inputs: &[u32] = &[1, 2, 3, 4, 5, 6, 7, 8];
        let input_values: Vec<AbiValue> = inputs
            .into_iter()
            .map(|i| AbiValue::Int(BigUint::from(*i)))
            .collect();

        let expected: &[u8] = &[
            0x00, 0x08, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x03, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x04, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x05,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x06, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x07, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x08,
        ];

        let type_ = "uint64[]".parse::<AbiType>().unwrap();

        assert_eq!(
            expected,
            type_.encode(AbiValue::Array(input_values)).unwrap(),
        );
    }

    #[test]
    fn test_dynamic_tuple() {
        let elements = vec![
            AbiValue::String("ABC".to_owned()),
            AbiValue::Bool(true),
            AbiValue::Bool(false),
            AbiValue::Bool(true),
            AbiValue::Bool(false),
            AbiValue::String("DEF".to_owned()),
        ];

        let expected = vec![
            0x00, 0x05, 0b10100000, 0x00, 0x0A, 0x00, 0x03, b'A', b'B', b'C', 0x00, 0x03, b'D',
            b'E', b'F',
        ];

        let type_ = "(string,bool,bool,bool,bool,string)"
            .parse::<AbiType>()
            .unwrap();

        assert_eq!(expected, type_.encode(AbiValue::Array(elements)).unwrap(),);
    }

    #[test]
    fn test_bool_array_tuple() {
        let elements = vec![
            AbiValue::Array(vec![AbiValue::Bool(true), AbiValue::Bool(true)]),
            AbiValue::Array(vec![AbiValue::Bool(true), AbiValue::Bool(true)]),
        ];

        let expected = vec![0b11000000, 0b11000000];

        let type_ = "(bool[2],bool[2])".parse::<AbiType>().unwrap();

        assert_eq!(expected, type_.encode(AbiValue::Array(elements)).unwrap(),);
    }

    #[test]
    fn test_bool_array_tuple1() {
        let elements = vec![
            AbiValue::Array(vec![AbiValue::Bool(true), AbiValue::Bool(true)]),
            AbiValue::Array(vec![AbiValue::Bool(true), AbiValue::Bool(true)]),
        ];

        let expected = vec![0b11000000, 0x00, 0x03, 0x00, 0x02, 0b11000000];

        let type_ = "(bool[2],bool[])".parse::<AbiType>().unwrap();

        assert_eq!(expected, type_.encode(AbiValue::Array(elements)).unwrap(),);
    }

    #[test]
    fn test_bool_array_tuple2() {
        let elements = vec![
            AbiValue::Array(vec![AbiValue::Bool(true), AbiValue::Bool(true)]),
            AbiValue::Array(vec![AbiValue::Bool(true), AbiValue::Bool(true)]),
        ];

        let expected = vec![
            0x00, 0x04, 0x00, 0x07, 0x00, 0x02, 0b11000000, 0x00, 0x02, 0b11000000,
        ];

        let type_ = "(bool[],bool[])".parse::<AbiType>().unwrap();

        assert_eq!(expected, type_.encode(AbiValue::Array(elements)).unwrap(),);
    }

    #[test]
    fn test_bool_array_tuple3() {
        let elements = vec![AbiValue::Array(vec![]), AbiValue::Array(vec![])];

        let expected = vec![0x00, 0x04, 0x00, 0x06, 0x00, 0x00, 0x00, 0x00];

        let type_ = "(bool[],bool[])".parse::<AbiType>().unwrap();

        assert_eq!(expected, type_.encode(AbiValue::Array(elements)).unwrap(),);
    }

    #[test]
    fn test_empty_tuple() {
        let elements = vec![];
        let expected: Vec<u8> = vec![];

        let type_ = "()".parse::<AbiType>().unwrap();

        assert_eq!(expected, type_.encode(AbiValue::Array(elements)).unwrap(),);
    }
}

#[cfg(test)]
mod test_decode {
    use std::{convert::TryInto, ops::Shl};

    use algonaut_core::Address;
    use num_bigint::{BigUint, RandBigInt};
    use rand::Rng;

    use crate::{
        abi_type::{AbiType, AbiValue},
        biguint_ext::BigUintExt,
    };

    #[test]
    fn test_decode_uint() {
        let mut rng = rand::thread_rng();

        for size in (8..512).step_by(8) {
            for _ in 0..1000 {
                let random_int: BigUint = rng.gen_biguint(size.clone().try_into().unwrap());
                let encoded_int = AbiType::uint(size)
                    .unwrap()
                    .encode(AbiValue::Int(random_int.clone()))
                    .unwrap();

                assert_eq!(
                    AbiValue::Int(random_int),
                    AbiType::uint(size).unwrap().decode(&encoded_int).unwrap(),
                );
            }

            // 2^[bitSize] - 1
            let largest: BigUint = BigUint::from(1u8).shl(size) - BigUint::from(1u8);

            let mut expected = vec![];
            for _ in 0..(size / 8) {
                expected.push(0xff);
            }

            assert_eq!(
                AbiValue::Int(largest),
                AbiType::uint(size).unwrap().decode(&expected).unwrap(),
            );
        }
    }

    #[test]
    fn test_decode_ufixed() {
        let mut rng = rand::thread_rng();

        for size in (8..512).step_by(8) {
            for precision in 1..160 {
                for _ in 0..20 {
                    let random_int: BigUint = rng.gen_biguint(size.clone().try_into().unwrap());
                    let encoded_int = AbiType::ufixed(size, precision)
                        .unwrap()
                        .encode(AbiValue::Int(random_int.clone()))
                        .unwrap();
                    assert_eq!(
                        AbiValue::Int(random_int),
                        AbiType::ufixed(size, precision)
                            .unwrap()
                            .decode(&encoded_int)
                            .unwrap(),
                    );
                }

                let largest: BigUint = BigUint::from(1u8).shl(size) - BigUint::from(1u8);
                let mut expected = vec![];
                for _ in 0..(size / 8) {
                    expected.push(0xff);
                }

                assert_eq!(
                    AbiValue::Int(largest),
                    AbiType::ufixed(size, precision)
                        .unwrap()
                        .decode(&expected)
                        .unwrap(),
                );
            }
        }
    }

    #[test]
    fn test_decode_address() {
        let mut rng = rand::thread_rng();

        let upper_limit: BigUint = BigUint::from(1u8).shl(256u16) - BigUint::from(1u8);
        let upper_encoded = BigUint::from(upper_limit).to_bytes_be_padded(32).unwrap();

        for _ in 0..1000 {
            let mut rand: BigUint = rng.gen_biguint(256);

            while rand >= BigUint::from(1u8).shl(256) {
                rand = rng.gen_biguint(256);
                let addr_encode = rand.to_bytes_be_padded(32).unwrap();

                assert_eq!(
                    AbiValue::Address(Address(addr_encode.clone().try_into().unwrap())),
                    AbiType::address().decode(&addr_encode).unwrap(),
                );
            }

            assert_eq!(
                AbiValue::Address(Address(upper_encoded.clone().try_into().unwrap())),
                AbiType::address().decode(&upper_encoded).unwrap(),
            );
        }
    }

    #[test]
    fn test_decode_bool() {
        assert_eq!(
            AbiType::bool().decode(&[0x00]).unwrap(),
            AbiValue::Bool(false)
        );

        assert_eq!(
            AbiType::bool().decode(&[0x80]).unwrap(),
            AbiValue::Bool(true)
        );
    }

    #[test]
    fn test_decode_byte() {
        for i in 0..=u8::MAX {
            assert_eq!(AbiType::byte().decode(&[i]).unwrap(), AbiValue::Byte(i));
        }
    }

    #[test]
    fn test_decode_string() {
        let rng = rand::thread_rng();
        for length in 1..=400 {
            for _ in 0..10 {
                let gen_string: String = rng
                    .clone()
                    .sample_iter::<char, _>(rand::distributions::Standard)
                    .take(length)
                    .collect();

                let mut str_bytes = gen_string.as_bytes().to_vec();
                let mut header = BigUint::from(str_bytes.len())
                    .to_bytes_be_padded(2)
                    .unwrap();
                let mut gen_bytes = vec![];
                gen_bytes.append(&mut header);
                gen_bytes.append(&mut str_bytes);

                assert_eq!(
                    AbiValue::String(gen_string),
                    AbiType::string().decode(&gen_bytes).unwrap(),
                );
            }
        }
    }

    #[test]
    fn test_decode_bool_array0() {
        let inputs = &[true, false, false, true, true];
        let input_values: Vec<AbiValue> = inputs.into_iter().map(|b| AbiValue::Bool(*b)).collect();

        let type_ = "bool[5]".parse::<AbiType>().unwrap();

        assert_eq!(
            type_.decode(&[0b10011000]).unwrap(),
            AbiValue::Array(input_values)
        );
    }

    #[test]
    fn test_decode_bool_array1() {
        let inputs = &[
            false, false, false, true, true, false, true, false, true, false, true,
        ];
        let input_values: Vec<AbiValue> = inputs.into_iter().map(|b| AbiValue::Bool(*b)).collect();

        let type_ = "bool[11]".parse::<AbiType>().unwrap();

        assert_eq!(
            type_.decode(&[0b00011010, 0b10100000]).unwrap(),
            AbiValue::Array(input_values)
        );
    }

    #[test]
    fn test_decode_bool_array2() {
        let inputs = &[
            false, false, false, true, true, false, true, false, true, false, true,
        ];
        let input_values: Vec<AbiValue> = inputs.into_iter().map(|b| AbiValue::Bool(*b)).collect();

        let type_ = "bool[]".parse::<AbiType>().unwrap();

        assert_eq!(
            type_.decode(&[0x00, 0x0B, 0b00011010, 0b10100000]).unwrap(),
            AbiValue::Array(input_values)
        );
    }

    #[test]
    fn test_decode_uint_array() {
        let inputs: &[u32] = &[1, 2, 3, 4, 5, 6, 7, 8];
        let input_values: Vec<AbiValue> = inputs
            .into_iter()
            .map(|i| AbiValue::Int(BigUint::from(*i)))
            .collect();

        let type_ = "uint64[8]".parse::<AbiType>().unwrap();

        assert_eq!(
            type_
                .decode(&[
                    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00,
                    0x00, 0x00, 0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x03, 0x00, 0x00,
                    0x00, 0x00, 0x00, 0x00, 0x00, 0x04, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                    0x05, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x06, 0x00, 0x00, 0x00, 0x00,
                    0x00, 0x00, 0x00, 0x07, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x08,
                ])
                .unwrap(),
            AbiValue::Array(input_values)
        );
    }

    #[test]
    fn test_decode_dynamic_tuple() {
        let tuple = vec![
            AbiValue::String("ABC".to_owned()),
            AbiValue::Bool(true),
            AbiValue::Bool(false),
            AbiValue::Bool(true),
            AbiValue::Bool(false),
            AbiValue::String("DEF".to_owned()),
        ];

        let type_ = "(string,bool,bool,bool,bool,string)"
            .parse::<AbiType>()
            .unwrap();

        assert_eq!(
            type_
                .decode(&[
                    0x00, 0x05, 0b10100000, 0x00, 0x0A, 0x00, 0x03, b'A', b'B', b'C', 0x00, 0x03,
                    b'D', b'E', b'F',
                ])
                .unwrap(),
            AbiValue::Array(tuple)
        );
    }

    #[test]
    fn test_decode_bool_array_tuple() {
        let elements = vec![
            AbiValue::Array(vec![AbiValue::Bool(true), AbiValue::Bool(true)]),
            AbiValue::Array(vec![AbiValue::Bool(true), AbiValue::Bool(true)]),
        ];

        let type_ = "(bool[2],bool[2])".parse::<AbiType>().unwrap();

        assert_eq!(
            type_.decode(&[0b11000000, 0b11000000]).unwrap(),
            AbiValue::Array(elements)
        );
    }

    #[test]
    fn test_decode_bool_array_tuple1() {
        let elements = vec![
            AbiValue::Array(vec![AbiValue::Bool(true), AbiValue::Bool(true)]),
            AbiValue::Array(vec![AbiValue::Bool(true), AbiValue::Bool(true)]),
        ];

        let type_ = "(bool[2],bool[])".parse::<AbiType>().unwrap();

        assert_eq!(
            type_
                .decode(&[0b11000000, 0x00, 0x03, 0x00, 0x02, 0b11000000])
                .unwrap(),
            AbiValue::Array(elements)
        );
    }

    #[test]
    fn test_decode_bool_array_tuple2() {
        let elements = vec![
            AbiValue::Array(vec![AbiValue::Bool(true), AbiValue::Bool(true)]),
            AbiValue::Array(vec![AbiValue::Bool(true), AbiValue::Bool(true)]),
        ];

        let type_ = "(bool[],bool[])".parse::<AbiType>().unwrap();

        assert_eq!(
            type_
                .decode(&[0x00, 0x04, 0x00, 0x07, 0x00, 0x02, 0b11000000, 0x00, 0x02, 0b11000000,])
                .unwrap(),
            AbiValue::Array(elements)
        );
    }

    #[test]
    fn test_decode_bool_array_tuple3() {
        let elements = vec![AbiValue::Array(vec![]), AbiValue::Array(vec![])];

        let type_ = "(bool[],bool[])".parse::<AbiType>().unwrap();

        assert_eq!(
            type_
                .decode(&[0x00, 0x04, 0x00, 0x06, 0x00, 0x00, 0x00, 0x00])
                .unwrap(),
            AbiValue::Array(elements)
        );
    }

    #[test]
    fn test_empty_tuple() {
        let elements = vec![];
        let type_ = "()".parse::<AbiType>().unwrap();
        assert_eq!(type_.decode(&[]).unwrap(), AbiValue::Array(elements));
    }
}

#[cfg(test)]
mod test_decode_invalid {
    use crate::abi_type::AbiType;

    #[test]
    fn test_bool_array() {
        let input = &[0xff];
        let type_ = "bool[9]".parse::<AbiType>().unwrap();
        assert!(type_.decode(input).is_err());
    }

    #[test]
    fn test_bool_array1() {
        let input = &[0xff, 0x00];
        let type_ = "bool[8]".parse::<AbiType>().unwrap();
        assert!(type_.decode(input).is_err());
    }

    #[test]
    fn test_bool_array2() {
        let input = &[0x00, 0x0A, 0b10101010];
        let type_ = "bool[]".parse::<AbiType>().unwrap();
        assert!(type_.decode(input).is_err());
    }

    #[test]
    fn test_bool_array3() {
        let input = &[0x00, 0x05, 0b10100000, 0x00];
        let type_ = "bool[]".parse::<AbiType>().unwrap();
        assert!(type_.decode(input).is_err());
    }

    #[test]
    fn test_uint_array_0() {
        let input = &[
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x03, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x04, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x05, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x06, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x07,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x08,
        ];
        let type_ = "uint64[10]".parse::<AbiType>().unwrap();
        assert!(type_.decode(input).is_err());
    }

    #[test]
    fn test_uint_array_1() {
        let input = &[
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x03, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x04, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x05, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x06, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x07,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x08,
        ];
        let type_ = "uint64[6]".parse::<AbiType>().unwrap();
        assert!(type_.decode(input).is_err());
    }

    #[test]
    fn test_tuple_0() {
        let input = &[
            0x00, 0x04, 0b10100000, 0x00, 0x0A, 0x00, 0x03, b'A', b'B', b'C', 0x00, 0x03, b'D',
            b'E', b'F',
        ];
        let type_ = "(string,bool,bool,bool,bool,string)"
            .parse::<AbiType>()
            .unwrap();
        assert!(type_.decode(input).is_err());
    }

    #[test]
    fn test_bool_array_tuple_0() {
        let encode0 = &[0b11000000, 0b11000000, 0x00];
        let encode1 = &[0b11000000];

        let type_ = "(bool[2],bool[2])".parse::<AbiType>().unwrap();
        assert!(type_.decode(encode0).is_err());
        assert!(type_.decode(encode1).is_err());
    }

    #[test]
    fn test_bool_array_tuple_1() {
        let encoded = &[0b11000000, 0x03, 0x00, 0x02, 0b11000000];

        let type_ = "(bool[2],bool[])".parse::<AbiType>().unwrap();
        assert!(type_.decode(encoded).is_err());
    }

    #[test]
    fn test_bool_array_tuple_2() {
        let encoded = &[
            0x00, 0x04, 0x00, 0x08, 0x00, 0x02, 0b11000000, 0x00, 0x00, 0x02, 0b11000000,
        ];

        let type_ = "(bool[],bool[])".parse::<AbiType>().unwrap();
        assert!(type_.decode(encoded).is_err());
    }

    #[test]
    fn test_bool_array_tuple_3() {
        let encoded = &[0x00, 0x04, 0x00, 0x07, 0x00, 0x00, 0x00, 0x00];

        let type_ = "(bool[],bool[])".parse::<AbiType>().unwrap();
        assert!(type_.decode(encoded).is_err());
    }

    #[test]
    fn test_empty_tuple() {
        let encoded = &[0x0F];
        let type_ = "()".parse::<AbiType>().unwrap();
        assert!(type_.decode(encoded).is_err());
    }
}

#[cfg(test)]
mod test_roundrip {
    use std::convert::TryInto;

    use crate::{
        abi_type::{AbiType, AbiValue},
        biguint_ext::BigUintExt,
    };
    use algonaut_core::Address;
    use num_bigint::{BigUint, RandBigInt};
    use rand::Rng;

    #[derive(Debug, Clone)]
    struct RawValueWithAbiType {
        type_str: String,
        value: AbiValue,
    }

    impl RawValueWithAbiType {
        fn new(type_str: &str, value: AbiValue) -> RawValueWithAbiType {
            RawValueWithAbiType {
                type_str: type_str.to_owned(),
                value,
            }
        }
    }

    fn generate_random_int(bit_size: u64) -> BigUint {
        let mut rng = rand::thread_rng();
        let random_int: BigUint = rng.gen_biguint(bit_size);
        return random_int;
    }

    fn generate_static_array(test_value_pool: &mut [Vec<RawValueWithAbiType>]) {
        let mut rng = rand::thread_rng();

        for i in (0..test_value_pool[0].len()).step_by(400) {
            let mut value_arr = vec![];
            for cnt in 0..20 {
                value_arr.push(test_value_pool[0][i + cnt].value.clone());
            }
            let type_ = test_value_pool[0][i].type_str.clone().parse().unwrap();
            test_value_pool[6].push(RawValueWithAbiType::new(
                &AbiType::static_array(type_, 20).to_string(),
                AbiValue::Array(value_arr),
            ));
        }

        let mut value_byte_arr = vec![];
        for i in 0..20 {
            value_byte_arr.push(test_value_pool[2][i].value.clone());
        }
        test_value_pool[6].push(RawValueWithAbiType::new(
            "byte[20]",
            AbiValue::Array(value_byte_arr),
        ));

        let mut bool_arr = vec![];
        for _ in 0..20 {
            let index = if rng.gen_bool(0.5) { 0 } else { 1 };
            bool_arr.push(test_value_pool[3][index].value.clone());
        }
        test_value_pool[6].push(RawValueWithAbiType::new(
            "bool[20]",
            AbiValue::Array(bool_arr),
        ));

        let mut address_arr = vec![];
        for i in 0..20 {
            address_arr.push(test_value_pool[4][i].value.clone());
        }
        test_value_pool[6].push(RawValueWithAbiType::new(
            "address[20]",
            AbiValue::Array(address_arr),
        ));

        let mut string_arr = vec![];
        for i in 0..20 {
            string_arr.push(test_value_pool[5][i].value.clone());
        }
        test_value_pool[6].push(RawValueWithAbiType::new(
            "string[20]",
            AbiValue::Array(string_arr),
        ));
    }

    fn generate_dynamic_array(test_value_pool: &mut [Vec<RawValueWithAbiType>]) {
        let mut rng = rand::thread_rng();

        for i in (0..test_value_pool[0].len()).step_by(400) {
            let mut value_arr = vec![];
            for cnt in 0..20 {
                value_arr.push(test_value_pool[0][i + cnt].value.clone());
            }
            let type_ = test_value_pool[0][i].type_str.clone().parse().unwrap();
            test_value_pool[6].push(RawValueWithAbiType::new(
                &AbiType::dynamic_array(type_).to_string(),
                AbiValue::Array(value_arr),
            ));
        }

        let mut value_byte_arr = vec![];
        for i in 0..20 {
            value_byte_arr.push(test_value_pool[2][i].value.clone());
        }
        test_value_pool[7].push(RawValueWithAbiType::new(
            "byte[]",
            AbiValue::Array(value_byte_arr),
        ));

        let mut bool_arr = vec![];
        for _ in 0..20 {
            let index = if rng.gen_bool(0.5) { 0 } else { 1 };
            bool_arr.push(test_value_pool[3][index].value.clone());
        }
        test_value_pool[7].push(RawValueWithAbiType::new(
            "bool[]",
            AbiValue::Array(bool_arr),
        ));

        let mut address_arr = vec![];
        for i in 0..20 {
            address_arr.push(test_value_pool[4][i].value.clone());
        }
        test_value_pool[7].push(RawValueWithAbiType::new(
            "address[]",
            AbiValue::Array(address_arr),
        ));

        let mut string_arr = vec![];
        for i in 0..20 {
            string_arr.push(test_value_pool[5][i].value.clone());
        }
        test_value_pool[7].push(RawValueWithAbiType::new(
            "string[]",
            AbiValue::Array(string_arr),
        ));
    }

    fn generate_tuple(index: usize, test_value_pool: &mut [Vec<RawValueWithAbiType>]) {
        let mut rng = rand::thread_rng();

        for _ in 0..100 {
            let tuple_len = 1 + rng.gen_range(0..20);
            let mut tuple_values = vec![];
            let mut tuple_types: Vec<AbiType> = vec![];

            for _ in 0..tuple_len {
                let value_type_slot: usize = rng.gen_range(0..index as usize + 1);
                let value_index = rng.gen_range(0..test_value_pool[value_type_slot].len());
                tuple_values.push(test_value_pool[value_type_slot][value_index].value.clone());
                tuple_types.push(
                    test_value_pool[value_type_slot][value_index]
                        .type_str
                        .parse()
                        .unwrap(),
                );
            }

            let abi_str = AbiType::tuple(tuple_types).unwrap().to_string();
            test_value_pool[8].push(RawValueWithAbiType::new(
                &abi_str,
                AbiValue::Array(tuple_values),
            ))
        }
    }

    fn setup() -> Vec<Vec<RawValueWithAbiType>> {
        println!("start test setup");
        let rng = rand::thread_rng();

        let test_value_pool: &mut Vec<Vec<RawValueWithAbiType>> = &mut vec![];
        for _ in 0..9 {
            let v = vec![];
            test_value_pool.push(v);
        }

        for i in (8..512).step_by(8) {
            for _ in 0..200 {
                test_value_pool[0].push(RawValueWithAbiType::new(
                    &AbiType::uint(i).unwrap().to_string(),
                    AbiValue::Int(generate_random_int(i as u64)),
                ));
            }
            for j in 1..160 {
                test_value_pool[1].push(RawValueWithAbiType::new(
                    &AbiType::ufixed(i, j).unwrap().to_string(),
                    AbiValue::Int(generate_random_int(i as u64)),
                ));
            }
        }

        for i in 0..=u8::MAX {
            test_value_pool[2].push(RawValueWithAbiType::new(
                &AbiType::byte().to_string(),
                AbiValue::Byte(i),
            ));
        }

        for i in 0..2 {
            test_value_pool[3].push(RawValueWithAbiType::new(
                &AbiType::bool().to_string(),
                AbiValue::Bool(i == 0),
            ));
        }

        for _ in 0..500 {
            let temp_address_int = generate_random_int(256);
            let address_encode = temp_address_int.to_bytes_be_padded(32).unwrap();
            let address = Address(address_encode.try_into().unwrap());

            test_value_pool[4].push(RawValueWithAbiType::new(
                &AbiType::address().to_string(),
                AbiValue::Address(address),
            ));
        }

        for i in 1..100 {
            for _ in 0..5 {
                let gen_string: String = rng
                    .clone()
                    .sample_iter::<char, _>(rand::distributions::Standard)
                    .take(i)
                    .collect();

                test_value_pool[5].push(RawValueWithAbiType::new(
                    &AbiType::string().to_string(),
                    AbiValue::String(gen_string),
                ));
            }
        }

        generate_static_array(test_value_pool);
        generate_dynamic_array(test_value_pool);
        generate_tuple(7, test_value_pool);
        generate_tuple(8, test_value_pool);

        test_value_pool.to_owned()
    }

    #[test]
    fn test_random_elem_roundtrip() {
        let test_pool = setup();
        for v in &test_pool[8] {
            let abi_type = v.type_str.parse::<AbiType>().unwrap();
            let encoded = abi_type.encode(v.value.to_owned()).unwrap();
            assert_eq!(abi_type.decode(&encoded).unwrap(), v.value)
        }
    }
}
