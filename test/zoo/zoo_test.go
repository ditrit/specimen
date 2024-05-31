package zoo_test

import (
	"testing"

	specimen "github.com/ditrit/specimen/go"
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
			"animalkind": func(s *specimen.S, input specimen.Dict) {
				name := input["name"].(string)
				if name == "deer" {
					s.ExpectEqual(input["horn"].(int), 2, "deer horns")
					s.ExpectEqual(input["leg"].(int), 4, "deer leg")
				} else if name == "earthpony" {
					s.ExpectEqual(input["horn"].(int), 0, "earthpony horn")
					s.ExpectEqual(input["leg"].(int), 4, "earthpony legs")
				} else {
					s.Fail("unknown animal name: " + name)
				}
			},
		}),
		[]specimen.File{
			specimen.ReadLocalFile("zoo_data.yaml"),
		},
	)
}
