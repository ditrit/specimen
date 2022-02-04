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
				animal := input["animal"]
				expected := input["expected_result"]

				if animal != nil {
					output := zoo.AddAnimal(animal.(string))

					if expected != nil {
						s.ExpectEqual(
							output,
							expected.(string),
							"result comparison",
						)
					}
				}
			},
		}),
		[]specimen.File{
			specimen.ReadLocalFile("zoo_data.yaml"),
		},
	)
}
