import dataclasses
from typing import Any, Dict, List
import yaml

from python.file import File
from python.flag import read_flag
import python.focustree as focustree
import python.syaml as syaml


@dataclasses.dataclass
class Nodule:
    node: yaml.Node
    flag: focustree.Flag
    be_leaf: bool
    file_path: str
    data_matrix: Dict[str, str]
    children: List["Nodule"]

    def is_leaf(self) -> bool:
        return self.be_leaf

    def get_flag(self) -> focustree.Flag:
        return self.flag

    def get_children(self) -> List["Nodule"]:
        return self.children

    def get_value(self) -> "Nodule":
        return self

    def warning(self, message: str, stdout: Any):
        print(f"Warning({self.get_location()}): {message}", file=stdout)

    @staticmethod
    def parse_file(file: File) -> "Nodule":
        document = yaml.compose(file.content)
        if not syaml.is_mapping(document):
            raise ValueError("the root of the YAML test data file must be a mapping")

        nodule = Nodule(
            node=document,
            flag=focustree.Flag.NONE,
            be_leaf=True,
            file_path=file.path,
            data_matrix=dict(file_path=[file.path]),
            children=[],
        )

        nodule.initialize_tree()

        return nodule

    def get_location(self):
        return f"{self.file_path}:{self.node.start_mark.line+1}:{self.node.start_mark.column+1}"

    def initialize_tree(self):
        if not syaml.is_mapping(self.node):
            raise ValueError("the content descendant nodes must be yaml mappings")

        # flag
        self.flag = read_flag(self.node)
        if self.flag == focustree.Flag.SKIP:
            return

        # content node (for children)
        content_node = syaml.map_try_get_value(self.node, "content")
        if content_node is not None:
            self.be_leaf = False
            if syaml.is_sequence(content_node):
                for node in content_node.value:
                    nodule = Nodule(
                        node=node,
                        flag=focustree.Flag.NONE,
                        be_leaf=True,
                        file_path=self.file_path,
                        data_matrix=dict(),
                        children=[],
                    )
                    nodule.initialize_tree()
                    self.children.append(nodule)
            else:
                raise ValueError(
                    "the value associated with the content keyword must be a sequence of mappings"
                )

    def populate(self, data_matrix: Dict[str, List[str]]):
        if self.flag == focustree.Flag.SKIP:
            return

        self.data_matrix = dict(data_matrix)

        for key, value in self.node.value:
            if not syaml.is_string(key):
                raise ValueError("the keys of the mapping node must be strings")

            if key.value in ["flag", "content", "about"]:
                continue

            if not syaml.is_string(value) and not syaml.is_sequence(value):
                raise ValueError(
                    f"the values of mapping nodes must be strings or sequences of strings (key: {key.value})"
                )

            value_list = (
                [value.value]
                if syaml.is_string(value)
                else [v.value for v in value.value]
            )

            self.data_matrix[key.value] = value_list

        for child in self.children:
            child.populate(self.data_matrix)

    def __iter__(self):
        reversed_key_list = [*reversed(self.data_matrix.keys())]
        reversed_size_list = [len(self.data_matrix[key]) for key in reversed_key_list]
        reversed_index_list = [0] * len(reversed_size_list)

        combination = { key: self.data_matrix[key][0] for key in reversed_key_list}
        yield combination

        while True:
            n = -1
            while n < 0 or reversed_index_list[n] == 0:
                n += 1
                if n >= len(reversed_index_list):
                    return
                reversed_index_list[n] += 1
                reversed_index_list[n] %= reversed_size_list[n]
                combination[reversed_key_list[n]] = self.data_matrix[reversed_key_list[n]][reversed_index_list[n]]
            yield combination
