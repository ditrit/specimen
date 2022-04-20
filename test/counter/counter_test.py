import sys
sys.path.append(".")
import python as specimen

counter = [0]

@specimen.run(
    specimen.read_local_file("counter_data.yaml", location=__file__),
)
class TestCounting(specimen.TestCase):
    def counter(self, expected_count=None):
        if expected is not None:
            self.assertEqual(counter[0], expected, "count comparison")
        counter[0] += 1
        print(counter[0])
