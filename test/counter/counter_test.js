const specimen = require("../../js/dist");

let counter = 0;

specimen.run(
  (s, tile) => {
    expected = tile.expected_count;
    if (expected !== undefined) {
      s.expectEqual(counter, Number(expected), "count comparison");
    }
    counter += 1;
  },
  [specimen.readLocalFile("counter_data.yaml", __dirname)]
);
