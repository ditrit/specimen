import * as yaml from "yaml"

import { deepEqual } from "./deepEqual"
import * as focustree from "./focustree/focustree"
import * as syaml from "./syaml"
import { Codebox, FailStatus, File, NoduleKind } from "./structure"

export class SpecimenContext {
    slabCount: number = 0
    slabPassed: number = 0
    slabFailed: number = 0
    slabAborted: number = 0
    slabThrew: number = 0
    failureReport: string[] = []

    status: FailStatus = "Pristine"
    failInfo: string[] = []

    fail(info: string) {
        this.status = "Failed"
        if (info.length > 0) {
            this.failInfo.push(info)
        }
    }

    abort(info: string) {
        this.status = "Aborted"
        if (info.length > 0) {
            this.failInfo.push(info)
        }
        throw new Error()
    }

    expectEqual<T>(value: T, wanted: T, context = "") {
        if (!deepEqual(value, wanted)) {
            if (context.length > 0) {
                context = `(${context}): `
            }
            this.fail(
                `${context}got <${typeof value}>${value} wanted <${typeof wanted}>${wanted}`,
            )
        }
    }
}

type TreeRootInterface = Nodule[]
export class TreeRoot
    extends Array
    implements TreeRootInterface, focustree.Node
{
    getChildren(): focustree.Node[] {
        return this
    }
    isLeaf(): boolean {
        return false
    }
    getFlag(): focustree.FlagType {
        return "None"
    }
    warning(info: string): void {
        console.warn("TreeRoot: %s", info)
    }
}

export class Nodule implements focustree.Node {
    public flag: focustree.FlagType = "Focus"
    public children: Nodule[] = []
    public location = ""
    public input: Record<string, any> = {}
    public matrix: Record<string, any[]> = {}
    public name: string = ""
    public codebox?: Codebox

    constructor(
        public file: File,
        public kind: NoduleKind,
        public lineCounter: yaml.LineCounter,
        public document: yaml.Document,
        public mapping: yaml.YAMLMap<string, any>,
    ) {}

    getChildren() {
        return this.children
    }
    isLeaf() {
        return this.children.length === 0
    }
    getFlag() {
        return this.flag
    }
    warning(info: string): void {
        console.warn("%s %s(%s): %s", this.kind, this.name, this.location, info)
    }

    initialize() {
        // location
        let { line, col } = this.lineCounter.linePos(
            this.mapping?.range?.[0] || 0,
        )
        this.location = `${this.file.path}:${line}:${col}`

        // flag
        this.flag = readFlag(this.mapping!)

        // name
        this.name = (this.mapping.get("name") as string) || ""

        // /\ content node (for children)
        let contentNode = syaml.resolve(
            this.mapping.get("content"),
            this.document,
        ) as yaml.Node
        if (contentNode === undefined) {
            let inputNode = syaml.resolve(
                this.mapping.get("input") as yaml.Node,
                this.document,
            )
            if (inputNode === undefined) {
                throw this.newError(
                    'the "content" entry and the "input" entry cannot be both absent from a nodule.',
                )
            }
            return
        }
        // content node processing
        syaml.assertIsSequence(contentNode)
        // kind
        if (contentNode.items.length > 0 && this.kind == "Slab") {
            this.kind = "Node"
        }
        // children
        contentNode.items.forEach((nodeOrAlias) => {
            let node = syaml.resolve(nodeOrAlias, this.document) as yaml.Node
            syaml.assertIsMap(node)
            let child = new Nodule(
                this.file,
                "Slab",
                this.lineCounter,
                this.document,
                node as any,
            )
            child.initialize()
            if (this.flag !== "Skip") {
                this.children.push(child)
            }
        })
        // \/ content node
    }

    populate(
        codeboxSet: Record<string, Codebox>,
        codebox: Codebox | undefined,
        input: Record<string, any>,
        matrix: Record<string, any[]>,
    ) {
        if (this.flag === "Skip") {
            return
        }
        // box
        let box = this.mapping.get("box")
        if (box !== undefined) {
            if (typeof box !== "string") {
                throw this.newError("the box entry must be a string")
            }
            if (codeboxSet[box]) {
                codebox = codeboxSet[box]
            } else {
                throw this.newError(`unknown codebox ${box}`)
            }
        }
        // input
        Object.entries(input).forEach(([key, value]) => {
            this.input[key] = value
        })
        let inputNode = this.mapping.get("input", true) as yaml.Node
        if (inputNode !== undefined) {
            syaml.assertIsMap(inputNode)
            let localInput = syaml.extractContent(inputNode, this.document)
            Object.entries(localInput).forEach(([key, value]) => {
                this.input[key] = value
            })
        }
        // matrix
        Object.entries(matrix).forEach(([key, value]) => {
            this.matrix[key] = value
        })
        let matrixNode = this.mapping.get("matrix", true) as yaml.Node
        if (matrixNode !== undefined) {
            syaml.assertIsMap(matrixNode)

            matrixNode.items.forEach(({ key, value }) => {
                let k = key as yaml.Node
                syaml.assertIsScalar(k)
                let sequence = value as yaml.Node
                syaml.assertIsSequence(sequence)
                if (typeof k.value !== "string") {
                    throw this.newError(
                        `expected string keys but got ${typeof key}`,
                    )
                }
                this.matrix[k.value] = sequence.items.map((value) => {
                    return syaml.extractContent(value, this.document)
                })
            })
        }

        // slab case
        if (this.children.length === 0) {
            if (codebox === undefined) {
                throw this.newError("no box declared down to this slab")
            }
            this.codebox = codebox
            if (inputNode === undefined) {
                throw this.newError("the input entry is mandatory on slabs")
            }

            if (Object.keys(this.matrix).length > 0) {
                let noduleArray = [this.clone()]
                noduleArray[0].flag = "None"
                noduleArray[0].matrix = {}
                Object.entries(this.matrix).forEach(([key, valueArray]) => {
                    noduleArray = multiplyArray(key, valueArray, noduleArray)
                })
                this.children = noduleArray
                this.children.forEach((child) => {
                    Object.entries(this.input).forEach(([key, value]) => {
                        child.input[key] = value
                    })
                })
            }
            // all good with the current slab
            return
        }

        // node case: populating children
        let validChildren: Nodule[] = []
        this.children.forEach((child) => {
            try {
                child.populate(codeboxSet, codebox, this.input, this.matrix)
                validChildren.push(child)
            } catch (e) {
                console.error(e)
            }
        })

        if (validChildren.length === 0) {
            throw this.newError("no valid children")
        }

        this.children = validChildren
        return
        // end of populate()
    }

    runCodebox(context: SpecimenContext) {
        try {
            this.codebox!.boxFunction(context, this.input)
        } catch (e) {
            if (context.status === "Aborted") {
                return
            }

            let report = new Error().stack
            let info = `\n>   ` + report?.replace("\n", "\n>   ")
            info += "\n" + e

            context.status = "Threw"
        }
    }

    clone() {
        let other = new Nodule(
            this.file,
            this.kind,
            this.lineCounter,
            this.document,
            this.mapping,
        )
        other.location = this.location
        other.flag = this.flag
        other.name = this.name
        other.children = this.children
        other.codebox = this.codebox
        other.input = {}
        Object.entries(this.input).forEach(([key, value]) => {
            other.input[key] = value
        })
        other.matrix = this.matrix
        return other
    }

    newError(message: string) {
        return new Error(`(${this.location}): ` + message)
    }

    resolve(node: yaml.Node) {
        return syaml.resolve(node, this.document)
    }
}

function readFlag(node: yaml.YAMLMap<string, any>): focustree.FlagType {
    let flagEntry = node.get("flag")
    let flag: focustree.FlagType = "None"

    if (flagEntry === undefined) {
        return flag
    }

    let name = ""
    let both = false
    let sentence = (flagEntry as string).split(" ")
    sentence.forEach((word) => {
        if (word === "FOCUS") {
            if (flag === "Skip") {
                both = true
            }
            flag = "Focus"
            name = word
        } else if (word === "PENDING") {
            if (flag === "Focus") {
                both = true
            }
            flag = "Skip"
            name = word
        } else if (word.toUpperCase() === word) {
            console.warn(
                `Unrecognized all uppercase flag \"${word}\". It has been ignored.`,
            )
        }
    })
    if (both) {
        console.warn(
            `Both FOCUS and PENDING have been found among the flags of a node. ${name} has been kept.`,
        )
    }
    return flag
}

function multiplyArray(
    key: string,
    valueArray: unknown[],
    noduleArray: Nodule[],
) {
    let resultArray: Nodule[] = []
    noduleArray.forEach((nodule) => {
        valueArray.forEach((value, k) => {
            let other = nodule.clone()
            other.input[key] = value
            other.location = `${other.location}(${key}[${k}])`
            resultArray.push(other)
        })
    })
    return resultArray
}
