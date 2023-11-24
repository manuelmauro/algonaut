#[cfg(test)]
mod test {
    use rand::Rng;

    struct SetupRes {
        type_testpool: Vec<AbiType>,
        tuple_testpool: Vec<AbiType>,
    }
    use crate::{abi_encode::find_bool_lr, abi_type::AbiType};

    fn generate_random_tuple_type(
        type_testpool: &mut [AbiType],
        tuple_testpool: &mut [AbiType],
    ) -> AbiType {
        let mut rng = rand::thread_rng();
        let tuple_len: usize = rng.gen_range(0..20);

        let mut tuple_elems = vec![];

        for _ in 0..tuple_len {
            let base_or_tuple: i32 = rng.gen_range(0..5);
            if base_or_tuple == 1 && tuple_testpool.len() > 0 {
                tuple_elems.push(tuple_testpool[rng.gen_range(0..tuple_testpool.len())].clone());
            } else {
                tuple_elems.push(type_testpool[rng.gen_range(0..type_testpool.len())].clone());
            }
        }
        AbiType::tuple(tuple_elems).unwrap()
    }

    fn setup() -> SetupRes {
        let mut type_testpool = vec![
            AbiType::bool(),
            AbiType::address(),
            AbiType::string(),
            AbiType::byte(),
        ];

        for i in (8..512).step_by(8) {
            type_testpool.push(AbiType::uint(i).unwrap());
        }

        for i in (8..512).step_by(8) {
            for j in 1..=160 {
                type_testpool.push(AbiType::ufixed(i, j).unwrap());
            }
        }

        for i in 0..type_testpool.len() {
            type_testpool.push(AbiType::dynamic_array(type_testpool[i].clone()));
            type_testpool.push(AbiType::static_array(type_testpool[i].clone(), 10));
            type_testpool.push(AbiType::static_array(type_testpool[i].clone(), 20));
        }

        let mut tuple_testpool = vec![];

        for _ in 0..100 {
            let temp_tuple = generate_random_tuple_type(&mut type_testpool, &mut tuple_testpool);
            tuple_testpool.push(temp_tuple);
        }

        SetupRes {
            type_testpool,
            tuple_testpool,
        }
    }

    #[test]
    fn test_uint_valid() {
        for i in (8..512).step_by(8) {
            let type_ = AbiType::uint(i).unwrap();
            assert_eq!(type_.to_string(), format!("uint{}", i))
        }
    }

    #[test]
    fn test_uint_invalid() {
        let mut rng = rand::thread_rng();

        for _ in 0..1000 {
            let mut size_rand = rng.gen_range(0..65536);

            while size_rand % 8 == 0 && size_rand <= 512 && size_rand >= 8 {
                size_rand = rng.gen_range(0..1000);
            }

            let final_size_rand = size_rand;
            assert!(AbiType::uint(final_size_rand).is_err())
        }
    }

    #[test]
    fn test_ufixed_valid() {
        for i in (8..512).step_by(8) {
            for j in 1..160 {
                let type_ = AbiType::ufixed(i, j).unwrap();
                assert_eq!(type_.to_string(), format!("ufixed{}x{}", i, j))
            }
        }
    }

    #[test]
    fn test_ufixed_invalid() {
        let mut rng = rand::thread_rng();

        for _ in 0..1000 {
            let mut size_rand = rng.gen_range(0..65536);

            while size_rand % 8 == 0 && size_rand <= 512 && size_rand >= 8 {
                size_rand = rng.gen_range(0..65536);
            }

            let mut precision_rand = rng.gen_range(0..1024);
            while precision_rand >= 1 && precision_rand <= 160 {
                precision_rand = rng.gen_range(0..1024);
            }

            let final_rand_precision = precision_rand;
            let final_size_rand = size_rand;
            assert!(AbiType::ufixed(final_size_rand, final_rand_precision).is_err())
        }
    }

    #[test]
    fn test_simple_types_valid() {
        assert_eq!(&AbiType::byte().to_string(), "byte");
        assert_eq!(&AbiType::string().to_string(), "string");
        assert_eq!(&AbiType::address().to_string(), "address");
        assert_eq!(&AbiType::bool().to_string(), "bool");
    }

    #[test]
    fn test_type_to_string_valid() {
        assert_eq!(
            &AbiType::dynamic_array(AbiType::uint(32).unwrap()).to_string(),
            "uint32[]"
        );
        assert_eq!(
            &AbiType::dynamic_array(AbiType::dynamic_array(AbiType::byte())).to_string(),
            "byte[][]"
        );
        assert_eq!(
            &AbiType::static_array(AbiType::ufixed(128, 10).unwrap(), 100).to_string(),
            "ufixed128x10[100]"
        );
        assert_eq!(
            &AbiType::static_array(AbiType::static_array(AbiType::bool(), 128), 256).to_string(),
            "bool[128][256]"
        );
        assert_eq!(
            &AbiType::tuple(vec![
                AbiType::uint(32).unwrap(),
                AbiType::tuple(vec![
                    AbiType::address(),
                    AbiType::byte(),
                    AbiType::static_array(AbiType::bool(), 10),
                    AbiType::dynamic_array(AbiType::ufixed(256, 10).unwrap()),
                ])
                .unwrap()
            ])
            .unwrap()
            .to_string(),
            "(uint32,(address,byte,bool[10],ufixed256x10[]))"
        );
        assert_eq!(&AbiType::tuple(vec![]).unwrap().to_string(), "()");
    }

    #[test]
    fn test_uint_from_string_valid() {
        for i in (8..512).step_by(8) {
            let encoded = format!("uint{}", i);
            let uint_type = AbiType::uint(i).unwrap();
            assert_eq!(encoded.parse::<AbiType>().unwrap(), uint_type)
        }
    }

    #[test]
    fn test_uint_from_string_invalid() {
        let mut rng = rand::thread_rng();

        for _ in 0..100 {
            let mut size_rand = rng.gen_range(0..65536);

            while size_rand % 8 == 0 && size_rand <= 512 && size_rand >= 8 {
                size_rand = rng.gen_range(0..65536);
            }

            let encoded = format!("uint{}", size_rand);
            assert!(encoded.parse::<AbiType>().is_err());
        }
    }

    #[test]
    fn test_ufixed_from_string_valid() {
        for i in (8..512).step_by(8) {
            for j in 1..160 {
                let encoded = format!("ufixed{}x{}", i, j);
                let ufixed_t = AbiType::ufixed(i, j).unwrap();
                assert_eq!(encoded.parse::<AbiType>().unwrap(), ufixed_t);
            }
        }
    }

    #[test]
    fn test_ufixed_from_string_invalid() {
        let mut rng = rand::thread_rng();

        for _ in 0..1000 {
            let mut size_rand = rng.gen_range(0..65536);

            while size_rand % 8 == 0 && size_rand <= 512 && size_rand >= 8 {
                size_rand = rng.gen_range(0..65536);
            }

            let mut precision_rand = rng.gen_range(0..1024);
            while precision_rand >= 1 && precision_rand <= 160 {
                precision_rand = rng.gen_range(0..1024);
            }

            let encoded = format!("ufixed{}x{}", size_rand, precision_rand);
            assert!(encoded.parse::<AbiType>().is_err());
        }
    }

    #[test]
    fn test_simple_type_from_string_valid() {
        assert_eq!("address".parse::<AbiType>().unwrap(), AbiType::address());
        assert_eq!("byte".parse::<AbiType>().unwrap(), AbiType::byte());
        assert_eq!("bool".parse::<AbiType>().unwrap(), AbiType::bool());
        assert_eq!("string".parse::<AbiType>().unwrap(), AbiType::string());
    }

    #[test]
    fn test_type_from_string_valid() {
        assert_eq!(
            "uint256[]".parse::<AbiType>().unwrap(),
            AbiType::dynamic_array(AbiType::uint(256).unwrap())
        );
        assert_eq!(
            "ufixed256x64[]".parse::<AbiType>().unwrap(),
            AbiType::dynamic_array(AbiType::ufixed(256, 64).unwrap())
        );
        assert_eq!(
            "byte[][][][]".parse::<AbiType>().unwrap(),
            AbiType::dynamic_array(AbiType::dynamic_array(AbiType::dynamic_array(
                AbiType::dynamic_array(AbiType::byte())
            )))
        );
        assert_eq!(
            "address[100]".parse::<AbiType>().unwrap(),
            AbiType::static_array(AbiType::address(), 100)
        );
        assert_eq!(
            "uint64[][100]".parse::<AbiType>().unwrap(),
            AbiType::static_array(AbiType::dynamic_array(AbiType::uint(64).unwrap()), 100)
        );
        assert_eq!(
            "()".parse::<AbiType>().unwrap(),
            AbiType::tuple(vec![]).unwrap()
        );
        assert_eq!(
            "(uint32,(address,byte,bool[10],ufixed256x10[]),byte[])"
                .parse::<AbiType>()
                .unwrap(),
            AbiType::tuple(vec![
                AbiType::uint(32).unwrap(),
                AbiType::tuple(vec![
                    AbiType::address(),
                    AbiType::byte(),
                    AbiType::static_array(AbiType::bool(), 10),
                    AbiType::dynamic_array(AbiType::ufixed(256, 10).unwrap())
                ])
                .unwrap(),
                AbiType::dynamic_array(AbiType::byte())
            ])
            .unwrap()
        );
        assert_eq!(
            "(uint32,(address,byte,bool[10],(ufixed256x10[])))"
                .parse::<AbiType>()
                .unwrap(),
            AbiType::tuple(vec![
                AbiType::uint(32).unwrap(),
                AbiType::tuple(vec![
                    AbiType::address(),
                    AbiType::byte(),
                    AbiType::static_array(AbiType::bool(), 10),
                    AbiType::tuple(vec![AbiType::dynamic_array(
                        AbiType::ufixed(256, 10).unwrap()
                    )])
                    .unwrap()
                ])
                .unwrap(),
            ])
            .unwrap()
        );
        assert_eq!(
            "((uint32),(address,(byte,bool[10],ufixed256x10[])))"
                .parse::<AbiType>()
                .unwrap(),
            AbiType::tuple(vec![
                AbiType::tuple(vec![AbiType::uint(32).unwrap()]).unwrap(),
                AbiType::tuple(vec![
                    AbiType::address(),
                    AbiType::tuple(vec![
                        AbiType::byte(),
                        AbiType::static_array(AbiType::bool(), 10),
                        AbiType::dynamic_array(AbiType::ufixed(256, 10).unwrap())
                    ])
                    .unwrap()
                ])
                .unwrap(),
            ])
            .unwrap()
        )
    }

    #[test]
    fn test_type_from_string_is_invalid() {
        let test_cases = vec![
            // uint
            "uint123x345",
            "uint 128",
            "uint8 ",
            "uint!8",
            "uint[32]",
            "uint-893",
            "uint#120\\",
            // ufixed
            "ufixed000000000016x0000010",
            "ufixed123x345",
            "ufixed 128 x 100",
            "ufixed64x10 ",
            "ufixed!8x2 ",
            "ufixed[32]x16",
            "ufixed-64x+100",
            "ufixed16x+12",
            // dynamic array
            "uint256 []",
            "byte[] ",
            "[][][]",
            "stuff[]",
            // static array
            "ufixed32x10[0]",
            "byte[10 ]",
            "uint64[0x21]",
            // tuple
            "(ufixed128x10))",
            "(,uint128,byte[])",
            "(address,ufixed64x5,)",
            "(byte[16],somethingwrong)",
            "(                )",
            "((uint32)",
            "(byte,,byte)",
            "((byte),,(byte))",
            // some random stuffs
            "",
        ];

        for test_case in test_cases {
            assert!(test_case.parse::<AbiType>().is_err());
        }
    }

    #[test]
    fn test_tuple_roundtrip() {
        let tuple_testpool = setup().tuple_testpool;
        for t in tuple_testpool {
            let encoded = t.to_string();
            let decoded = encoded.parse::<AbiType>().unwrap();
            assert_eq!(decoded, t.clone());
        }
    }

    #[test]
    fn test_self_equiv() {
        let mut rng = rand::thread_rng();

        let SetupRes {
            type_testpool,
            tuple_testpool,
        } = setup();

        for t in &type_testpool {
            assert_eq!(t, t);
        }

        for t in &tuple_testpool {
            assert_eq!(t, t);
        }

        for _ in 0..1000 {
            let index0 = rng.gen_range(0..type_testpool.len());
            let mut index1 = rng.gen_range(0..type_testpool.len());

            while type_testpool[index0].to_string() == type_testpool[index1].to_string() {
                index1 = rng.gen_range(0..type_testpool.len());
            }

            assert_ne!(type_testpool[index0], type_testpool[index1]);
        }

        for _ in 0..1000 {
            let index0 = rng.gen_range(0..tuple_testpool.len());
            let mut index1 = rng.gen_range(0..tuple_testpool.len());

            while tuple_testpool[index0].to_string() == tuple_testpool[index1].to_string() {
                index1 = rng.gen_range(0..tuple_testpool.len());
            }

            assert_ne!(tuple_testpool[index0], tuple_testpool[index1]);
        }
    }

    #[test]
    fn test_is_dynamic() {
        let SetupRes {
            type_testpool,
            tuple_testpool,
        } = setup();

        for t in &type_testpool {
            let encoded = t.to_string();
            let infer_from_string = encoded.contains("[]") || encoded.contains("string");
            assert_eq!(infer_from_string, t.is_dynamic());
        }

        for t in &tuple_testpool {
            let encoded = t.to_string();
            let infer_from_string = encoded.contains("[]") || encoded.contains("string");
            assert_eq!(infer_from_string, t.is_dynamic());
        }
    }

    #[test]
    fn test_byte_len() {
        let SetupRes {
            type_testpool,
            tuple_testpool,
        } = setup();

        assert_eq!(AbiType::address().byte_len().unwrap(), 32);
        assert_eq!(AbiType::byte().byte_len().unwrap(), 1);

        for t in type_testpool {
            if t.is_dynamic() {
                assert!(t.byte_len().is_err())
            }
        }

        for t in tuple_testpool {
            if t.is_dynamic() {
                assert!(t.byte_len().is_err())
            } else {
                let mut size = 0;
                let ct_list = t.children();

                for i in 0..ct_list.len() {
                    match ct_list[i] {
                        AbiType::Bool => {
                            let bool_num = find_bool_lr(&ct_list, i, 1).unwrap() + 1;
                            size += bool_num / 8;
                            size += if bool_num % 8 != 0 { 1 } else { 0 };
                        }
                        _ => {
                            size += ct_list[i].byte_len().unwrap();
                        }
                    }
                }
                assert_eq!(size, t.byte_len().unwrap());
            }
        }
    }
}
