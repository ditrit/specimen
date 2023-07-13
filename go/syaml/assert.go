package syaml

import (
	"gopkg.in/yaml.v3"
)

func (c *Config) AssertIsScalar(node *yaml.Node) {
	if !c.IsScalar(node) {
		panic("expected a scalar node")
	}
}

func (c *Config) AssertIsString(node *yaml.Node) {
	if !c.IsString(node) {
		panic("expected a string")
	}
}

func (c *Config) AssertIsInteger(node *yaml.Node) {
	if !c.IsInterger(node) {
		panic("expected an integer")
	}
}

func (c *Config) AssertIsSequence(node *yaml.Node) {
	if !c.IsSequence(node) {
		panic("expected a sequence node")
	}
}

func (c *Config) AssertIsMapping(node *yaml.Node) {
	if !c.IsMapping(node) {
		panic("expected a mapping node")
	}
}

func (c *Config) AssertIsDocument(node *yaml.Node) {
	if !c.IsDocument(node) {
		panic("expected a document node")
	}
}

func (c *Config) IsMerge(node *yaml.Node) bool {
	return node.ShortTag() == "!!merge"
}

func (c *Config) IsScalar(node *yaml.Node) bool {
	c.Resolve(&node)
	return node.Kind == yaml.ScalarNode
}

func (c *Config) IsString(node *yaml.Node) bool {
	c.Resolve(&node)
	return node.Tag == "!!str" && c.IsScalar(node)
}

func (c *Config) IsInterger(node *yaml.Node) bool {
	c.Resolve(&node)
	return node.Tag == "!!int" && c.IsScalar(node)
}

func (c *Config) IsSequence(node *yaml.Node) bool {
	c.Resolve(&node)
	return node.Kind == yaml.SequenceNode
}

func (c *Config) IsMapping(node *yaml.Node) bool {
	c.Resolve(&node)
	return node.Kind == yaml.MappingNode
}

func (c *Config) IsDocument(node *yaml.Node) bool {
	return node.Kind == yaml.DocumentNode
}
