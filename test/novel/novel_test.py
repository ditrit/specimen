import dataclasses
import sys
sys.path.append(".")

import python as specimen
import novel

# Alias are not supported yet :(

@specimen.run(
    specimen.read_local_file("novel_data.yaml", location=__file__),
    # specimen.read_local_file("novel_data_with_alias.yaml", location=__file__),
)
class TestNovel(specimen.TestCase):
    def turn_page(self, book, turn_page_count, expected_result):
        book_instance = novel.Book(**book)
        book_instance.turn_page(turn_page_count)

        self.assertEqual(dataclasses.asdict(book_instance), expected_result, "book result comparison")

    def turn_page_expect_page(self, book, turn_page_count, expected_left_page):
        book_instance = novel.Book(**book)
        book_instance.turn_page(turn_page_count)

        self.assertEqual(book_instance.left_page, expected_left_page, "page matching")

    def get_page(self, book, expected_result, **kwargs):
        book_instance = novel.Book(**book)
        self.assertEqual(book_instance.get_page(), expected_result, "page comparison")