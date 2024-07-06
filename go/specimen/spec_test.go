package specimen_test

import (
	"fmt"
	"io"
	"os"
	"regexp"
	"strings"
	"testing"

	"github.com/ditrit/specimen/go/specimen"

	"gopkg.in/yaml.v3"
)

func indent(s string, n int) string {
	indent := strings.Repeat(" ", n)
	return indent + strings.ReplaceAll(s, "\n", "\n"+indent)

}

func callLogger(s *specimen.S, tile specimen.Dict, finishCapturingStdout func() string) {
	var callYaml interface{}
	err := yaml.Unmarshal([]byte(tile["calls"]), &callYaml)
	if err != nil {
		panic(err)
	}
	t := testing.T{}
	callSlice := callYaml.([]interface{})
	index := 0
	var error error = nil
	specimen.Run(
		&t,
		func(ss *specimen.S, specTile specimen.Dict) {
			if error == nil {
				expectedTile := callSlice[index].(map[string]interface{})
				same := true
				if len(expectedTile)+1 != len(specTile) {
					same = false
				}
				if same {
					for k, v := range specTile {
						if k == "filepath" {
							continue
						}
						if expectedTile[k] != v {
							same = false
							break
						}
					}
				}
				if !same {
					error = fmt.Errorf(
						"[Call %d]\nExpected: %v\nActual__: %v",
						index, expectedTile, specTile,
					)
				}
			}
			index++
		},
		[]specimen.File{
			{
				Path:    tile["filepath"],
				Content: []byte(tile["spec"]),
			},
		},
	)

	finishCapturingStdout()

	if error != nil {
		s.Fail(error.Error())
		return
	}

	if index != len(callSlice) {
		s.Fail(fmt.Sprintf("Expected %d calls, but got %d", len(callSlice), index))
		return
	}
}

func report(s *specimen.S, tile specimen.Dict, finishCapturingStdout func() string) {
	var behaviorYaml interface{}
	err := yaml.Unmarshal([]byte(tile["behavior"]), &behaviorYaml)
	if err != nil {
		panic(err)
	}
	behavior := map[string]string{}
	for k, v := range behaviorYaml.(map[string]interface{}) {
		behavior[k] = v.(string)
	}

	t := testing.T{}
	specimen.Run(
		&t,
		func(ss *specimen.S, specTile specimen.Dict) {
			outcome := behavior[specTile["letter"]]
			switch outcome {
			case "pass":
			case "fail":
				ss.Fail("This test is expected to fail")
			case "abort":
				ss.Abort("This test is expected to abort")
			default:
				ss.Abort("Unknown outcome: " + outcome)
			}
		},
		[]specimen.File{
			{
				Path:    tile["filepath"],
				Content: []byte(tile["spec"]),
			},
		},
	)

	out := finishCapturingStdout()

	reportRegex := regexp.MustCompile(tile["report"])
	if !reportRegex.MatchString(out) {
		s.Fail(fmt.Sprintf(
			"Expected:\n%s\nActual:\n%s",
			indent(tile["report"], 4), indent(out, 4),
		))
	}
}

func TestSpec(t *testing.T) {
	specimen.Run(
		t,
		func(s *specimen.S, tile specimen.Dict) {
			// Capture stdout
			rescueStdout := os.Stdout
			r, w, _ := os.Pipe()
			os.Stdout = w

			finishCapturingStdout := func() string {
				// Read captured stdout
				w.Close()
				out, err := io.ReadAll(r)
				if err != nil {
					panic(err)
				}
				os.Stdout = rescueStdout
				return string(out)
			}

			switch tile["box"] {
			case "call-logger":
				callLogger(s, tile, finishCapturingStdout)
			case "report":
				report(s, tile, finishCapturingStdout)
			}
		},
		[]specimen.File{
			specimen.ReadLocalFile("../../spec/about.yaml"),
			specimen.ReadLocalFile("../../spec/flag.yaml"),
			specimen.ReadLocalFile("../../spec/matrix.yaml"),
			specimen.ReadLocalFile("../../spec/report.yaml"),
		},
	)
}
