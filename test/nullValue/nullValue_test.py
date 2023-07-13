import sys

sys.path.append(".")

import python as specimen

# This test ensures that the null value does not crash the library engine


@specimen.run(
    specimen.read_local_file("nullValue_data.yaml", location=__file__),
)
class TestNovel(specimen.TestCase):
    def nullValue(self, nullValue):
        # empty test, since we are just checking that the null value is accepted
        # by the library
        pass
