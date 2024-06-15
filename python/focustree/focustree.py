import enum
from typing import Protocol, TypeVar


class Flag(enum.Enum):
    NONE = 0
    FOCUS = 1
    SKIP = 2


TValue = TypeVar("TValue")


class Node(Protocol):
    def is_leaf(
        self,
    ) -> bool: ...
    def get_flag(
        self,
    ) -> Flag: ...
    def get_children(
        self,
    ) -> list["Node"]: ...
    def get_value(
        self,
    ) -> TValue: ...
    def warning(self, message: str): ...


def extract_selected_leaves(tree: Node) -> list[TValue]:
    focused_node_slice = []
    find_focused_nodes(tree, focused_node_slice)
    if len(focused_node_slice) == 0:
        focused_node_slice = [tree]
    leaf_slice = []
    for node in focused_node_slice:
        get_leaf_values(node, leaf_slice)
    return leaf_slice


def find_focused_nodes(node: Node, focused_node_slice: list[Node]):
    flag = node.get_flag()
    if flag == Flag.SKIP:
        return
    initial_length = len(focused_node_slice)
    for child in node.get_children():
        find_focused_nodes(child, focused_node_slice)
    if flag == Flag.FOCUS:
        if len(focused_node_slice) > initial_length:
            node.warning(
                "This node is marked as focused and it has focused descendents. "
                "The focus on this node will be ignored in favor of that of its descendents."
            )
        else:
            focused_node_slice.append(node)


def get_leaf_values(node: Node, leaf_slice: list[TValue]):
    if node.get_flag() == Flag.SKIP:
        return
    if node.is_leaf():
        leaf_slice.append(node.get_value())
    for child in node.get_children():
        get_leaf_values(child, leaf_slice)
