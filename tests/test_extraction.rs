use realmpipe::extractor::Extractor;
use std::fs::write;
use tempfile::tempdir;

const CLIENT_SWF: &[u8] = include_bytes!("AssembleeGameClient1554116567.swf");

#[test]
fn test_extraction() {
    simple_logger::init().expect("error initializing logger");

    // create a directory to work from
    let dir = tempdir().expect("error creating temp dir");

    // extract client swf
    let swf = dir.path().join("client.swf");
    write(&swf, CLIENT_SWF).expect("error extracting client SWF");

    // extract rabcdasm binaries
    let extractor = Extractor::unpack().expect("error extracting binaries");

    // extract game mappings
    let mappings = extractor
        .extract_mappings(&swf, true)
        .expect("error extracting mappings");
}
