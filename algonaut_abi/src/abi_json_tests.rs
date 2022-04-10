#[cfg(test)]
mod tests {
    use crate::abi_interactions::{
        AbiContract, AbiContractNetworkInfo, AbiInterface, AbiMethod, AbiMethodArg, AbiReturn,
    };

    #[test]
    fn test_encode_json_method() {
        let args = vec![
            AbiMethodArg {
                name: Some("0".to_owned()),
                type_: "uint32".to_owned(),
                description: None,
                parsed: None,
            },
            AbiMethodArg {
                name: Some("1".to_owned()),
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

        let expected = r#"{"name":"add","args":[{"name":"0","type":"uint32"},{"name":"1","type":"uint32"}],"returns":{"type":"uint32"}}"#;

        let json_res = serde_json::to_string(&method);
        assert!(json_res.is_ok());
        let json = json_res.unwrap();

        assert_eq!(expected, json);
    }

    #[test]
    fn test_encode_json_method_with_description() {
        let args = vec![
            AbiMethodArg {
                name: Some("0".to_owned()),
                type_: "uint32".to_owned(),
                description: Some("description".to_owned()),
                parsed: None,
            },
            AbiMethodArg {
                name: Some("1".to_owned()),
                type_: "uint32".to_owned(),
                description: Some("description".to_owned()),
                parsed: None,
            },
        ];
        let method = AbiMethod {
            name: "add".to_owned(),
            description: Some("description".to_owned()),
            args,
            returns: AbiReturn {
                type_: "uint32".to_owned(),
                description: Some("description".to_owned()),
                parsed: None,
            },
        };

        let expected = r#"{"name":"add","desc":"description","args":[{"name":"0","type":"uint32","desc":"description"},{"name":"1","type":"uint32","desc":"description"}],"returns":{"type":"uint32","desc":"description"}}"#;

        let json_res = serde_json::to_string(&method);
        assert!(json_res.is_ok());
        let json = json_res.unwrap();

        assert_eq!(expected, json);
    }

    #[test]
    fn test_encode_json_interface() {
        let args = vec![
            AbiMethodArg {
                name: Some("0".to_owned()),
                type_: "uint32".to_owned(),
                description: None,
                parsed: None,
            },
            AbiMethodArg {
                name: Some("1".to_owned()),
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

        let interface_object = AbiInterface {
            name: "interface".to_owned(),
            description: None,
            methods: vec![method],
        };

        let expected = r#"{"name":"interface","methods":[{"name":"add","args":[{"name":"0","type":"uint32"},{"name":"1","type":"uint32"}],"returns":{"type":"uint32"}}]}"#;

        let json_res = serde_json::to_string(&interface_object);
        assert!(json_res.is_ok());
        let json = json_res.unwrap();

        assert_eq!(expected, json);
    }

    #[test]
    fn test_encode_json_interface_with_description() {
        let args = vec![
            AbiMethodArg {
                name: Some("0".to_owned()),
                type_: "uint32".to_owned(),
                description: Some("description".to_owned()),
                parsed: None,
            },
            AbiMethodArg {
                name: Some("1".to_owned()),
                type_: "uint32".to_owned(),
                description: Some("description".to_owned()),
                parsed: None,
            },
        ];
        let method = AbiMethod {
            name: "add".to_owned(),
            description: Some("description".to_owned()),
            args,
            returns: AbiReturn {
                type_: "uint32".to_owned(),
                description: Some("description".to_owned()),
                parsed: None,
            },
        };

        let interface_object = AbiInterface {
            name: "interface".to_owned(),
            description: None,
            methods: vec![method],
        };

        let expected = r#"{"name":"interface","methods":[{"name":"add","desc":"description","args":[{"name":"0","type":"uint32","desc":"description"},{"name":"1","type":"uint32","desc":"description"}],"returns":{"type":"uint32","desc":"description"}}]}"#;

        let json_res = serde_json::to_string(&interface_object);
        assert!(json_res.is_ok());
        let json = json_res.unwrap();

        assert_eq!(expected, json);
    }

    #[test]
    fn test_encode_json_contract() {
        let args = vec![
            AbiMethodArg {
                name: Some("0".to_owned()),
                type_: "uint32".to_owned(),
                description: None,
                parsed: None,
            },
            AbiMethodArg {
                name: Some("1".to_owned()),
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

        let network = AbiContractNetworkInfo { app_id: 123 };

        let contract = AbiContract {
            name: "contract".to_owned(),
            networks: [("genesis hash".to_owned(), network)].into(),
            description: None,
            methods: vec![method],
        };

        let expected = r#"{"name":"contract","networks":{"genesis hash":{"appID":123}},"methods":[{"name":"add","args":[{"name":"0","type":"uint32"},{"name":"1","type":"uint32"}],"returns":{"type":"uint32"}}]}"#;

        let json_res = serde_json::to_string(&contract);
        assert!(json_res.is_ok());
        let json = json_res.unwrap();

        assert_eq!(expected, json);
    }

    #[test]
    fn test_encode_json_contract_with_description() {
        let args = vec![
            AbiMethodArg {
                name: Some("0".to_owned()),
                type_: "uint32".to_owned(),
                description: Some("description".to_owned()),
                parsed: None,
            },
            AbiMethodArg {
                name: Some("1".to_owned()),
                type_: "uint32".to_owned(),
                description: Some("description".to_owned()),
                parsed: None,
            },
        ];
        let method = AbiMethod {
            name: "add".to_owned(),
            description: Some("description".to_owned()),
            args,
            returns: AbiReturn {
                type_: "uint32".to_owned(),
                description: Some("description".to_owned()),
                parsed: None,
            },
        };

        let network = AbiContractNetworkInfo { app_id: 123 };

        let contract = AbiContract {
            name: "contract".to_owned(),
            networks: [("genesis hash".to_owned(), network)].into(),
            description: Some("description for contract".to_owned()),
            methods: vec![method],
        };

        let expected = r#"{"name":"contract","desc":"description for contract","networks":{"genesis hash":{"appID":123}},"methods":[{"name":"add","desc":"description","args":[{"name":"0","type":"uint32","desc":"description"},{"name":"1","type":"uint32","desc":"description"}],"returns":{"type":"uint32","desc":"description"}}]}"#;

        let json_res = serde_json::to_string(&contract);
        assert!(json_res.is_ok());
        let json = json_res.unwrap();

        assert_eq!(expected, json);
    }

    #[test]
    fn test_decode_interface() {
        let json = r#"{"name": "Calculator","desc":"example description","methods": [{ "name": "add", "args": [ { "name": "a", "type": "uint64", "desc": "..." },{ "name": "b", "type": "uint64", "desc": "..." } ], "returns":{"type":"void"}},{ "name": "multiply", "args": [ { "name": "a", "type": "uint64", "desc": "..." },{ "name": "b", "type": "uint64", "desc": "..." } ], "returns":{"type":"void"}}]}"#;
        let contract: AbiContract = serde_json::from_str(json).unwrap();

        let methods = vec![
            AbiMethod {
                name: "add".to_owned(),
                description: None,
                args: vec![
                    AbiMethodArg {
                        name: Some("a".to_owned()),
                        type_: "uint64".to_owned(),
                        description: Some("...".to_owned()),
                        parsed: None,
                    },
                    AbiMethodArg {
                        name: Some("b".to_owned()),
                        type_: "uint64".to_owned(),
                        description: Some("...".to_owned()),
                        parsed: None,
                    },
                ],
                returns: AbiReturn {
                    type_: "void".to_owned(),
                    description: None,
                    parsed: None,
                },
            },
            AbiMethod {
                name: "multiply".to_owned(),
                description: None,
                args: vec![
                    AbiMethodArg {
                        name: Some("a".to_owned()),
                        type_: "uint64".to_owned(),
                        description: Some("...".to_owned()),
                        parsed: None,
                    },
                    AbiMethodArg {
                        name: Some("b".to_owned()),
                        type_: "uint64".to_owned(),
                        description: Some("...".to_owned()),
                        parsed: None,
                    },
                ],
                returns: AbiReturn {
                    type_: "void".to_owned(),
                    description: None,
                    parsed: None,
                },
            },
        ];

        assert_eq!(
            contract,
            AbiContract {
                name: "Calculator".to_owned(),
                description: Some("example description".to_owned()),
                networks: [].into(),
                methods
            }
        )
    }

    #[test]
    fn test_decode_contract() {
        let json = r#"{"name": "Calculator", "desc":"example description", "networks": {"wGHE2Pwdvd7S12BL5FaOP20EGYesN73ktiC1qzkkit8=": {"appID": 10}}, "methods": [{ "name": "add", "args": [ { "name": "a", "type": "uint64", "desc": "..." },{ "name": "b", "type": "uint64", "desc": "..." } ], "returns":{"type":"void"}},{ "name": "multiply", "args": [ { "name": "a", "type": "uint64", "desc": "..." },{ "name": "b", "type": "uint64", "desc": "..." } ], "returns":{"type":"void"}}]}"#;
        let contract: AbiContract = serde_json::from_str(json).unwrap();

        let methods = vec![
            AbiMethod {
                name: "add".to_owned(),
                description: None,
                args: vec![
                    AbiMethodArg {
                        name: Some("a".to_owned()),
                        type_: "uint64".to_owned(),
                        description: Some("...".to_owned()),
                        parsed: None,
                    },
                    AbiMethodArg {
                        name: Some("b".to_owned()),
                        type_: "uint64".to_owned(),
                        description: Some("...".to_owned()),
                        parsed: None,
                    },
                ],
                returns: AbiReturn {
                    type_: "void".to_owned(),
                    description: None,
                    parsed: None,
                },
            },
            AbiMethod {
                name: "multiply".to_owned(),
                description: None,
                args: vec![
                    AbiMethodArg {
                        name: Some("a".to_owned()),
                        type_: "uint64".to_owned(),
                        description: Some("...".to_owned()),
                        parsed: None,
                    },
                    AbiMethodArg {
                        name: Some("b".to_owned()),
                        type_: "uint64".to_owned(),
                        description: Some("...".to_owned()),
                        parsed: None,
                    },
                ],
                returns: AbiReturn {
                    type_: "void".to_owned(),
                    description: None,
                    parsed: None,
                },
            },
        ];

        assert_eq!(
            contract,
            AbiContract {
                name: "Calculator".to_owned(),
                description: Some("example description".to_owned()),
                networks: [(
                    "wGHE2Pwdvd7S12BL5FaOP20EGYesN73ktiC1qzkkit8=".to_owned(),
                    AbiContractNetworkInfo { app_id: 10 }
                )]
                .into(),
                methods
            }
        )
    }
}
