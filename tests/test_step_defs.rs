mod step_defs;

use step_defs::util::AppArg;

#[test]
fn app_args() {
    assert_eq!(AppArg::Int(0), "0".parse().unwrap());
    assert_eq!(AppArg::Str("name".to_owned()), "str:name".parse().unwrap());
    assert_eq!(AppArg::B64("aaa".to_owned()), "b64:aaa".parse().unwrap());
    assert_eq!(AppArg::Addr("AAA".to_owned()), "addr:AAA".parse().unwrap());
}
