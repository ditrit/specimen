package specimen

import (
	"fmt"
	"log"
	"runtime/debug"
	"strings"
	"testing"
	"time"

	"github.com/ditrit/specimen/go/focustree"
	"github.com/ditrit/specimen/go/orderedstringmap"
)

// Load the data of the given files and runs the code sandboxes with it
func Run(t *testing.T, boxFunction BoxFunction, fileSlice []File) {
	s := S{T: t}
	log.Printf("%s:\n", s.T.Name())
	// Todo: consider replacing all "log.Print(|f|ln)" by "s.t.Log(|f|ln)"
	// (which would require passing s as parameter everywhere around)

	// Parsing the data into a noduleRoot of Nodule-s, Nodule-s
	var noduleRoot NoduleRoot
	for _, file := range fileSlice {
		nodule, err := NewNoduleFromFile(file)
		if err != nil {
			log.Println(err.Error())
		} else {
			noduleRoot = append(noduleRoot, nodule)
		}
	}

	// Populating the dataMatrices through the tree
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
	selectedLeaves := focustree.ExtractSelectedLeaves(validTree)

	startTime := time.Now()

	// Run all the selected slab
	for _, leaf := range selectedLeaves {
		slab := leaf.(Nodule)
		// Pass the slab data to the testbox
		// - Manage the context (s, test start and test end)
		// - Recover from any panic that might arise during the testbox call
		// - Check the output if an expected output is provided
		// Nodule Start
		s.status = Pristine
		s.failInfo = nil

		// Nodule Run
		iterator := slab.NewResolveDataMatrixIterator()
		for {
			tile := iterator()
			if tile == nil {
				break
			}
			slab.runBoxFunction(&s, tile, boxFunction)
		}

		// Nodule End
		s.slabCount += 1
		switch s.status {
		case Pristine:
			s.slabPassed += 1
		case Failed:
			s.slabFailed += 1
		case Aborted:
			s.slabAborted += 1
		case Panicked:
			s.slabPanicked += 1
		}
		// summarize the failures
		if s.status != Pristine {
			slabInfo := fmt.Sprintf("(%s)", slab.GetLocation())

			info := strings.Join(s.failInfo, "; ")

			message := ""
			switch s.status {
			case Failed:
				message = "FAIL"
			case Aborted:
				message = "ABORT"
			case Panicked:
				message = "PANIC"
			}

			message = fmt.Sprintf("%s[slab: %s]: %s", message, slabInfo, info)

			s.failureReport = append(s.failureReport, message)
		}
	}

	duration := time.Since(startTime)

	// Reporting what has been saved in s
	var outcome = "SUCCESS"
	if len(s.failureReport) > 0 {
		s.T.Fail()
		log.Println(strings.Join(s.failureReport, "\n"))
		outcome = "FAILURE"
	}
	log.Printf(
		"Ran %d tiles in %v\n"+
			"%s -- %d Passed | %d Failed | %d Aborted | %d Panicked",
		s.slabCount, duration,
		outcome, s.slabPassed, s.slabFailed, s.slabAborted, s.slabPanicked,
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
				info += v
			} else if v, ok := data.(error); ok {
				info += v.Error()
			}

			s.status = Panicked
			s.failInfo = append(s.failInfo, info)
		}
	}()
	box(s, tile)
}
