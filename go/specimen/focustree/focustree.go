package focustree

type FlagType int

const (
	None FlagType = iota
	Focus
	Skip
)

type Node interface {
	GetFlag() FlagType
	GetChildren() []Node
	IsLeaf() bool
	Warning(info string)
}

// ExctractSelectedLeaves goes through the tree and find leaves
// whose data should be processed according to the flag
// (FOCUS, SKIP)
func ExctractSelectedLeaves(tree Node) []Node {
	focusedNodeSlice := []Node{}
	findFocusedNodes(tree, &focusedNodeSlice)
	if len(focusedNodeSlice) == 0 {
		focusedNodeSlice = append(focusedNodeSlice, tree)
	}
	leafSlice := []Node{}
	for _, node := range focusedNodeSlice {
		getLeaves(node, &leafSlice)
	}
	return leafSlice
}

// findFocusedNodes goes through the tree and adds focused nodes to a slice.
// If a node with focused descendents is found to be marked as focused
// itself, it is NOT added as a focused node and a warning is issued.
func findFocusedNodes(node Node, focusedNodeSlice *[]Node) {
	flag := node.GetFlag()
	if flag == Skip {
		return
	}
	initialLength := len(*focusedNodeSlice)
	for _, child := range node.GetChildren() {
		findFocusedNodes(child, focusedNodeSlice)
	}
	if flag == Focus {
		if len(*focusedNodeSlice) > initialLength {
			node.Warning("This node is marked as focused and it has focused descendents. " +
				"The focus on this node will be ignored in favor of that of its descendents.")
		} else {
			*focusedNodeSlice = append(*focusedNodeSlice, node)
		}
	}
}

// getLeaves adds all the leaves below the given node to a slice.
func getLeaves(node Node, leafSlice *[]Node) {
	if node.GetFlag() == Skip {
		return
	}
	for _, child := range node.GetChildren() {
		getLeaves(child, leafSlice)
	}
	if node.IsLeaf() {
		*leafSlice = append(*leafSlice, node)
	}
}
