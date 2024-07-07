import { Writer } from "../structure"

export type FlagType = "None" | "Focus" | "Skip"

export interface FlagStat {
    focusCount: number
    skipCount: number
}

export interface Node {
    isLeaf(): boolean
    getFlag(): FlagType
    getChildren(): Node[]
    warning(message: string, stdout: Writer): void
}

export function extractSelectedLeaves(
    tree: Node,
    flagStat: FlagStat,
    stdout: Writer,
): Node[] {
    let focusedNodeArray = [] as Node[]
    findFocusedNodes(tree, focusedNodeArray, stdout)
    flagStat.focusCount = focusedNodeArray.length
    if (focusedNodeArray.length === 0) {
        focusedNodeArray.push(tree)
    }
    let leafArray = [] as Node[]
    focusedNodeArray.forEach((node) => {
        getLeaves(node, leafArray, flagStat)
    })
    return leafArray
}

function findFocusedNodes(
    node: Node,
    focusedNodeArray: Node[],
    stdout: Writer,
) {
    let flag = node.getFlag()
    if (flag == "Skip") {
        return
    }
    let initialLength = focusedNodeArray.length
    node.getChildren().forEach((child) => {
        findFocusedNodes(child, focusedNodeArray, stdout)
    })
    if (flag == "Focus") {
        if (focusedNodeArray.length > initialLength) {
            node.warning(
                "This node is marked as focused and it has focused descendents. The focus on this node will be ignored in favor of that of its descendents.",
                stdout,
            )
        } else {
            focusedNodeArray.push(node)
        }
    }
}

function getLeaves(node: Node, leafArray: Node[], flagStat: FlagStat) {
    if (node.getFlag() == "Skip") {
        flagStat.skipCount += 1
        return
    }
    node.getChildren().forEach((child) => {
        getLeaves(child, leafArray, flagStat)
    })
    if (node.isLeaf()) {
        leafArray.push(node)
    }
}
