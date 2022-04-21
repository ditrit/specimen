import sys
sys.path.append(".")
import python as specimen
import zoo


@specimen.run(
    specimen.read_local_file("zoo_data.yaml", location=__file__),
)
class TestZoo(specimen.TestCase):
    def zoo(self, animal=None, expected_result=None, **kwargs):
        if animal is not None:
            output = zoo.add_animal(animal)
            if expected_result is not None:
                self.assertEqual(output, expected_result)
    def animalkind(self, name, horn, leg, **kwargs):
        if name == "deer":
            self.assertEqual(horn, 2, "deer horns")
            self.assertEqual(leg, 4, "deer legs")
        elif name == "earthpony":
            self.assertEqual(horn, 0, "earthpony horn")
            self.assertEqual(leg, 4, "earthpony legs")
        else:
            self.test_case.fail("unknown animal name: " + name)