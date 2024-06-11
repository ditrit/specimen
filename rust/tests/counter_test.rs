#[test]
fn test_counter() {
    let mut counter = 0;

    specimen::run(
        &mut |tile: &specimen::Dict| -> Result<(), Box<str>> {
            match tile.get("expected_count") {
                Some(expected_count) => {
                    let expected_count = expected_count.parse().unwrap();
                    if counter != expected_count {
                        return Err(Box::from(format!(
                            "Counter ({counter}) did not match expected count ({expected_count})",
                        )));
                    }
                }
                None => {}
            };
            counter += 1;
            Ok(())
        },
        &[specimen::file::File::read_local_file(
            "../test/counter/counter_data.yaml",
        )],
    )
}
