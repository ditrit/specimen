const specimen = require("../../js/dist");

function turn_page(s, input) {
  book = new Book(input.book);
  book.turnPage(input.turn_page_count);

  s.expectEqual(
    {
      title: book.title,
      left_page: book.leftPage,
      size: book.size,
    },
    input.expected_result,
    "result comparison"
  );
}

function turn_page_expect_page(s, input) {
  book = new Book(input.book);
  book.turnPage(input.turn_page_count);

  if (input.expected_left_page !== book.leftPage) {
    s.fail("page mismatch");
  }
}

function get_page(s, input) {
  book = new Book(input.book);
  s.expectEqual(book.getPage(), input.expected_result, "result comparison");
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
    specimen.readLocalFile("novel_data_with_alias.yaml", __dirname),
  ]
);
