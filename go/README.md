# Code Overview

**`/go/specimen`**

- `structure.go` is about the TreeRoot, Nodule, Nodule, Nodule and how to create them
- `file.go` is about local files and virtual files
- `run.go` implements the entry point to the library: "specimen.Run(...)"
- `tree.go` implements the `focustree.Node` interface on the types defined in `structure.go`

The .Initialize() method is run on all nodules of the tree, while the
.Populate() only acts on nodes that are scheduled to run.