package danger_test

import (
	"testing"

	specimen "github.com/ditrit/specimen/go/specimen"
)

func TestDanger(t *testing.T) {
	alt := testing.T{}
	specimen.Run(
		&alt,
		func(s *specimen.S, input specimen.Dict) {
			panic(input)
		},
		[]specimen.File{specimen.ReadLocalFile("./danger.yaml")},
	)
	if !alt.Failed() {
		t.Fail()
	}
}
