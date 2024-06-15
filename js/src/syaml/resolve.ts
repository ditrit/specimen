import * as yaml from "yaml"

export function resolve(node: unknown, document: yaml.Document) {
    if (yaml.isAlias(node)) {
        return node.resolve(document)!
    }
    return node
}
