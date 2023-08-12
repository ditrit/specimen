#[derive(Debug, Default)]
struct Runner {
    counter: u32,
}

impl specimen::Evaluator for Runner {
    fn evaluate(&mut self, tile: &specimen::Dict) -> bool {
        let result = match tile.get("expected_count") {
            Some(expected_count) => {
                let expected_count = expected_count.parse().unwrap();
                self.counter == expected_count
            }
            None => true,
        };
        self.counter += 1;
        result
    }
}

#[test]
fn test_counter() {
    specimen::run(
        Box::new(Runner::default()),
        &[specimen::file::File::read_local_file(
            "../test/counter/counter_data.yaml",
        )],
    )
}
