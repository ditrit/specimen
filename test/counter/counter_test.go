package zoo_test

import (
	"testing"

	specimen "github.com/ditrit/specimen/go"
)

var counter = 0

func TestCounting(t *testing.T) {
	specimen.Run(
		t,
		specimen.MakeCodeboxSet(map[string]specimen.BoxFunction{
			"counter": func(s *specimen.S, input specimen.Dict) {
				expected, found := input["expected_count"]
				if found {
					s.ExpectEqual(
						counter,
						expected.(int),
						"count comparison",
					)
				}
				counter += 1
			},
		}),
		[]specimen.File{
			specimen.ReadLocalFile("counter_data.yaml"),
		},
	)
}
