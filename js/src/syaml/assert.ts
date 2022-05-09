import * as yaml from "yaml"

export function assertIsSequence(
    node: yaml.Node,
): asserts node is yaml.YAMLSeq<unknown> {
    if (!yaml.isSeq(node)) {
        throw new TypeError(`expected a yaml sequence`)
    }
}

export function assertIsMap<T extends yaml.YAMLMap<unknown, unknown>>(
    node: yaml.Node,
): asserts node is T {
    if (!yaml.isMap(node)) {
        throw new TypeError(`expected a yaml map`)
    }
}

export function assertIsScalar(
    node: yaml.Node,
): asserts node is yaml.Scalar<unknown> {
    if (!yaml.isScalar(node)) {
        throw new TypeError(`expected a yaml scalar`)
    }
}
