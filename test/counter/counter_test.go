package zoo_test

import (
	"strconv"
	"testing"

	specimen "github.com/ditrit/specimen/go"
)

var counter = 0

func TestCounting(t *testing.T) {
	specimen.Run(
		t,
		func(s *specimen.S, input specimen.Dict) {
			expectedString, found := input["expected_count"]
			if found {
				expected, err := strconv.Atoi(expectedString)
				if err != nil {
					panic(err)
				}

				s.ExpectEqual(counter, expected, "count comparison")
			}
			counter += 1
		},
		[]specimen.File{
			specimen.ReadLocalFile("counter_data.yaml"),
		},
	)
}
