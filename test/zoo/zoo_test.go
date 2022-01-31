package zoo_test

import (
	"testing"

	"github.com/ditrit/specimen/go/specimen"
	"github.com/ditrit/specimen/test/zoo"
)

func TestFocusZoo(t *testing.T) {
	specimen.Run(
		t,
		specimen.MakeCodeboxSet(map[string]specimen.BoxFunction{
			"zoo": func(s *specimen.S, input specimen.Dict) {
				s.ExpectEqual(
					zoo.AddAnimal(input["animal"].(string)),
					input["expected_result"].(string),
					"result comparison",
				)
			},
		}),
		[]specimen.File{
			specimen.ReadLocalFile("zoo_data.yaml"),
		},
	)
}
