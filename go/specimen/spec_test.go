package specimen_test

import (
	"bytes"
	"fmt"
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

func callLogger(s *specimen.S, tile specimen.Dict) {
	var callYaml interface{}
	err := yaml.Unmarshal([]byte(tile["calls"]), &callYaml)
	if err != nil {
		panic(err)
	}
	t := testing.T{}
	callSlice := callYaml.([]interface{})
	index := 0
	var error error = nil

	specimen.IolessRun(
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
		&bytes.Buffer{},
	)

	if error != nil {
		s.Fail(error.Error())
		return
	}

	if index != len(callSlice) {
		s.Fail(fmt.Sprintf("Expected %d calls, but got %d", len(callSlice), index))
		return
	}
}

func report(s *specimen.S, tile specimen.Dict) {
	var behaviorYaml interface{}
	err := yaml.Unmarshal([]byte(tile["behavior"]), &behaviorYaml)
	if err != nil {
		panic(err)
	}
	behavior := map[string]string{}
	for k, v := range behaviorYaml.(map[string]interface{}) {
		behavior[k] = v.(string)
	}

	buffer := bytes.Buffer{}

	t := testing.T{}
	specimen.IolessRun(
		&t,
		func(ss *specimen.S, specTile specimen.Dict) {
			outcome := behavior[specTile["letter"]]
			switch outcome {
			case "pass":
			case "fail":
				ss.Fail("failure")
			case "abort":
				ss.Abort("aborted")
			default:
				ss.Abort(fmt.Sprintf("unknown outcome: '%s', specTile: %s", outcome, specTile))
			}
		},
		[]specimen.File{
			{
				Path:    tile["filepath"],
				Content: []byte(tile["spec"]),
			},
		},
		&buffer,
	)

	out := buffer.String()

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
			switch tile["box"] {
			case "call-logger":
				callLogger(s, tile)
			case "report":
				report(s, tile)
			default:
				s.Abort(fmt.Sprintf("unknown box: %s", tile["box"]))
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
