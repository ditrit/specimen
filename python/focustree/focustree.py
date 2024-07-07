import dataclasses
import enum
from typing import Any, List, Protocol, TypeVar


class Flag(enum.Enum):
    NONE = 0
    FOCUS = 1
    SKIP = 2


@dataclasses.dataclass
class FlagStat:
    focus_count: int
    skip_count: int

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
    def warning(self, message: str, stdout: Any): ...


def extract_selected_leaves(tree: Node, flag_stat: FlagStat, stdout: Any) -> List[TValue]:
    focused_node_list = []
    find_focused_nodes(tree, focused_node_list, stdout)
    flag_stat.focus_count = len(focused_node_list)
    if len(focused_node_list) == 0:
        focused_node_list = [tree]
    leaf_slice = []
    for node in focused_node_list:
        get_leaf_values(node, leaf_slice, flag_stat)
    return leaf_slice


def find_focused_nodes(node: Node, focused_node_slice: List[Node], stdout: Any):
    flag = node.get_flag()
    if flag == Flag.SKIP:
        return
    initial_length = len(focused_node_slice)
    for child in node.get_children():
        find_focused_nodes(child, focused_node_slice, stdout)
    if flag == Flag.FOCUS:
        if len(focused_node_slice) > initial_length:
            node.warning(
                "This node is marked as focused and it has focused descendents. "
                "The focus on this node will be ignored in favor of that of its descendents.",
                stdout=stdout,
            )
        else:
            focused_node_slice.append(node)


def get_leaf_values(node: Node, leaf_slice: List[TValue], flag_stat: FlagStat):
    if node.get_flag() == Flag.SKIP:
        flag_stat.skip_count += 1
        return
    if node.is_leaf():
        leaf_slice.append(node.get_value())
    for child in node.get_children():
        get_leaf_values(child, leaf_slice, flag_stat)
