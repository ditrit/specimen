# Specimen

_Yaml-based data-driven testing_

Specimen is a data-driven testing library as well as a yaml data format. It enforces separation between the _feature being tested_ and the _data_ used for testing.

It comes with a **Golang**, a **Python** and a **JS** implementation for loading the data, checking its format, running your _test box_ and comparing the result with the expected one.

It supports using the `FOCUS` and `PENDING` flags in the data tree to run only parts of the test data.

## Overview

![overview of the way the specimen library works](doc/specimen-overview.svg)

- A **Test Box** is a user-defined function passed to `specimen.run`. It serves as an adaptator between Specimen and the user code being tested. As such, it prepares the data for testing, runs the code being tested and performs the checks on the code result once it has finished.
- A **Slab** is a leaf of the yaml files data tree that Specimen processes.
- A **Tile** is a chunk of data to be loaded into a test box. When test matrices are used, a slab will produce multiple tiles.

## Getting started with Specimen in Golang

To get started, create a directory `it/` and the three files `it.go` `it_test.go` and `it_testdata.yaml`. For each file, copy the content of linked section. Install the dependencies and finally run `go test` in the `it/` directory:

```sh
mkdir it
cd it
touch it.go it_test.go it_testdata.yaml
```

Fill each of the three files with the code found in this README:

- For `it.go` see [Example package code](#example-package-code)
- For `it_test.go` see [Code box](#test-box)
- For `it_testdata.yml` see [Yaml Data](#yaml-data)

Finally, run:

```sh
go mod init it
go mod tidy
go test
```

You should get an output similar to this one:

```
TestIt:
Ran 4 tiles in 0s
SUCCESS -- 4 Passed | 0 Failed | 0 Aborted | 0 Panicked
PASS
ok      it      0.646s
```

`/!\` If `go mod tidy` fails to find the specimen module, you may need to clean the go cache: `go clean -cache`, `go clean -modcache`.

## Yaml Data

The yaml data file looks like this:

```yaml
box: zoo
content:
  - content:
      - name: horse
        animal: horse
        expected_result: horse
      - flag: PENDING
        name: parasprite # this slab will be ignored
        animal: parasprite
  - name: zebra
    animal: zebra
    expected_result: horse zebra
  - name: animal matrix
    animal: [mouse, cat, dog]
  - name: matrix check
    animal: wolf
    expected_result: horse zebra mouse cat dog wolf
```

## Test box

A test box is an **adapter** between the parsed data and the library code being tested. It takes as input the testing context `s` and the **input map**. A test box looks like this:

```go
package it_test

import (
	"testing"

	specimen "github.com/ditrit/specimen/go"
	"github.com/ditrit/specimen/test/zoo"
)


func TestIt(t *testing.T) {
    specimen.Run(
        t,
        func(s *specimen.S, input specimen.Dict) {
            animal := input["animal"]
            expected := input["expected_result"]

            if len(animal) > 0 {
                output := zoo.AddAnimal(animal)

                if len(expected) > 0 {
                    s.ExpectEqual(output, expected, "result comparison")
                }
            }
		    },
        []specimen.File{
            specimen.ReadLocalFile("it_testdata.yaml"),
        },
    )
}
```

## Example package code

```go
package it

import "strings"

type Zoo []string

var zoo Zoo

func AddAnimal(animal string) string {
	zoo = append(zoo, animal)
	return strings.Join(zoo, " ")
}
```

## Running the tests

```sh
# golang
go test ./test/counter ./test/danger ./test/novel ./test/nullValue ./test/zoo
# or
go test test/counter/counter_test.go
go test test/danger/danger_test.go
go test test/novel/novel_test.go
go test test/nullValue/nullValue_test.go
go test test/zoo/zoo_test.go

# python (v3.10 or a future version)
cd python
python -m poetry install
cd ..
python test/counter/counter_test.py
python test/novel/novel_test.py
python test/novel/nullValue_test.py
python test/zoo/zoo_test.py

# js
cd js
yarn install
# yarn parcel build src/index.ts
yarn tsc
cd ..
node test/counter/counter_test.js
node test/novel/novel_test.js
node test/novel/nullValue_test.js
node test/zoo/zoo_test.js
```

## Yaml Schema

The content of a yaml test data file must match the `main` rule of the lidy schema below:

```yaml
main: nodule

# scalar is any yaml scalar. Please note that whatever the type is, it will
# appear as a string in the tile Dict.
scalar:
  _oneOf:
    - string
    - int
    - float
    - boolean
    - nullType

# A tip can be a scalar or a list of scalars. In the case of a list, all the
# combination of values taken from this list and lists of other parameters will
# be generated and run. This produces the effect of a test matrix.
tip:
  _oneOf:
    - scalar
    - _listOf: scalar

nodule:
  _mapFacultative:
    # `content` contains all the children of the current nodule. Nodes which
    # contain a content entry are seen as tree nodes, while nodes which do
    # not contain it are seen as leaves
    content:
      - _listOf: nodule
    # The flag can be either "PENDING", "FOCUS" or .not specified.
    # The PENDING flag tells the engine to skip the node and all its decendants.
    # The FOCUS flag tells the engine to skip all the OTHER nodes; the one which
    # do not have the flag "FOCUS".
    flag:
      _in: ["PENDING", "FOCUS"]
    # `about` can contain any data: it will not be checked by the parser, and it
    # will not appear in the data passed to the box function
    about: any
  # all the entries of the mapping will be added to the descendant slabs of
  # this nodule and then passed to the code box, except for the `content` entry
  _mapOf: { string: tip }
# Besides all the keys that are found in the yaml, the test box will be passed
# an argument "filepath" which contains the path to the yaml file, as specified
# to Lidy.
```
