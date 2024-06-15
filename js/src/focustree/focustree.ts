export type FlagType = "None" | "Focus" | "Skip"

export interface Node {
    isLeaf(): boolean
    getFlag(): FlagType
    getChildren(): Node[]
    warning(message: string): void
}

export function extractSelectedLeaves(tree: Node): Node[] {
    let focusedNodeArray = [] as Node[]
    findFocusedNodes(tree, focusedNodeArray)
    if (focusedNodeArray.length === 0) {
        focusedNodeArray.push(tree)
    }
    let leafArray = [] as Node[]
    focusedNodeArray.forEach((node) => {
        getLeaves(node, leafArray)
    })
    return leafArray
}

function findFocusedNodes(node: Node, focusedNodeArray: Node[]) {
    let flag = node.getFlag()
    if (flag == "Skip") {
        return
    }
    let initialLength = focusedNodeArray.length
    node.getChildren().forEach((child) => {
        findFocusedNodes(child, focusedNodeArray)
    })
    if (flag == "Focus") {
        if (focusedNodeArray.length > initialLength) {
            node.warning(
                "This node is marked as focused and it has focused descendents. The focus on this node will be ignored in favor of that of its descendents.",
            )
        } else {
            focusedNodeArray.push(node)
        }
    }
}

function getLeaves(node: Node, leafArray: Node[]) {
    if (node.getFlag() == "Skip") {
        return
    }
    node.getChildren().forEach((child) => {
        getLeaves(child, leafArray)
    })
    if (node.isLeaf()) {
        leafArray.push(node)
    }
}
