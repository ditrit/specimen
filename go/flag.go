package specimen

import (
	"log"
	"strings"

	"github.com/ditrit/specimen/go/focustree"
	"gopkg.in/yaml.v3"
)

func readFlag(flagNode *yaml.Node) (flag focustree.FlagType) {
	syc.AssertIsString(flagNode)

	// flagName is used for printing warning(s) if needed
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
