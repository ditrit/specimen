import { Scalar, YAMLMap, YAMLSeq } from "yaml"

export type FailStatus = "Pristine" | "Failed" | "Aborted" | "Threw"

export type NoduleKind = "File" | "Node" | "Slab"

export interface File {
    path: string
    content: string
}

export type YAMLNode = Scalar | YAMLMap | YAMLSeq

export interface Writer {
    write(s: string): void
}
