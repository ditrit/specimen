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
        let animal = tile.get("animal");
        let expected = tile.get("expected_result");

        if let Some(animal) = animal {
            let output = self.add_animal(animal.to_owned());

            if let Some(expected) = expected {
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

#[test]
fn test_focus_zoo() {
    let mut zoo = Zoo(Vec::new());

    specimen::run(
        &mut |tile: &specimen::Dict| -> Result<(), Box<str>> {
            let result = if tile.get("box").is_none() {
                format!("Encountered tile without a box entry: {:?}", tile);
                false
            } else if tile["box"] == Box::from("zoo") {
                zoo.zoo(tile.to_owned())
            } else if tile["box"] == Box::from("animalkind") {
                zoo.animal_kind(tile.to_owned())
            } else {
                eprintln!("Encountered unhandled box name: {}", tile["box"]);
                false
            };
            match result {
                true => Ok(()),
                false => Err(Box::from("Failed")),
            }
        },
        &[specimen::file::File::read_local_file(
            "../test/zoo/zoo_data.yaml",
        )],
    )
}
