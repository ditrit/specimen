import * as specimen from "../../js/dist/index.js";

let codeboxSet = specimen.makeCodeboxSet({
  turn_page: (s, input) => {
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
  },

  turn_page_expect_page: (s, input) => {
    book = new Book(input.book);
    book.turnPage(input.turn_page_count);

    if (input.expected_left_page !== book.leftPage) {
      s.fail("page mismatch");
    }
  },

  get_page: (s, input) => {
    book = new Book(input.book);
    s.expectEqual(book.getPage(), input.expected_result, "result comparison");
  },
});

specimen.run(codeboxSet, [
  specimen.readLocalFile("novel_data.yaml"),
  specimen.readLocalFile("novel_data_with_alias.yaml"),
]);
