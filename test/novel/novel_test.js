const yaml = require("yaml");

const { Book } = require("./novel");
const specimen = require("../../js/dist");

function turn_page(s, tile) {
  book = new Book(yaml.parse(tile.book));
  book.turnPage(tile.turn_page_count);

  s.expectEqual(
    {
      title: book.title,
      left_page: book.leftPage,
      size: book.size,
    },
    yaml.parse(tile.expected_result),
    "result comparison"
  );
}

function turn_page_expect_page(s, tile) {
  book = new Book(yaml.parse(tile.book));
  book.turnPage(tile.turn_page_count);

  if (Number(tile.expected_left_page) !== book.leftPage) {
    s.fail("page mismatch");
  }
}

function get_page(s, tile) {
  book = new Book(yaml.parse(tile.book));
  s.expectEqual(
    book.getPage(),
    Number(tile.expected_result),
    "result comparison"
  );
}

specimen.run(
  (s, input) => {
    ({
      turn_page,
      turn_page_expect_page,
      get_page,
    })[input.box](s, input);
  },
  [
    specimen.readLocalFile("novel_data.yaml", __dirname),
    // specimen.readLocalFile("novel_data_with_alias.yaml", __dirname),
  ]
);
