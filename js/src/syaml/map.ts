import * as yaml from "yaml"
import { resolve } from "./resolve"

export function extractContent(nodeOrAlias: unknown, document: yaml.Document) {
    let node = resolve(nodeOrAlias, document)
    if (yaml.isSeq(node)) {
        return node.items.map((child) => extractContent(child, document))
    } else if (yaml.isMap(node)) {
        return Object.fromEntries(
            node.items.map(({ key, value }) => {
                return [key as string, extractContent(value, document)]
            }),
        )
    } else if (yaml.isScalar(node)) {
        return node.value
    } else {
        return node
    }
}
