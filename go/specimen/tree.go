package specimen

import (
	"log"

	"github.com/ditrit/specimen/go/specimen/focustree"
)

// This file implement the focustree.Node interface for the NoduleRoot and for Nodules

// NoduleRoot implements focustree.Node

func (t NoduleRoot) GetChildren() (children []focustree.Node) {
	for _, c := range t {
		children = append(children, c)
	}
	return
}

func (NoduleRoot) IsLeaf() bool {
	return false
}

func (NoduleRoot) GetFlag() focustree.FlagType {
	return focustree.None
}

func (NoduleRoot) Warning(info string) {
	log.Printf("Warning: NoduleRoot: %s\n", info)
}

// Nodule implements focustree.Node

func (n Nodule) GetChildren() (children []focustree.Node) {
	// convert []Nodule to []focustree.Node
	for _, c := range n.Children {
		children = append(children, c)
	}
	return
}

func (n Nodule) IsLeaf() bool {
	return !n.HasContentKey
}

func (n Nodule) GetFlag() focustree.FlagType {
	return n.Flag
}

func (n Nodule) Warning(info string) {
	log.Printf("Warning(%s): %s\n", n.GetLocation(), info)
}
