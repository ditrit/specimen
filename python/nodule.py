import dataclasses
from typing import Dict, List
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

    def get_children(self):
        return self.children

    def get_value(self):
        return self

    def warning(self, info: str) -> focustree:
        print("Warning: %s %s(%s): %s" % (self.kind, self.name, self.location, info))

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
            data_matrix=dict(file_path=file.path),
            children=[],
        )

        nodule.initialize_tree()

        return nodule

    def get_location(self):
        return f"{self.file_path}:{self.node.start_mark.line+1}:{self.node.start_mark.column+1}"

    def initialize_tree(self):
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
                        children=[],
                        data_matrix=dict(),
                    )
                    nodule.initialize_tree()
                    self.children.append(nodule)

    def populate(self, data_matrix: Dict[str, str]):
        if not syaml.is_mapping(self.node):
            raise ValueError("the content descendant nodes must be yaml mappings")

        if self.flag == focustree.Flag.SKIP:
            return

        self.data_matrix = dict(data_matrix)

        for key, value in self.node.value:
            if not syaml.is_string(key):
                raise ValueError("the keys of the mapping node must be strings")

            if key.value == "flag" or key.value == "content":
                continue

            if not syaml.is_string(value) and not syaml.is_sequence(value):
                raise ValueError(
                    f"the values of mapping nodes must be strings or sequences (key: {key.value})"
                )

            value_list = [value.value] if syaml.is_string(value) else value.value

            self.data_matrix[key.value] = value_list

        for child in self.children:
            child.populate(self.data_matrix)

    def __iter__(self):
        length = len(self.data_matrix)

        reversed_key_array = [*self.data_matrix.keys()][::-1]
        total_combinations = 1
        size_array = []
        for key in reversed_key_array:
            size = len(self.data_matrix[key])
            total_combinations *= size
            size_array.append(total_combinations)

        index_array = [0] * length

        combination = dict()

        for key in reversed_key_array:
            combination[key] = self.data_matrix[key][0]

        yield combination
        for index in range(1, total_combinations):
            for k, key in enumerate(reversed_key_array):
                if index % size_array[k] == 0:
                    index_array[k] += 1
                    index_array[k] %= size_array[k]
                else:
                    # bump the identified index
                    index_array[k] += 1
                    index_array[k] %= size_array[k]

                    # update the combination entry corresponding to the identified key
                    combination[key] = self.data_matrix[key][index_array[k]]
                    break
            yield combination
        return
