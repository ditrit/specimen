package datatree

import "gopkg.in/yaml.v3"

type Node interface {
	GetChildren() []Node
	ReadData() map[string]*yaml.Node
	WriteData()
}
