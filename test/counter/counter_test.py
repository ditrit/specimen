import sys

sys.path.extend([".", "../.."])
import python as specimen

counter = [0]


@specimen.run(
    specimen.read_local_file("counter_data.yaml", location=__file__),
)
def test(context, expected_count=None, **kwargs):
    try:
        if expected_count is not None:
            context.test_case.assertEqual(
                counter[0], expected_count, "count comparison"
            )
    finally:
        counter[0] += 1
