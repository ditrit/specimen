const zoo = require("./zoo.js");

const specimen = require("../../js/dist");

specimen.run(
  specimen.makeCodeboxSet({
    zoo: (s, input) => {
      let animal = input.animal;
      let expected = input.expected_result;

      if (animal !== undefined) {
        let output = zoo.addAnimal(animal);

        if (expected !== undefined) {
          s.expectEqual(output, expected, "result comparison");
        }
      }
    },

    animalkind: (s, input) => {
      let name = input.name;
      if (name === "deer") {
        s.expectEqual(input.horn, 2, "deer horns");
        s.expectEqual(input.leg, 4, "deer leg");
      } else if (name === "earthpony") {
        s.expectEqual(input.horn, 0, "earthpony horns");
        s.expectEqual(input.leg, 4, "earthpony leg");
      } else {
        s.fail("unknown animal name: " + name);
      }
    },
  }),
  [specimen.readLocalFile("zoo_data.yaml")]
);
