import { Scalar, YAMLMap, YAMLSeq } from "yaml"
import { FlagType } from "./focustree/focustree"

export type FailStatus = "Pristine" | "Failed" | "Aborted" | "Panicked"

export interface S {
    slabCount: number
    slabPassed: number
    slabFailed: number
    slabAborted: number
    slabPanicked: number
    failureReport: string[]
    status: FailStatus
}

export interface File {
    path: string
    content: string
}

export interface BoxFunction {
    (s: S, input: Record<string, any>): void
}

export interface Codebox {
    name: string
    boxFunction: BoxFunction
}

export type TreeRoot = Nodule[]

export type YAMLNode = Scalar | YAMLMap | YAMLSeq

export interface Nodule {
    file: File
    mapping: YAMLNode
    kind: "File" | "Node" | "Slab"
    location: string
    flag: FlagType
    name?: string
    children: Nodule[]
    codebox: Codebox
    input: Record<string, any>
    matrix: Record<string, any[]>
}
