# Code Overview

**`/go/specimen`**

- `file.go` is about local files and virtual files
- `flag.go` allows to read the flags on a YAML node
- `init.go` sets global objects and configurations
- `nodule.go` implements methods and functions related to the Nodule struct
- `run.go` implements the entry point to the library: "specimen.Run(...)"
- `structure.go` contains all the type declarations
- `testcontext.go` contains the methods specified on the testcontext
- `tree.go` implements the `focustree.Node` interface on the Nodule and NoduleRoot types

The `.InitializeTree()` method is run on all nodules of the tree.
`.Populate()` excludes the nodes carrying the "PENDING" flag.
