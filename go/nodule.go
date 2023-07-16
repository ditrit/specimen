package specimen

import (
	"errors"
	"fmt"

	"github.com/ditrit/specimen/go/focustree"
	"github.com/ditrit/specimen/go/syaml"
	"gopkg.in/yaml.v3"
)

func (n *Nodule) GetLocation() string {
	return fmt.Sprintf("%s:%d:%d", n.FilePath, n.YamlNode.Line, n.YamlNode.Column)
}

func (n Nodule) Errorf(format string, a ...interface{}) error {
	info := fmt.Sprintf(format, a...)
	return fmt.Errorf("(%s): %s", n.GetLocation(), info)
}

func NewNoduleFromFile(file File) (n Nodule, err error) {
	n.FilePath = file.Path

	document := &yaml.Node{}

	err = yaml.Unmarshal(file.Content, document)
	if err != nil {
		return
	}

	// mapping
	n.YamlNode, err = syaml.EnterDocument(document)
	if err != nil {
		return
	}
	if !syc.IsMapping(n.YamlNode) {
		err = fmt.Errorf("the root of the document must be a yaml mapping")
		return
	}

	n.InitializeTree()

	return
}

// The initialization creates all the nodules corresponding to the mapping nodes of the yaml tree, except for the PENDING nodes. It fills the fields Flag, HasContentKey and Children. **It expects YamlNode and FilePath to be already set**, and it sets YamlNode and FilePath for its children.
func (n *Nodule) InitializeTree() (err error) {
	if !syc.IsMapping(n.YamlNode) {
		err = fmt.Errorf("the root of the document must be a yaml mapping")
		return
	}
	flagNode := syc.MapTryGetValue(n.YamlNode, "flag")
	if flagNode != nil {
		n.Flag = readFlag(flagNode)
	}

	if n.Flag == focustree.Skip {
		return
	}

	contentNode := syc.MapTryGetValue(n.YamlNode, "content")
	if contentNode != nil {
		n.HasContentKey = true
		for _, childYamlNode := range contentNode.Content {
			child := Nodule{
				FilePath: n.FilePath,
				YamlNode: childYamlNode,
			}
			child.InitializeTree()
			n.Children = append(n.Children, child)
		}
	}

	return
}

// Populate fills the DataMatrix and DataOrder fields from the Yaml data
func (n *Nodule) Populate(dataMatrix map[string][]string, dataOrder []string) (err error) {
	// Todo: improve the performance by detecting all the cases where creating a copy of the dataMatrix and dataOrder is unneccessary
	if len(n.YamlNode.Content) == 0 {
		return nil
	}

	// Cloning dataMatrix and dataOrder
	n.DataMatrix = map[string][]string{}
	for k, v := range dataMatrix {
		n.DataMatrix[k] = v
	}
	n.DataOrder = make(
		[]string,
		len(dataOrder),
		len(dataOrder)+len(n.YamlNode.Content)/2,
	)
	copy(n.DataOrder, dataOrder)

	errorSlice := []error{}

	for k := 0; k < len(n.YamlNode.Content)/2; k += 1 {
		key := n.YamlNode.Content[2*k]
		if key.Value == "content" {
			continue
		}
		value := n.YamlNode.Content[2*k+1]
		if value.Kind == yaml.SequenceNode {
			slice := []string{}
			for _, entry := range value.Content {
				slice = append(slice, entry.Value)
			}
			n.DataMatrix[key.Value] = slice
			n.DataOrder = append(n.DataOrder, key.Value)
		} else if value.Kind == yaml.ScalarNode {
			n.DataMatrix[key.Value] = []string{value.Value}
			n.DataOrder = append(n.DataOrder, key.Value)
			// /!\ TODO: extract DataMatrix&DataOrder to their own module and
			// perform a check to see if the entry is already in the map, so
			// as to avoid duplicate entries in the DataOrder slice.
		} else {
			errorSlice = append(errorSlice, n.Errorf("Unexpected node kind [%d] for key [%s]", value.Kind, key.Value))
		}
	}

	// Recursing over children
	for k := range n.Children {
		errorSlice = append(errorSlice, n.Children[k].Populate(n.DataMatrix, n.DataOrder))
	}

	err = errors.Join(errorSlice...)

	return
}

// TODO: make it so inserting a key in n.DataOrder checks and when present, removes the older instance of the key.

func (n *Nodule) NewResolveDataMatrixIterator() func() Dict {
	// reverse the dataOrder so that we iterate quickly on the latest keys, and
	// more slowly in the earlier keys
	dataOrder := make([]string, len(n.DataOrder))
	for k, v := range n.DataOrder {
		dataOrder[len(n.DataOrder)-1-k] = v
	}

	// Calculate the total number of combinations and the intermediate slice
	// sizes
	totalCombinations := 1
	sizeSlice := make([]int, 0, len(n.DataOrder))
	for _, key := range dataOrder {
		set := n.DataMatrix[key]
		totalCombinations *= len(set)
		sizeSlice = append(sizeSlice, totalCombinations)
	}

	// The indexSlice traks the progress of values through every set
	indexSlice := make([]int, len(dataOrder))

	// The combination is the variable that will be updated and returned by the
	// iterator
	combination := Dict{}
	// Initialize the combination
	for k, key := range dataOrder {
		combination[key] = n.DataMatrix[key][indexSlice[k]]
		// ^ Note that in this loop, indexSlice[k] is actually always 0
	}

	// Create a closure-based iterator function
	index := 0
	return func() Dict {
		if index == 0 {
			index += 1
			return combination
		} else if index == totalCombinations {
			index += 1
			return nil
		}

		// Go through the keys to find which one is affected by the index change
		for k, key := range dataOrder {
			if index%sizeSlice[k] > 0 {
				// bump the identified index
				indexSlice[k] += 1
				// update the combination entry corresponding to the identified key
				combination[key] = n.DataMatrix[key][indexSlice[k]]
				break
			}
		}
		index += 1
		return combination
	}
}
