const specimen = require("../../js/dist");

let codeboxSet = specimen.makeCodeboxSet({
  nullValue: (s, input) => {
    let _ = input.nullValue
  },
});

specimen.run(codeboxSet, [
  specimen.readLocalFile("nullValue_data.yaml", __dirname),
]);
