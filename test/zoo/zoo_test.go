package zoo_test

import (
	"strconv"
	"testing"

	"github.com/ditrit/specimen/go/specimen"
	"github.com/ditrit/specimen/test/zoo"
)

func Zoo(s *specimen.S, input specimen.Dict) {
	animal := input["animal"]
	expected := input["expected_result"]

	if len(animal) > 0 {
		output := zoo.AddAnimal(animal)

		if len(expected) > 0 {
			s.ExpectEqual(output, expected, "result comparison")
		}
	}
}

func AnimalKind(s *specimen.S, input specimen.Dict) {
	name := input["name"]
	horn, errHorn := strconv.Atoi(input["horn"])
	leg, errLeg := strconv.Atoi(input["leg"])
	if errHorn != nil || errLeg != nil {
		panic("failed to parse horn or leg number")
	}
	if name == "deer" {
		s.ExpectEqual(horn, 2, "deer horns")
		s.ExpectEqual(leg, 4, "deer leg")
	} else if name == "earthpony" {
		s.ExpectEqual(horn, 0, "earthpony horn")
		s.ExpectEqual(leg, 4, "earthpony legs")
	} else {
		s.Fail("unknown animal name: " + name)
	}
}

func TestFocusZoo(t *testing.T) {
	specimen.Run(
		t,
		func(s *specimen.S, input specimen.Dict) {
			if input["box"] == "zoo" {
				Zoo(s, input)
			} else if input["box"] == "animalkind" {
				AnimalKind(s, input)
			} else {
				t.Logf("Encountered unhandled box name: '%s'", input["box"])
				t.FailNow()
			}
		},
		[]specimen.File{
			specimen.ReadLocalFile("./zoo_data.yaml"),
		},
	)
}
