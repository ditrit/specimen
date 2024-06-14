import dataclasses
import sys
import yaml

sys.path.extend([".", "../.."])

import python as specimen
import novel

# Alias are not supported yet :(


class NovelTestCase:
    @staticmethod
    def turn_page(suit, book, turn_page_count, expected_result, **kw):
        book_instance = novel.Book(**yaml.load(book, Loader=yaml.Loader))
        book_instance.turn_page(int(turn_page_count))

        suit.test_case.assertEqual(
            dataclasses.asdict(book_instance),
            yaml.load(expected_result, Loader=yaml.Loader),
            "book result comparison",
        )

    @staticmethod
    def turn_page_expect_page(suit, book, turn_page_count, expected_left_page, **kw):
        book_instance = novel.Book(**yaml.load(book, Loader=yaml.Loader))
        book_instance.turn_page(int(turn_page_count))

        suit.test_case.assertEqual(
            book_instance.left_page, int(expected_left_page), "page matching"
        )

    @staticmethod
    def get_page(suit, book, expected_result, **kw):
        book_instance = novel.Book(**yaml.load(book, Loader=yaml.Loader))
        suit.test_case.assertEqual(
            book_instance.get_page(), int(expected_result), "page comparison"
        )


@specimen.run(
    specimen.read_local_file("novel_data.yaml", location=__file__),
    # specimen.read_local_file("novel_data_with_alias.yaml", location=__file__),
)
def test(suit, box, **tile):
    testsuit = NovelTestCase()
    testFunction = getattr(testsuit, box)
    testFunction(suit, **tile)
