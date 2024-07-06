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

type FlagStat struct {
	FocusCount int
	SkipCount  int
}

// ExtractSelectedLeaves goes through the tree and find leaves
// whose data should be processed according to the flag
// (FOCUS, PENDING)
func ExtractSelectedLeaves(tree Node, flagStat *FlagStat) []Node {
	focusedNodeSlice := []Node{}
	findFocusedNodes(tree, &focusedNodeSlice)
	flagStat.FocusCount = len(focusedNodeSlice)
	if len(focusedNodeSlice) == 0 {
		focusedNodeSlice = []Node{tree}
	}
	leafSlice := []Node{}
	for _, node := range focusedNodeSlice {
		getLeaves(node, &leafSlice, flagStat)
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
func getLeaves(node Node, leafSlice *[]Node, flagStat *FlagStat) {
	if node.GetFlag() == Skip {
		flagStat.SkipCount += 1
		return
	}
	for _, child := range node.GetChildren() {
		getLeaves(child, leafSlice, flagStat)
	}
	if node.IsLeaf() {
		*leafSlice = append(*leafSlice, node)
	}
}
