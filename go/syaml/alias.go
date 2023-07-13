package syaml

import "gopkg.in/yaml.v3"

type Config struct {
	HandleAlias          bool
	AliasResolutionDepth int
}

func NewSyaml(handleAlias bool, aliasResolutionDepth int) (c Config) {
	c.HandleAlias = handleAlias
	c.AliasResolutionDepth = aliasResolutionDepth
	return
}

func (c *Config) Resolve(node **yaml.Node) {
	if c.HandleAlias {
		counter := 0
		for (**node).Kind == yaml.AliasNode {
			*node = (**node).Alias
			if counter > c.AliasResolutionDepth {
				panic("Cannot resolve alias: resolution depth exceeded")
			}
		}
	}
}
