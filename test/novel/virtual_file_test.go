package novel_test

import (
	"testing"

	"github.com/ditrit/specimen/go/specimen"
)

var virtualCodeboxSet = specimen.MakeCodeboxSet(map[string]specimen.BoxFunction{
	"turn_page": codeboxSet["turn_page"].BoxFunction,
	"get_page":  codeboxSet["get_page"].BoxFunction,
})

func TestVirtualFile(t *testing.T) {
	specimen.Run(
		t,
		codeboxSet,
		[]specimen.File{
			specimen.VirtualFileDedent("virtual_novel_data.yaml", []byte(`
            content:
              -
                box: turn_page
                input:
                  book:
                    title: aleph
                    left_page: 0
                    size: 90
                  turn_page_count: 4
                  expected_result:
                    title: aleph
                    left_page: 8
                    size: 90
              -
                box: get_page
                input:
                  book:
                    title: aleph
                    left_page: 44
                    size: 90
                  expected_result: 44
            `)),
		},
	)
}
