import { Scalar, YAMLMap, YAMLSeq } from "yaml"
import { Nodule, SpecimenContext } from "./tree"

export type FailStatus = "Pristine" | "Failed" | "Aborted" | "Threw"

export type NoduleKind = "File" | "Node" | "Slab"

export interface File {
    path: string
    content: string
}

export interface BoxFunction {
    (s: SpecimenContext, input: Record<string, any>): void
}

export interface Codebox {
    name: string
    boxFunction: BoxFunction
}

export type YAMLNode = Scalar | YAMLMap | YAMLSeq
