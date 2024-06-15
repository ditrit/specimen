const zoo = require("./zoo.js");

const specimen = require("../../js/dist");

function zoofunction(s, tile) {
  if (tile.animal) {
    let output = zoo.addAnimal(tile.animal);

    if (tile.expected_result) {
      s.expectEqual(output, tile.expected_result, "result comparison");
    }
  }
}

function animalkind(s, tile) {
  let name = tile.name;
  if (name === "deer") {
    s.expectEqual(Number(tile.leg), 4, "deer leg");
  } else if (name === "earthpony") {
    s.expectEqual(Number(tile.horn), 0, "earthpony horns");
    s.expectEqual(Number(tile.leg), 4, "earthpony leg");
  } else {
    s.fail("unknown animal name: " + name);
  }
}

specimen.run(
  (context, tile) => {
    if (tile.box == "zoo") {
      zoofunction(context, tile);
    } else if (tile.box == "animalkind") {
      animalkind(context, tile);
    } else {
      throw new Error(`unknown input box: ${tile.box}`);
    }
  },
  [specimen.readLocalFile("zoo_data.yaml", __dirname)]
);
