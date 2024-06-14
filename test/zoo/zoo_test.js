const zoo = require("./zoo.js");

const specimen = require("../../js/dist");

function zoofunction(s, input) {
  let animal = input.animal;
  let expected = input.expected_result;

  if (animal !== undefined) {
    let output = zoo.addAnimal(animal);

    if (expected !== undefined) {
      s.expectEqual(output, expected, "result comparison");
    }
  }
}

function animalkind(s, input) {
  let name = input.name;
  if (name === "deer") {
    s.expectEqual(input.leg, 4, "deer leg");
  } else if (name === "earthpony") {
    s.expectEqual(input.horn, 0, "earthpony horns");
    s.expectEqual(input.leg, 4, "earthpony leg");
  } else {
    s.fail("unknown animal name: " + name);
  }
}

specimen.run(
  (context, input) => {
    if (input.box == "zoo") {
      zoo(context, input);
    } else if (input.box == "animalkind") {
      animalkind(context, input);
    } else {
      throw new Error(`unknown input box: ${input.box}`);
    }
  },
  [specimen.readLocalFile("zoo_data.yaml", __dirname)]
);
