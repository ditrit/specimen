package novel_test

import (
	"testing"

	"github.com/ditrit/specimen/example/novel"
	"github.com/ditrit/specimen/go/specimen"
)

func deserialize_book(book_data specimen.Dict) novel.Book {
	return novel.Book{
		Title:    book_data["title"].(string),
		LeftPage: book_data["left_page"].(int),
		Size:     book_data["size"].(int),
	}
}

var codeboxSet = specimen.MakeCodeboxSet(map[string]specimen.BoxFunction{
	"turn_page": func(s *specimen.S, input specimen.Dict) {
		book := deserialize_book(input["book"].(specimen.Dict))
		book.TurnPage(input["turn_page_count"].(int))

		s.ExpectEqual(
			specimen.Dict{
				"title":     book.Title,
				"left_page": book.LeftPage,
				"size":      book.Size,
			},
			input["expected_result"].(specimen.Dict),
			"result comparison",
		)
	},

	"turn_page_expect_page": func(s *specimen.S, input specimen.Dict) {
		book := deserialize_book(input["book"].(specimen.Dict))
		book.TurnPage(input["turn_page_count"].(int))

		if input["expected_left_page"] != book.LeftPage {
			s.Fail("page mismatch")
		}
	},

	"get_page": func(s *specimen.S, input specimen.Dict) {
		book := deserialize_book(input["book"].(specimen.Dict))
		s.ExpectEqual(
			book.GetPage(),
			input["expected_result"].(int),
			"result comparison",
		)
	},
})

func TestNovel(t *testing.T) {
	specimen.Run(
		t,
		codeboxSet,
		[]specimen.File{
			specimen.ReadLocalFile("novel_data.yaml"),
			specimen.ReadLocalFile("novel_data_with_alias.yaml"),
		},
	)
}
