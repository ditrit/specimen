package specimen

import (
	"fmt"
	"io"
	"log"
	"os"
	"runtime/debug"
	"strings"
	"testing"
	"time"

	"github.com/ditrit/specimen/go/specimen/focustree"
	"github.com/ditrit/specimen/go/specimen/orderedstringmap"
)

func Run(t *testing.T, boxFunction BoxFunction, fileSlice []File) {
	IolessRun(t, boxFunction, fileSlice, os.Stdout)
}

// Load the data of the given files and runs the code sandboxes with it
func IolessRun(t *testing.T, boxFunction BoxFunction, fileSlice []File, stdout io.Writer) {
	s := S{T: t}

	// Parsing the data into a noduleRoot of Nodule-s, Nodule-s
	var noduleRoot NoduleRoot
	for _, file := range fileSlice {
		nodule, err := NewNoduleFromFile(file)
		if err != nil {
			fmt.Fprintf(stdout, "%s: %s", file.Path, err.Error())
		} else {
			noduleRoot = append(noduleRoot, nodule)
		}
	}

	// Populating the dataMatrices through the tree, forming a new
	// noduleRoot from the sucessfully populated nodules
	var validTree NoduleRoot
	for _, nodule := range noduleRoot {
		dataMatrix := orderedstringmap.NewOSM()
		dataMatrix.Set("filepath", []string{nodule.FilePath})
		err := nodule.Populate(dataMatrix)
		if err != nil {
			log.Println(err.Error())
		} else {
			validTree = append(validTree, nodule)
		}
	}

	// Retreiving focused nodes, if any. This is done using a
	// suffix tree-traversal: The presence of the FOCUS flag on a node is checked
	// after all its children have been checked. If a node which has FOCUS-ed
	// children is FOCUS-ed too, its FOCUS-ed flag is ignored and a warning is
	// issued.
	flagStat := focustree.FlagStat{}
	selectedLeaves := focustree.ExtractSelectedLeaves(validTree, &flagStat, stdout)

	startTime := time.Now()

	// Run all the selected slab
	for _, leaf := range selectedLeaves {
		slab := leaf.(Nodule)
		// Pass the tile data to the testbox
		// - Manage the context (s, test start and test end)
		// - Recover from any panic that might arise during the testbox call

		iterator := slab.DataMatrix.ProductIterator()
		index := 0
		for {
			tile := iterator()
			if tile == nil {
				break
			}
			// Tile Start
			s.status = Pristine
			s.failInfo = nil

			// Tile Run
			slab.runBoxFunction(&s, tile, boxFunction)

			// Tile End
			s.tileCount += 1
			switch s.status {
			case Pristine:
				s.tilePassed += 1
			case Failed:
				s.tileFailed += 1
			case Aborted:
				s.tileAborted += 1
			case Panicked:
				s.tilePanicked += 1
			}
			// summarize the failures
			if s.status != Pristine {
				info := strings.Join(s.failInfo, "; ")

				word := ""
				switch s.status {
				case Failed:
					word = "FAIL"
				case Aborted:
					word = "ABORT"
				case Panicked:
					word = "PANIC"
				}

				message := fmt.Sprintf("%s[%s][%d]: %s", word, slab.GetLocation(), index, info)

				s.failureReport = append(s.failureReport, message)
			}

			index += 1
		}
	}

	duration := time.Since(startTime)

	// Reporting what has been saved in s
	var outcome = "SUCCESS"
	if len(s.failureReport) > 0 {
		s.T.Fail()
		fmt.Fprintln(stdout, strings.Join(s.failureReport, "\n"))
		outcome = "FAILURE"
	}
	// Reporting flag stats
	if flagStat.FocusCount > 0 || flagStat.SkipCount > 0 {
		messageSlice := []string{}
		if flagStat.FocusCount > 0 {
			messageSlice = append(messageSlice, fmt.Sprintf("%d focused node(s)", flagStat.FocusCount))
		}
		if flagStat.SkipCount > 0 {
			messageSlice = append(messageSlice, fmt.Sprintf("%d pending node(s)", flagStat.SkipCount))
		}
		fmt.Fprintf(stdout, "Encountered %s\n", strings.Join(messageSlice, " and "))
	}
	fmt.Fprintf(
		stdout,
		"Ran %d tiles in %vms\n"+
			"%s -- %d Passed | %d Failed | %d Aborted | %d Panicked\n",
		s.tileCount, duration.Milliseconds(),
		outcome, s.tilePassed, s.tileFailed, s.tileAborted, s.tilePanicked,
	)
}

func (nodule Nodule) runBoxFunction(s *S, tile Dict, box BoxFunction) {
	defer func() {
		// report that the testbox has panicked
		if data := recover(); data != nil {
			if s.status == Aborted {
				return
			}

			report := strings.TrimSuffix(string(debug.Stack()), "\n")
			info := "\n>   " + strings.ReplaceAll(report, "\n", "\n>   ")
			if v, ok := data.(string); ok {
				info = fmt.Sprintf("\n>>> %s%s", v, info)
			} else if v, ok := data.(error); ok {
				info = fmt.Sprintf("\n>>> %s%s", v.Error(), info)
			}

			s.status = Panicked
			s.failInfo = append(s.failInfo, info)
		}
	}()
	box(s, tile)
}
