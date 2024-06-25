import * as yaml from "yaml"

import * as focustree from "./focustree/focustree"
import * as syaml from "./syaml"
import { File, YAMLNode } from "./structure"
import { readFlag } from "./flag"

export function parseFileIntoNodule(file: File): Nodule {
    let lineCounter = new yaml.LineCounter()
    let document = yaml.parseDocument(file.content, { lineCounter })
    let mapping = document.contents
    if (!yaml.isMap(mapping)) {
        throw new Error("the root of the YAML test data file must be a mapping")
    }

    let nodule = new Nodule(
        mapping as unknown as yaml.YAMLMap<yaml.Scalar, any>,
        "None",
        true,
        file.path,
        { filePath: [file.path] },
        [],
        lineCounter,
    )

    nodule.initializeTree()

    return nodule
}

export class Nodule implements focustree.Node {
    constructor(
        public node: yaml.YAMLMap<any, YAMLNode>,
        public flag: focustree.FlagType,
        public beLeaf: boolean,
        public filePath: string,
        public dataMatrix: Record<string, string[]>,
        public children: Nodule[],
        public lineCounter: yaml.LineCounter,
    ) {}

    isLeaf() {
        return this.children.length === 0
    }
    getFlag() {
        return this.flag
    }
    getChildren() {
        return this.children
    }
    warning(message: string): void {
        console.warn("Warning(%s): %s", this.getLocation(), message)
    }

    getLocation() {
        let { line, col } = this.lineCounter.linePos(this.node?.range?.[0] || 0)
        return `${this.filePath}:${line}:${col}`
    }

    initializeTree() {
        if (!yaml.isMap(this.node)) {
            throw new Error(
                "the content descendant nodes must be yaml mappings",
            )
        }
        // flag
        this.flag = readFlag(this.node)
        if (this.flag === "Skip") {
            return
        }

        // content node (for children)
        let contentNode = this.node.get("content")
        if (contentNode !== undefined) {
            this.beLeaf = false
            if (yaml.isSeq(contentNode)) {
                let cn = contentNode as yaml.YAMLSeq<yaml.YAMLMap<string, any>>
                this.children = cn.items.map((node) => {
                    let nodule = new Nodule(
                        node,
                        "None",
                        true,
                        this.filePath,
                        this.dataMatrix,
                        [],
                        this.lineCounter,
                    )
                    nodule.initializeTree()
                    return nodule
                })
            } else {
                throw new Error(
                    "the value associated with the content keyword must be a sequence of mappings",
                )
            }
        }
    }

    populate(dataMatrix: Record<string, string[]>) {
        ;``
        if (this.flag === "Skip") {
            return
        }

        this.dataMatrix = { ...dataMatrix }

        this.node.items.forEach(({ key, value }) => {
            if (!syaml.isString(key)) {
                throw new Error("the keys of the mapping must be strings")
            }

            if (["flag", "content", "about"].includes(key.value)) {
                return
            }

            if (!syaml.isString(value!) && !yaml.isSeq(value)) {
                throw new Error(
                    `the values of the mapping must be strings or sequences of strings (key: ${key.value})`,
                )
            }

            let valueArray = yaml.isScalar(value)
                ? [(value as yaml.Scalar<string>).value]
                : (value!.items as YAMLNode[]).map((item) => {
                      if (!syaml.isString(item)) {
                          throw new Error(
                              `the values of the mapping must be strings or sequences of strings (key: ${
                                  key.value
                              }, subitem value: ${
                                  (item as yaml.Scalar).value
                              })`,
                          )
                      }
                      return (item as yaml.Scalar<string>).value
                  })

            this.dataMatrix[key.value] = valueArray
        })

        this.children.forEach((child) => {
            child.populate(this.dataMatrix)
        })
        // end of populate()
    }

    *iterateDataMatrix() {
        let length = Object.keys(this.dataMatrix).length

        let reversedKeyArray = Object.keys(this.dataMatrix).reverse()

        let totalCombinations = 1
        let sizeArray: number[] = []
        reversedKeyArray.forEach((key) => {
            let size = this.dataMatrix[key].length
            totalCombinations *= size
            sizeArray.push(totalCombinations)
        })

        let indexArray = Array.from({ length }, () => 0)

        let combination: Record<string, string> = {}

        reversedKeyArray.forEach((key) => {
            combination[key] = this.dataMatrix[key][0]
        })

        yield combination
        for (let index = 1; index < totalCombinations; index++) {
            for (let k = 0; k < length; k++) {
                let size = sizeArray[k]
                let nonZero = index % size > 0
                // bump the index
                indexArray[k] += 1
                indexArray[k] %= size
                if (nonZero) {
                    // update the combination entry corresponding to the identified key
                    let key = reversedKeyArray[k]
                    combination[key] = this.dataMatrix[key][indexArray[k]]
                    yield combination
                    break
                }
            }
        }
        return
    }
}
