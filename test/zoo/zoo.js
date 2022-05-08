let zoo = [];

function addAnimal(animal) {
  zoo.push(animal);
  return zoo.join(" ");
}

module.exports = { addAnimal };
