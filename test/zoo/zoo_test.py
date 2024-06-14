import sys

sys.path.extend([".", "../.."])
import python as specimen
import zoo


def zoofunction(context, animal=None, expected_result=None, **kwargs):
    if animal is not None:
        output = zoo.add_animal(animal)
        if expected_result is not None:
            context.test_case.assertEqual(output, expected_result)


def animalkind(context, name, horn, leg, **kwargs):
    if name == "deer":
        context.test_case.assertEqual(horn, 2, "deer horns")
        context.test_case.assertEqual(leg, 4, "deer legs")
    elif name == "earthpony":
        context.test_case.assertEqual(horn, 0, "earthpony horn")
        context.test_case.assertEqual(leg, 4, "earthpony legs")
    else:
        self.test_case.fail("unknown animal name: " + name)


@specimen.run(
    specimen.read_local_file("zoo_data.yaml", location=__file__),
)
def test(context, box, **kwargs):
    if box == "zoo":
        zoofunction(context, **kwargs)
    elif box == "animalkind":
        animalkind(context, **kwargs)
