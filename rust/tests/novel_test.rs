// struct Book {
//     title: String,
//     left_page: i32,
//     size: i32,
// }

// impl Book {
//     fn turn_page(&mut self, count: i32) {
//         self.left_page += 2 * count;

//         if self.left_page < 0 {
//             self.left_page = 0;
//         } else if self.left_page >= self.size {
//             self.left_page = self.size - 1;
//         }
//     }

//     fn get_page(&self) -> i32 {
//         self.left_page
//     }
// }

// #[derive(Debug, Default)]
// struct Runner {}

// impl specimen::Evaluator for Runner {
//     fn evaluate(&mut self, tile: &specimen::Dict) -> bool {}
// }

// #[test]
// fn test_counter() {
//     specimen::run(
//         Box::new(Runner::default()),
//         &[specimen::file::File::read_local_file(
//             "../test/novel/novel_data.yaml",
//         )],
//     )
// }
