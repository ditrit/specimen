package specimen

import (
	"fmt"
	"log"
	"runtime/debug"
	"strings"

	"github.com/ditrit/specimen/go/specimen/focustree"
	"github.com/ditrit/specimen/go/specimen/syaml"
	"gopkg.in/yaml.v3"
)

var _ = func() int {
	log.SetFlags(0)
	return 0
}()

// This file implement the focustree.Node interface for each of the the four levels found in the test data tree.

// Level:
// - TreeRoot
// - YamlFile
// - Nodule
// - Nodule
// - ...

// TreeRoot implements focustree.Node

func (t TreeRoot) GetChildren() (children []focustree.Node) {
	for _, c := range t {
		children = append(children, c)
	}
	return
}

func (TreeRoot) IsLeaf() bool {
	return false
}

func (TreeRoot) GetFlag() focustree.FlagType {
	return focustree.None
}

func (TreeRoot) Warning(info string) {
	log.Printf("Warning: TreeRoot: %s\n", info)
}

// Nodule implements focustree.Node

func (n Nodule) GetChildren() (children []focustree.Node) {
	// convert []Nodule to []focustree.Node
	for _, c := range n.Children {
		children = append(children, c)
	}
	return
}

func (n Nodule) IsLeaf() bool {
	return len(n.Children) == 0
}

func (n Nodule) GetFlag() focustree.FlagType {
	return n.Flag
}

func (n Nodule) Warning(info string) {
	log.Printf("Warning: %s %s(%s): %s\n", n.Kind, n.Name, n.Location, info)
}

// ---

var syc = syaml.NewSyaml(true, 90)

func (n *Nodule) InitializeFile() (err error) {
	document := &yaml.Node{}

	err = yaml.Unmarshal(n.File.Content, document)
	if err != nil {
		return
	}

	// mapping
	n.Mapping, err = syaml.EnterDocument(document)
	if err != nil {
		return
	}
	if !syc.IsMapping(n.Mapping) {
		err = fmt.Errorf("the root of the document must be a yaml mapping")
		return
	}

	n.Initialize()

	return
}

// Initialize tries to compute the properties of a Nodule necessary to perform
// the slab selection.
func (n *Nodule) Initialize() (err error) {
	// location
	n.Location = fmt.Sprintf("%s:%d:%d", n.File.Path, n.Mapping.Line, n.Mapping.Column)

	if !syc.IsMapping(n.Mapping) {
		err = n.Errorf("expected a mapping")
		return
	}

	// input map initialisation
	n.Input = map[string]interface{}{}

	// flag
	n.Flag = readFlag(n.Mapping)

	// name
	if nameNode := syc.MapTryGetValue(n.Mapping, "name"); nameNode != nil {
		n.Name = nameNode.Value
	}

	// content node (for children)
	contentNode := syc.MapTryGetValue(n.Mapping, "content")
	if contentNode == nil {
		if syc.MapTryGetValue(n.Mapping, "input") == nil {
			err = n.Errorf("missing both \"content\" key and \"input\" key in the mapping")
		}
		return
	}

	// /\ content node processing
	// kind
	if len(contentNode.Content) > 0 && n.Kind == "Slab" {
		n.Kind = "Node"
	}

	// children
	if !syc.IsSequence(contentNode) {
		err = n.Errorf("the \"content\" value must be a yaml sequence")
		return
	}
	for _, node := range contentNode.Content {
		child := Nodule{File: n.File, Kind: "Slab", Mapping: node}
		err := child.Initialize()
		if err != nil {
			log.Printf("%s -- this has been ignored\n", err.Error())
		} else if n.Flag != focustree.Skip {
			n.Children = append(n.Children, child)
		}
	}
	// \/ content node processing

	return
}

// Populate fills the Codebox and Input fields through the tree of nodules
func (n *Nodule) Populate(
	codeboxSet map[string]*Codebox, codebox *Codebox,
	input map[string]interface{},
	matrix map[string][]interface{},
) error {
	if n.Flag == focustree.Skip {
		return nil
	}

	// box
	if boxNode := syc.MapTryGetValue(n.Mapping, "box"); boxNode != nil {
		if otherCodebox, ok := codeboxSet[boxNode.Value]; ok {
			codebox = otherCodebox
		} else {
			return n.Errorf("no codebox with the name \"%s\" has been registered", boxNode.Value)
		}
	}

	// input
	for k, v := range input {
		n.Input[k] = v
	}
	inputNode := syc.MapTryGetValue(n.Mapping, "input")
	if inputNode != nil {
		if !syc.IsMapping(inputNode) {
			return n.Errorf("the value of \"input\" must be a mapping")
		} else {
			localInput := syc.ExtractContent(inputNode).(map[string]interface{})
			for k, v := range localInput {
				n.Input[k] = v
			}
		}
	}

	// matrix
	for k, v := range matrix {
		n.Matrix[k] = v
	}
	matrixNode := syc.MapTryGetValue(n.Mapping, "matrix")
	if matrixNode != nil {
		if !syc.IsMapping(matrixNode) {
			return n.Errorf("the value of \"matrix\" must be a mapping")
		}

		n.Matrix = map[string][]interface{}{}

		for k := 0; k < len(matrixNode.Content); k += 2 {
			key := syc.GetString(matrixNode.Content[k])
			valueNode := matrixNode.Content[k+1]
			syc.AssertIsSequence(valueNode)
			slice := []interface{}{}
			for _, node := range valueNode.Content {
				slice = append(slice, syc.ExtractContent(node))
			}
			n.Matrix[key] = slice
		}
	}

	// slab case
	if len(n.Children) == 0 {
		if codebox == nil {
			return n.Errorf("no box declared down to this slab")
		}
		n.Codebox = codebox
		if inputNode == nil {
			return n.Errorf("the input entry is mandatory on slabs")
		}

		if len(n.Matrix) > 0 {
			debug.SetMaxStack(50 * 1000)
			// generate matrix children
			noduleSlice := []Nodule{n.Clone()}
			noduleSlice[0].Flag = focustree.None
			noduleSlice[0].Matrix = nil
			for key, valueSlice := range n.Matrix {
				noduleSlice = multiplySlice(key, valueSlice, noduleSlice)
			}
			n.Children = noduleSlice
			for _, child := range n.Children {
				for k, v := range n.Input {
					child.Input[k] = v
				}
			}
		}
		// all good with the current slab
		return nil
	}

	// node case: populating children
	validChildren := []Nodule{}
	for _, child := range n.Children {
		err := child.Populate(codeboxSet, codebox, n.Input, n.Matrix)
		if err != nil {
			log.Println(err.Error())
		} else {
			validChildren = append(validChildren, child)
		}
	}

	if len(validChildren) == 0 {
		return n.Errorf("no valid children")
	}

	n.Children = validChildren
	return nil
}

func (n Nodule) Errorf(format string, a ...interface{}) error {
	info := fmt.Sprintf(format, a...)
	return fmt.Errorf("%s %s(%s): %s", n.Kind, n.Name, n.Location, info)
}

// ---

func readFlag(node *yaml.Node) (flag focustree.FlagType) {
	syc.AssertIsMapping(node)
	flagNode := syc.MapTryGetValue(node, "flag")

	if flagNode == nil {
		return
	}

	var flagName string
	both := false

	for _, word := range strings.Split(syc.GetString(flagNode), " ") {
		switch word {
		case "FOCUS":
			if flag == focustree.Skip {
				both = true
			}
			flag = focustree.Focus
			flagName = word
		case "PENDING":
			if flag == focustree.Focus {
				both = true
			}
			flag = focustree.Skip
			flagName = word
		default:
			if isUpperCase(word) {
				log.Printf("Warning: Unrecognised all uppercase flag \"%s\". It has been ignored.\n", word)
			}
		}
	}
	if both {
		log.Printf("Warning: both FOCUS and PENDING have been found among the flags of a node. %s has been kept.\n", flagName)
	}

	return
}

// isUppercase tells if the string only consists of letters in the range [A-Z]
func isUpperCase(s string) bool {
	for _, r := range s {
		if r < 'A' || 'Z' < r {
			return false
		}
	}
	return true
}

func (n *Nodule) Clone() (m Nodule) {
	m.File = n.File
	m.Mapping = n.Mapping
	m.Kind = n.Kind
	m.Location = n.Location
	m.Flag = n.Flag
	m.Name = n.Name
	m.Children = n.Children
	m.Codebox = n.Codebox
	m.Input = map[string]interface{}{}
	for k, v := range n.Input {
		m.Input[k] = v
	}
	m.Matrix = n.Matrix
	return
}

func multiplySlice(key string, valueSlice []interface{}, noduleSlice []Nodule) (resultSlice []Nodule) {
	for _, nodule := range noduleSlice {
		for k, value := range valueSlice {
			other := nodule.Clone()
			other.Input[key] = value
			other.Location = fmt.Sprintf("%s(%s[%d])", other.Location, key, k)
			resultSlice = append(resultSlice, other)
		}
	}
	return
}
