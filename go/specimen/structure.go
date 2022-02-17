package specimen

import (
	"testing"

	"github.com/ditrit/specimen/go/specimen/focustree"
	"gopkg.in/yaml.v3"
)

type FailStatus int

const (
	Pristine FailStatus = iota
	Failed
	Aborted
	Panicked
)

// S is a context structure for the Specimen package
type S struct {
	t             *testing.T
	slabCount     int
	slabPassed    int
	slabFailed    int
	slabAborted   int
	slabPanicked  int
	nodulePending int
	failureReport []string

	// The below values are reset for each slab
	status   FailStatus
	failInfo []string
}

// File represents a file by its path and content
type File struct {
	Path    string
	Content []byte
}

// Dict is a shorthand for map of strings to interfaces
type Dict = map[string]interface{}

// BoxFunction is the type that user-defined functions must implement in codeboxes
type BoxFunction func(s *S, input Dict)

// Codebox is a function with a name to be matched with data from the yaml files
type Codebox struct {
	// Name is the name given when registring the codebox
	Name string
	// BoxFunction is the function which adapts to the code.
	BoxFunction BoxFunction
}

// TreeRoot is used to gather the files for exploration by the focustree package
type TreeRoot []Nodule

// Nodule is a node inside a file
type Nodule struct {
	File    *File
	Mapping *yaml.Node
	// Kind is one of "File", "Node", "Slab" -- Kind is used in error reports
	Kind string
	// Location is a clickable link to the beginning of the nodule
	Location string
	Flag     focustree.FlagType
	// Name is an indicative name for the nodule
	Name     string
	Children []Nodule
	Codebox  *Codebox
	Input    map[string]interface{}
	Matrix   map[string][]interface{}
}
