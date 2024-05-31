package nullValue_test

import (
	"testing"

	specimen "github.com/ditrit/specimen/go"
)

// This test ensures that the null value does not crash the library engine

func TestNullValue(t *testing.T) {
	specimen.Run(
		t,
		specimen.MakeCodeboxSet(map[string]specimen.BoxFunction{
			"nullValue": func(s *specimen.S, input specimen.Dict) {
				_ = input["nullValue"]
			},
		}),
		[]specimen.File{
			specimen.ReadLocalFile("nullValue_data.yaml"),
		},
	)
}
