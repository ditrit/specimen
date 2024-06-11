use serde::Deserialize;

#[derive(Debug, Default, Deserialize)]
struct Book {
    left_page: i32,
    size: i32,
}

impl Book {
    fn turn_page(&mut self, count: i32) {
        self.left_page += 2 * count;

        if self.left_page < 0 {
            self.left_page = 0;
        } else if self.left_page >= self.size {
            self.left_page = self.size - 1;
        }
    }

    fn get_page(&self) -> i32 {
        self.left_page
    }
}

fn deserialize_book(data: &Box<str>) -> Book {
    serde_yaml::from_str(data).unwrap()
}

fn turn_page(input: &specimen::Dict) -> Result<(), Box<str>> {
    let mut book = deserialize_book(&input["book"]);
    let count = input["turn_page_count"].parse().unwrap();
    book.turn_page(count);
    Ok(())
}

fn turn_page_expect_page(input: &specimen::Dict) -> Result<(), Box<str>> {
    let mut book = deserialize_book(&input["book"]);
    let count = input["turn_page_count"].parse().unwrap();
    book.turn_page(count);
    let expected_page = input["expected_left_page"].parse().unwrap();
    match book.left_page == expected_page {
        true => Ok(()),
        false => Err(Box::from(format!(
            "Expected page: {}, Actual page: {}",
            expected_page, book.left_page
        ))),
    }
}

fn get_page(input: &specimen::Dict) -> Result<(), Box<str>> {
    let book = deserialize_book(&input["book"]);
    let expected = input["expected_result"].parse().unwrap();
    match book.get_page() == expected {
        true => Ok(()),
        false => Err(Box::from(format!(
            "Expected page: {}, Actual page: {}",
            book.get_page(),
            book.left_page
        ))),
    }
}

#[test]
fn test_novel() {
    specimen::run(
        &mut |tile: &specimen::Dict| -> Result<(), Box<str>> {
            match &*tile["box"] {
                "get_page" => get_page(tile),
                "turn_page_expect_page" => turn_page_expect_page(tile),
                "turn_page" => turn_page(tile),
                _ => Err(Box::from(format!("Unknown box: {}", tile["box"]))),
            }
        },
        &[specimen::file::File::read_local_file(
            "../test/novel/novel_data.yaml",
        )],
    )
}
