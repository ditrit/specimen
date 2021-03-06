package danger_test

import (
	"testing"

	"github.com/ditrit/specimen/go/specimen"
)

func TestDanger(t *testing.T) {
	alt := testing.T{}
	specimen.Run(
		&alt,
		specimen.MakeCodeboxSet(map[string]specimen.BoxFunction{
			"panicker": func(s *specimen.S, input specimen.Dict) {
				panic(input)
			},
		}),
		[]specimen.File{specimen.ReadLocalFile("danger.yaml")},
	)
	if !alt.Failed() {
		t.Fail()
	}
}
