package specimen

import (
	"fmt"
	"log"
	"runtime/debug"
	"strings"
	"testing"
	"time"

	"github.com/ditrit/specimen/go/specimen/focustree"
	"github.com/go-test/deep"
)

// Fail marks the slab as failed and saves the given information, if provided. It can be called multiple times for a single slab. All saved information will be reported.
func (s *S) Fail(info string) {
	s.status = Failed
	if len(info) > 0 {
		s.failInfo = append(s.failInfo, info)
	}
}

// Abort marks the slab as aborted and saves the given information, if provided.
func (s *S) Abort(info string) {
	s.status = Aborted
	if len(info) > 0 {
		s.failInfo = append(s.failInfo, info)
	}
	panic(nil)
}

// ExpectEqual test if the two given values are equal data structures
func (s *S) ExpectEqual(value, wanted interface{}, context string) {
	if diff := deep.Equal(value, wanted); diff != nil {
		if len(context) > 0 {
			context = "(" + context + "): "
		}
		s.Fail(context + strings.Join(diff, ", "))
	}
}

// MakeCodeboxSet names the box functions by their mapping key, making codeboxes
func MakeCodeboxSet(codeboxMap map[string]BoxFunction) map[string]*Codebox {
	set := make(map[string]*Codebox)
	for name, function := range codeboxMap {
		box := Codebox{Name: name, BoxFunction: function}
		set[name] = &box
	}
	return set
}

// Load the data of the given files and runs the code sandboxes with it
func Run(t *testing.T, codeboxSet map[string]*Codebox, dataFileSlice []File) {
	s := S{t: t}
	log.Printf("%s:\n", s.t.Name())
	// Todo: consider replacing all "log.Print(|f|ln)" by "s.t.Log(|f|ln)"
	// (which would require passing s as parameter everywhere around)

	// Parsing the data into a tree of Nodule-s, Nodule-s
	var tree TreeRoot
	for _, file := range dataFileSlice {
		nodule := Nodule{File: &file, Kind: "File"}
		err := nodule.InitializeFile()
		if err != nil {
			log.Println(err.Error())
		} else {
			tree = append(tree, nodule)
		}
	}

	// Populating input and codebox fields
	var validTree TreeRoot
	for _, nodule := range tree {
		// err := nodule.Populate(codeboxSet, nil, map[string]interface{}{})
		err := nodule.Populate(codeboxSet, nil, nil, nil)
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
		// Pass the slab data to the codebox
		// - Manage the context (s, test start and test end)
		// - Recover from any panic that might arise during the codebox call
		// - Check the output if an expected output is provided
		// Nodule Start
		s.status = Pristine
		s.failInfo = nil

		// Nodule Run
		slab.runCodebox(&s)

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
			slabInfo := fmt.Sprintf("%s(%s)", slab.Name, slab.Location)

			info := strings.Join(s.failInfo, "; ")

			message := ""
			switch s.status {
			case Failed:
				databoxInfo := ""
				if len(slab.Name) > 0 {
					databoxInfo = fmt.Sprintf("[nodule %s]", slab.Name)
				}
				message = fmt.Sprintf("FAIL%s", databoxInfo)
			case Aborted:
				message = "ABORT"
			case Panicked:
				message = "PANIC"
			}

			message = fmt.Sprintf("%s[codebox: %s][slab: %s]: %s", message, slab.Codebox.Name, slabInfo, info)

			s.failureReport = append(s.failureReport, message)
		}
	}

	duration := time.Since(startTime)

	// Reporting what has been saved in s
	var outcome = "SUCCESS"
	if len(s.failureReport) > 0 {
		s.t.Fail()
		log.Println(strings.Join(s.failureReport, "\n"))
		outcome = "FAILURE"
	}
	log.Printf(
		"Ran %d slabs in %v\n"+
			"%s -- %d Passed | %d Failed | %d Aborted | %d Panicked",
		s.slabCount, duration,
		outcome, s.slabPassed, s.slabFailed, s.slabAborted, s.slabPanicked,
	)
}

func (nodule Nodule) runCodebox(s *S) {
	defer func() {
		// report that the codebox has panicked
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
	nodule.Codebox.BoxFunction(s, nodule.Input)
}
