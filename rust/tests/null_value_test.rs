#[derive(Debug, Default)]
struct Runner {}

impl specimen::Evaluator for Runner {
    fn evaluate(&mut self, tile: &specimen::Dict) -> bool {
        match tile.get("nullValue") {
            Some(_) => true,
            _ => false,
        }
    }
}

#[test]
fn test_null_value() {
    specimen::run(
        Box::new(Runner::default()),
        &[specimen::file::File::read_local_file(
            "../test/counter/counter_data.yaml",
        )],
    )
}
