#[derive(Debug, Default)]
struct Zoo(Vec<String>);

impl Zoo {
    // Zoo implementation
    fn add_animal(&mut self, animal: Box<str>) -> Box<str> {
        self.0.push(animal.into());
        Box::from(self.0.join(" "))
    }

    // Test util functions
    fn zoo(&mut self, tile: specimen::Dict) -> bool {
        let animal = &tile["animal"];
        let expected = &tile["expected_result"];

        if animal.len() > 0 {
            let output = self.add_animal(animal.to_owned());

            if expected.len() > 0 {
                return output == *expected;
            }
        }
        return true;
    }

    fn animal_kind(&mut self, tile: specimen::Dict) -> bool {
        let name = &tile["name"];
        let horn: i32 = tile["horn"].parse().unwrap();
        let leg: i32 = tile["leg"].parse().unwrap();
        if *name == Box::from("deer") {
            horn == 2 && leg == 4
        } else if *name == Box::from("earthpony") {
            horn == 0 && leg == 4
        } else {
            false
        }
    }
}

#[derive(Debug, Default)]
struct Runner {
    zoo: Zoo,
}

impl specimen::Evaluator for Runner {
    fn evaluate(&mut self, tile: &specimen::Dict) -> bool {
        if tile.get("box").is_none() {
            eprintln!("Encountered tile without a box entry: {:?}", tile);
        }
        if tile["box"] == Box::from("zoo") {
            self.zoo.zoo(tile.to_owned())
        } else if tile["box"] == Box::from("animalkind") {
            self.zoo.animal_kind(tile.to_owned())
        } else {
            eprintln!("Encountered unhandled box name: {}", tile["box"]);
            false
        }
    }
}

#[test]
fn test_focus_zoo() {
    specimen::run(
        Box::new(Runner::default()),
        &[specimen::file::File::read_local_file(
            "../test/zoo/zoo_data.yaml",
        )],
    )
}
