use crate::step_defs::integration::world::World;
use algonaut_core::Address;
use cucumber::{then, when};
use data_encoding::BASE64;
use std::fs;

fn read_resource(name: &str) -> Vec<u8> {
    fs::read(format!("tests/features/resources/{name}"))
        .unwrap_or_else(|e| panic!("failed to read tests/features/resources/{name}: {e}"))
}

fn parse_status_from_error(err: &str) -> u16 {
    regex::Regex::new(r"status:\s*(\d+)")
        .unwrap()
        .captures(err)
        .and_then(|c| c.get(1))
        .and_then(|m| m.as_str().parse().ok())
        .unwrap_or(0)
}

#[when(regex = r#"^I compile a teal program "([^"]+)"$"#)]
async fn i_compile_a_teal_program(w: &mut World, program: String) {
    let algod = w.algod.as_ref().expect("algod not set");
    let source = read_resource(&program);

    match algod.teal_compile(&source, None).await {
        Ok(compiled) => {
            let hash: Address = compiled.hash().into();
            w.compile_status = Some(200);
            w.compile_result = Some(BASE64.encode(&compiled.0));
            w.compile_hash = Some(hash.to_string());
            w.compiled_program = Some(compiled.0);
        }
        Err(e) => {
            w.compile_status = Some(parse_status_from_error(&e.to_string()));
            w.compile_result = Some(String::new());
            w.compile_hash = Some(String::new());
            w.compiled_program = None;
        }
    }
}

#[then(regex = r#"^it is compiled with (\d+) and "([^"]*)" and "([^"]*)"$"#)]
async fn it_is_compiled_with(w: &mut World, status: u16, result: String, hash: String) {
    assert_eq!(w.compile_status, Some(status), "compile status mismatch");
    assert_eq!(w.compile_result.as_deref(), Some(result.as_str()));
    assert_eq!(w.compile_hash.as_deref(), Some(hash.as_str()));
}

#[then(regex = r#"^base64 decoding the response is the same as the binary "([^"]+)"$"#)]
async fn base64_decoding_response_matches_binary(w: &mut World, binary: String) {
    let compiled = w
        .compiled_program
        .as_ref()
        .expect("compiled program not set");
    let expected = read_resource(&binary);
    assert_eq!(compiled, &expected, "compiled bytes differ from {binary}");
}

#[then(regex = r#"^disassembly of "([^"]+)" matches "([^"]+)"$"#)]
async fn disassembly_matches(w: &mut World, bytecode_filename: String, source_filename: String) {
    let algod = w.algod.as_ref().expect("algod not set");
    let bytecode = read_resource(&bytecode_filename);
    let expected_source = String::from_utf8(read_resource(&source_filename))
        .expect("source filename is not valid UTF-8");
    let resp = algod
        .teal_disassemble(&bytecode)
        .await
        .expect("teal_disassemble failed");
    assert_eq!(resp.result, expected_source, "disassembly mismatch");
}

#[when(regex = r#"^I compile a teal program "([^"]+)" with mapping enabled$"#)]
async fn i_compile_a_teal_program_with_mapping_enabled(w: &mut World, program: String) {
    let algod = w.algod.as_ref().expect("algod not set");
    let source = read_resource(&program);
    let (_compiled, map) = algod
        .teal_compile_with_sourcemap(&source)
        .await
        .expect("teal_compile_with_sourcemap failed");
    w.compiled_sourcemap = Some(map);
}

#[then(regex = r#"^the resulting source map is the same as the json "([^"]+)"$"#)]
async fn the_resulting_source_map_is_the_same_as_the_json(w: &mut World, fixture: String) {
    let actual = w
        .compiled_sourcemap
        .as_ref()
        .expect("source map not produced");
    let expected_bytes = read_resource(&fixture);
    let expected_json =
        String::from_utf8(expected_bytes).expect("fixture sourcemap is not valid UTF-8");
    let expected = algonaut_abi::sourcemap::SourceMap::from_json(&expected_json)
        .expect("failed to parse fixture sourcemap");

    assert_eq!(
        actual.pc_line_string(),
        expected.pc_line_string(),
        "compiled source map differs from fixture"
    );
}
