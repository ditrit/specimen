package syaml

import (
	"fmt"
	"strconv"
	"strings"

	"gopkg.in/yaml.v3"
)

// Search through a yaml map for the entry associated to a given key and return the value. Returns nil if the key is missing
func (c *Config) MapTryGetValue(node *yaml.Node, key string) *yaml.Node {
	c.Resolve(&node)
	for k := 0; k < len(node.Content); k += 2 {
		if node.Content[k].Value == key {
			result := node.Content[k+1]
			c.Resolve(&result)
			return result
		}
	}
	return nil
}

// Search through a yaml map for the value associated to a given key. Panic if the key is missing
func (c *Config) MapGetValue(node *yaml.Node, key string) *yaml.Node {
	c.Resolve(&node)
	value := c.MapTryGetValue(node, key)
	if value == nil {
		panic(fmt.Errorf("could not find key %s", key))
	}
	return value
}

// Retrieve a string value from a yaml.Node
func (c *Config) GetString(node *yaml.Node) string {
	c.Resolve(&node)
	c.AssertIsString(node)
	return node.Value
}

// Retrieve an integer value from a yaml.Node
func (c *Config) GetInt(node *yaml.Node) int {
	c.Resolve(&node)
	c.AssertIsInteger(node)
	result, err := strconv.ParseInt(node.Value, 0, 32)
	if err != nil {
		panic(fmt.Errorf("integer parsing failed with message [%s]", err))
	}
	return int(result)
}

// MapAssertStringKeysAmong asserts the keys of the mapping node are all strings whose value is among the given slice of accepted values
func (c *Config) MapAssertStringKeysAmong(node *yaml.Node, acceptSlice []string) {
	c.Resolve(&node)
checking:
	for k := 0; k < len(node.Content); k += 2 {
		key := node.Content[k]
		if c.IsMerge(key) {
			c.MapAssertStringKeysAmong(node.Content[k+1], acceptSlice)
		} else {
			c.AssertIsString(key)
		}
		for _, acceptedKey := range acceptSlice {
			if key.Value == acceptedKey {
				continue checking
			}
		}
		panic(fmt.Errorf("unexpected key %s (should be among %s)", key.Value, acceptSlice))
	}
}

// Search through a yaml map for the entry associated to a given key and retrieve any kind of value, as interface{}
func (c *Config) MapGetAny(node *yaml.Node, key string) interface{} {
	return c.ExtractContent(c.MapGetValue(node, key))
}

// ExtractContent recurses through a tree of yaml.Node and produces a golang
// tree of map[string]interface{}, []interface{}, string and integer
func (c *Config) ExtractContent(node *yaml.Node) interface{} {
	c.Resolve(&node)
	switch node.Kind {
	case yaml.DocumentNode:
		return c.ExtractContent(node.Content[0])
	case yaml.ScalarNode:
		if node.Tag == "!!str" {
			return node.Value
		} else if node.Tag == "!!int" {
			result, err := strconv.ParseInt(node.Value, 0, 32)
			if err != nil {
				panic(fmt.Errorf("integer parsing failed with message [%s]", err))
			}
			return int(result)
		} else if node.Tag == "!!bool" {
			value := strings.ToLower(node.Value)
			if value == "true" || value == "on" || value[0] == 'y' {
				return true
			}
			return false
		} else if node.Tag == "!!null" {
			return nil
		}
		panic(fmt.Errorf("unknown scalar tag [%s]", node.Tag))
	case yaml.SequenceNode:
		result := make([]interface{}, len(node.Content))
		for k, v := range node.Content {
			result[k] = c.ExtractContent(v)
		}
		return result
	case yaml.MappingNode:
		result := make(map[string]interface{}, len(node.Content)/2)
		for k := 0; k < len(node.Content); k += 2 {
			key := node.Content[k]
			if c.IsMerge(key) {
				content := c.ExtractContent(node.Content[k+1]).(map[string]interface{})
				for k, v := range content {
					result[k] = v
				}
			} else {
				c.AssertIsString(key)
			}
			value := node.Content[k+1]
			result[key.Value] = c.ExtractContent(value)
		}
		return result
	case yaml.AliasNode:
		return c.ExtractContent(node.Alias)
	default:
		panic(fmt.Errorf("unknown node kind [%d]", node.Kind))
	}
}
