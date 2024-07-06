package specimen

import (
	"errors"
	"fmt"

	"github.com/ditrit/specimen/go/specimen/focustree"
	"github.com/ditrit/specimen/go/specimen/orderedstringmap"
	"github.com/ditrit/specimen/go/specimen/syaml"
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

	err = n.InitializeTree()

	return
}

// The initialization creates all the nodules corresponding to the mapping nodes of the yaml tree, except for the PENDING nodes. It fills the fields Flag, HasContentKey and Children. **It expects YamlNode and FilePath to be already set**, and it sets YamlNode and FilePath for its children.
func (n *Nodule) InitializeTree() (err error) {
	if !syc.IsMapping(n.YamlNode) {
		err = fmt.Errorf("the content descendant nodes must be yaml mappings")
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

// Populate fills the DataMatrix with the Yaml data
func (n *Nodule) Populate(dataMatrix orderedstringmap.OSM) (err error) {
	// Cloning dataMatrix
	n.DataMatrix = dataMatrix.Clone()

	errorSlice := []error{}

	for k := 0; k < len(n.YamlNode.Content)/2; k += 1 {
		key := n.YamlNode.Content[2*k]
		if key.Value == "flag" || key.Value == "content" || key.Value == "about" {
			continue
		}
		value := n.YamlNode.Content[2*k+1]
		if value.Kind == yaml.SequenceNode {
			slice := []string{}
			for _, entry := range value.Content {
				slice = append(slice, entry.Value)
			}
			n.DataMatrix.Set(key.Value, slice)
		} else if value.Kind == yaml.ScalarNode {
			n.DataMatrix.Set(key.Value, []string{value.Value})
		} else {
			errorSlice = append(errorSlice, n.Errorf("Unexpected node kind [%d] for key [%s]", value.Kind, key.Value))
		}
	}

	// Recursing over children
	for k := range n.Children {
		errorSlice = append(errorSlice, n.Children[k].Populate(n.DataMatrix))
	}

	err = errors.Join(errorSlice...)

	return
}
