package novel_test

import (
	"strconv"
	"testing"

	specimen "github.com/ditrit/specimen/go"
	"github.com/ditrit/specimen/test/novel"

	"gopkg.in/yaml.v3"
)

func deserializeBook(bookString string) novel.Book {
	book := novel.Book{}
	yaml.Unmarshal([]byte(bookString), &book)
	return book
}

func turnPage(s *specimen.S, input specimen.Dict) {
	book := deserializeBook(input["book"])
	pageCount, _ := strconv.Atoi(input["turn_page_count"])
	book.TurnPage(pageCount)

	s.ExpectEqual(
		book,
		deserializeBook(input["expected_result"]),
		"result comparison",
	)
}

func turnPageExpectPage(s *specimen.S, input specimen.Dict) {
	book := deserializeBook(input["book"])
	pageCount, _ := strconv.Atoi(input["turn_page_count"])
	book.TurnPage(pageCount)

	expected, _ := strconv.Atoi(input["expected_left_page"])
	if expected != book.LeftPage {
		s.Fail("page mismatch")
	}
}

func getPage(s *specimen.S, input specimen.Dict) {
	book := deserializeBook(input["book"])
	expected, _ := strconv.Atoi(input["expected_result"])
	s.ExpectEqual(book.GetPage(), expected, "result comparison")
}

func boxFunction(s *specimen.S, input specimen.Dict) {
	switch input["box"] {
	case "get_page":
		getPage(s, input)
	case "turn_page_expect_page":
		turnPageExpectPage(s, input)
	case "turn_page":
		turnPage(s, input)
	}
}

func TestNovel(t *testing.T) {
	specimen.Run(
		t,
		boxFunction,
		[]specimen.File{
			specimen.ReadLocalFile("novel_data.yaml"),
			// specimen.ReadLocalFile("novel_data_with_alias.yaml"),
		},
	)
}
