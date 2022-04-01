#[cfg(test)]
mod tests {
    use crate::{
        abi_interactions::{AbiMethod, AbiMethodArg, AbiReturn},
        abi_type::AbiType,
    };

    #[test]
    fn test_method_from_signature() {
        let uint32_abi_type_res = "uint32".parse::<AbiType>();
        assert!(uint32_abi_type_res.is_ok());
        let uint32_abi_type = uint32_abi_type_res.clone().unwrap();

        let expected_args = vec![
            AbiMethodArg {
                name: None,
                type_: "uint32".to_owned(),
                description: None,
                parsed: Some(uint32_abi_type.clone()),
            },
            AbiMethodArg {
                name: None,
                type_: "uint32".to_owned(),
                description: None,
                parsed: Some(uint32_abi_type.clone()),
            },
        ];
        let expected = AbiMethod {
            name: "add".to_owned(),
            description: None,
            args: expected_args,
            returns: AbiReturn {
                type_: "uint32".to_owned(),
                description: None,
                parsed: Some(uint32_abi_type),
            },
        };

        let method_sig = "add(uint32,uint32)uint32";

        let result = AbiMethod::from_signature(method_sig);
        assert!(result.is_ok());
        let method = result.unwrap();

        assert_eq!(expected, method);
    }

    #[test]
    fn test_method_from_signature_with_tuple() {
        let uint32_abi_type_res = "uint32".parse::<AbiType>();
        assert!(uint32_abi_type_res.is_ok());
        let uint32_abi_type = uint32_abi_type_res.clone().unwrap();

        let uint32_tuple_abi_type_res = "(uint32,uint32)".parse::<AbiType>();
        assert!(uint32_tuple_abi_type_res.is_ok());
        let uint32_tuple_abi_type = uint32_tuple_abi_type_res.clone().unwrap();

        let uint32_tuple_tuple_abi_type_res = "(uint32,(uint32,uint32))".parse::<AbiType>();
        assert!(uint32_tuple_tuple_abi_type_res.is_ok());
        let uint32_tuple_tuple_abi_type = uint32_tuple_tuple_abi_type_res.clone().unwrap();

        let expected_args = vec![
            AbiMethodArg {
                name: None,
                type_: "(uint32,(uint32,uint32))".to_owned(),
                description: None,
                parsed: Some(uint32_tuple_tuple_abi_type.clone()),
            },
            AbiMethodArg {
                name: None,
                type_: "uint32".to_owned(),
                description: None,
                parsed: Some(uint32_abi_type.clone()),
            },
        ];
        let expected = AbiMethod {
            name: "add".to_owned(),
            description: None,
            args: expected_args,
            returns: AbiReturn {
                type_: "(uint32,uint32)".to_owned(),
                description: None,
                parsed: Some(uint32_tuple_abi_type),
            },
        };

        let method_sig = "add((uint32,(uint32,uint32)),uint32)(uint32,uint32)";

        let result = AbiMethod::from_signature(method_sig);
        assert!(result.is_ok());
        let method = result.unwrap();

        assert_eq!(expected, method);
    }

    #[test]
    fn test_method_from_signature_with_void_return() {
        let uint32_abi_type_res = "uint32".parse::<AbiType>();
        assert!(uint32_abi_type_res.is_ok());
        let uint32_abi_type = uint32_abi_type_res.clone().unwrap();

        let expected_args = vec![
            AbiMethodArg {
                name: None,
                type_: "uint32".to_owned(),
                description: None,
                parsed: Some(uint32_abi_type.clone()),
            },
            AbiMethodArg {
                name: None,
                type_: "uint32".to_owned(),
                description: None,
                parsed: Some(uint32_abi_type.clone()),
            },
        ];
        let expected = AbiMethod {
            name: "add".to_owned(),
            description: None,
            args: expected_args,
            returns: AbiReturn {
                type_: "void".to_owned(),
                description: None,
                parsed: None,
            },
        };

        let method_sig = "add(uint32,uint32)void";

        let result = AbiMethod::from_signature(method_sig);
        assert!(result.is_ok());
        let method = result.unwrap();

        assert_eq!(expected, method);
    }

    #[test]
    fn test_method_from_signature_with_no_args() {
        let expected_args = vec![];
        let expected = AbiMethod {
            name: "add".to_owned(),
            description: None,
            args: expected_args,
            returns: AbiReturn {
                type_: "void".to_owned(),
                description: None,
                parsed: None,
            },
        };

        let method_sig = "add()void";

        let result = AbiMethod::from_signature(method_sig);
        assert!(result.is_ok());
        let method = result.unwrap();

        assert_eq!(expected, method);
    }

    #[test]
    fn test_method_from_signature_invalid_format() {
        let method_sig = "add)uint32,uint32)uint32";
        let result = AbiMethod::from_signature(method_sig);
        assert!(result.is_err());

        let method_sig = "add(uint32, uint32)uint32";
        let result = AbiMethod::from_signature(method_sig);
        assert!(result.is_err());

        let method_sig = "(uint32,uint32)uint32";
        let result = AbiMethod::from_signature(method_sig);
        assert!(result.is_err());

        let method_sig = "add((uint32, uint32)uint32";
        let result = AbiMethod::from_signature(method_sig);
        assert!(result.is_err());
    }

    #[test]
    fn test_method_from_signature_invalid_abi_type() {
        let method_sig = "add(uint32,uint32)int32";
        let result = AbiMethod::from_signature(method_sig);
        assert!(result.is_err());
    }

    #[test]
    fn test_get_signature() {
        let args = vec![
            AbiMethodArg {
                name: None,
                type_: "uint32".to_owned(),
                description: None,
                parsed: None,
            },
            AbiMethodArg {
                name: None,
                type_: "uint32".to_owned(),
                description: None,
                parsed: None,
            },
        ];
        let method = AbiMethod {
            name: "add".to_owned(),
            description: None,
            args,
            returns: AbiReturn {
                type_: "uint32".to_owned(),
                description: None,
                parsed: None,
            },
        };

        let expected = "add(uint32,uint32)uint32";

        assert_eq!(expected, method.get_signature());
    }

    #[test]
    fn test_get_selector() {
        let args = vec![
            AbiMethodArg {
                name: None,
                type_: "uint32".to_owned(),
                description: None,
                parsed: None,
            },
            AbiMethodArg {
                name: None,
                type_: "uint32".to_owned(),
                description: None,
                parsed: None,
            },
        ];
        let method = AbiMethod {
            name: "add".to_owned(),
            description: None,
            args,
            returns: AbiReturn {
                type_: "uint32".to_owned(),
                description: None,
                parsed: None,
            },
        };

        let expected = [0x3e, 0x1e, 0x52, 0xbd];

        let selector_res = method.get_selector();
        assert!(selector_res.is_ok());
        let selector = selector_res.unwrap();

        assert_eq!(expected, selector);
    }
}

#[cfg(test)]
mod tests_cases {
    use crate::abi_interactions::{AbiMethod, AbiMethodArg};
    use std::collections::HashMap;

    struct MethodTestCase {
        signature_str: String,
        tx_call_count: usize,
        args: Vec<AbiMethodArg>,
    }

    impl MethodTestCase {
        fn new(method_str: &str, tx_call_count: usize, args: Vec<&str>) -> MethodTestCase {
            MethodTestCase {
                signature_str: method_str.to_owned(),
                tx_call_count,
                args: args
                    .into_iter()
                    .map(|type_| AbiMethodArg {
                        name: None,
                        type_: type_.to_owned(),
                        description: None,
                        parsed: None,
                    })
                    .collect(),
            }
        }
    }

    fn test_cases() -> Vec<MethodTestCase> {
        vec![
            MethodTestCase::new(
                "someMethod(uint64,ufixed64x2,(bool,byte),address)void",
                1,
                vec!["uint64", "ufixed64x2", "(bool,byte)", "address"],
            ),
            MethodTestCase::new("pseudoRandomGenerator()uint256", 1, vec![]),
            MethodTestCase::new("add(uint64,uint64)uint128", 1, vec!["uint64", "uint64"]),
            MethodTestCase::new(
                "someEffectOnTheOtherSide___(uint64,(ufixed256x10,bool))void",
                1,
                vec!["uint64", "(ufixed256x10,bool)"],
            ),
            MethodTestCase::new(
                "someMethod(uint64,ufixed64x2,(bool,byte),address)void",
                1,
                vec!["uint64", "ufixed64x2", "(bool,byte)", "address"],
            ),
            MethodTestCase::new("returnATuple(address)(byte[32],bool)", 1, vec!["address"]),
            MethodTestCase::new(
                "txcalls(pay,pay,axfer,byte)bool",
                4,
                vec!["pay", "pay", "axfer", "byte"],
            ),
            MethodTestCase::new(
                "foreigns(account,pay,asset,application,bool)void",
                2,
                vec!["account", "pay", "asset", "application", "bool"],
            ),
        ]
    }

    #[test]
    fn test_method_from_signature() {
        for test_case in test_cases() {
            let method = AbiMethod::from_signature(&test_case.signature_str).unwrap();
            assert_eq!(method.get_signature(), test_case.signature_str);
            assert_eq!(method.get_tx_count(), test_case.tx_call_count);
            assert_eq!(method.args, test_case.args);
        }
    }

    #[test]
    fn test_method_from_signature_invalid() {
        let failing_test_cases = vec![
            "___nopeThis Not Right nAmE () void",
            "intentional(MessAroundWith(Parentheses(address)(uint8)",
        ];
        for test_case in failing_test_cases {
            let method_res = AbiMethod::from_signature(test_case);
            assert!(method_res.is_err())
        }
    }

    #[test]
    fn test_method_get_selector() {
        let method_selector_map: HashMap<&str, Vec<u8>> = [
            ("add(uint32,uint32)uint32", vec![0x3e, 0x1e, 0x52, 0xbd]),
            ("add(uint64,uint64)uint128", vec![0x8a, 0xa3, 0xb6, 0x1f]),
        ]
        .into();
        for (key, value) in method_selector_map {
            let method = AbiMethod::from_signature(key).unwrap();
            assert_eq!(method.get_selector().unwrap().to_vec(), value);
        }
    }

    #[test]
    fn test_method_json_roundtrip() {
        for test_case in test_cases() {
            let method = AbiMethod::from_signature(&test_case.signature_str).unwrap();
            let bytes = serde_json::to_vec(&method).unwrap();
            let read_method: AbiMethod = serde_json::from_slice(&bytes).unwrap();
            assert_eq!(read_method, method);
        }
    }
}
