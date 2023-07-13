package syaml

import (
	"fmt"

	"gopkg.in/yaml.v3"
)

func EnterDocument(document *yaml.Node) (node *yaml.Node, err error) {
	if document.Kind != yaml.DocumentNode {
		err = fmt.Errorf("expected the yaml root node kind to be yaml.DocumentNode")
		return
	}
	if len(document.Content) != 1 {
		err = fmt.Errorf("unexpected yaml document content length: %d (expected length 1)", len(document.Content))
		return
	}
	node = document.Content[0]
	return
}
