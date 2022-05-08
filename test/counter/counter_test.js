const specimen = require("../../js/dist");

let counter = 0;

specimen.run(
  specimen.makeCodeboxSet({
    counter: (s, input) => {
      expected = input.expected_count;
      if (expected !== undefined) {
        s.expectEqual(counter, expected, "count comparison");
      }
      counter += 1;
    },
  }),
  [specimen.readLocalFile("counter_data.yaml")]
);
