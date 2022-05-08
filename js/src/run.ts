import { Codebox, Context, File } from "./structure"

export interface BoxFunction {
    (s: Context, input: Record<string, any>): void
}

export function makeCodeboxSet(codeboxMap: Record<string, BoxFunction>): Record<string, Codebox> {}

export function run(codeboxSet: Record<string, Codebox>, dataFileArray: File[]) {}
