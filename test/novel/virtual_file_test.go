package novel_test

import (
	"testing"

	specimen "github.com/ditrit/specimen/go"
)

func TestVirtualFile(t *testing.T) {
	specimen.Run(
		t,
		boxFunction,
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
