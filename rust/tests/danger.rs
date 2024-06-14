#[test]
fn test_danger() {
    specimen::run(
        &mut |tile: &specimen::Dict| -> Result<(), Box<str>> {
            panic!("This code should never be run")
        },
        &[specimen::file::File::read_local_file(
            "../test/danger/danger_data.yaml",
        )],
    )
}
