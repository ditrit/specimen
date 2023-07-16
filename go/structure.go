package specimen

import (
	"testing"

	"github.com/ditrit/specimen/go/focustree"
	"github.com/ditrit/specimen/go/orderedstringmap"
	"gopkg.in/yaml.v3"
)

// FailStatus specifies the kind of failure a slab produced, if any
type FailStatus int

const (
	Pristine FailStatus = iota
	Failed
	Aborted
	Panicked
)

// S is a context structure for the Specimen package. It is preseved from the
// run of a slab to the next
type S struct {
	T             *testing.T
	slabCount     int
	slabPassed    int
	slabFailed    int
	slabAborted   int
	slabPanicked  int
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

// Dict is a shorthand for map of string to string
type Dict = map[string]string

// BoxFunction is the type that user-defined functions must implement in codeboxes
type BoxFunction func(s *S, tile Dict)

// NoduleRoot is used to gather the files for exploration by the focustree package
type NoduleRoot []Nodule

// Nodule is a node inside a file
type Nodule struct {
	FilePath      string
	YamlNode      *yaml.Node
	Flag          focustree.FlagType
	HasContentKey bool
	Children      []Nodule
	DataMatrix    orderedstringmap.OSM
}
