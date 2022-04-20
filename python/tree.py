import yaml

import python.syaml as syaml
import python.focustree as focustree


class TreeRoot(list["Nodule"]):
    def get_children(self) -> list[focustree.Node]:
        children = []
        for c in self:
            children.append(c)
        return list(self)

    def is_leaf(self) -> bool:
        return False

    def get_flag(self) -> focustree.Flag:
        return focustree.Flag.NONE

    def warning(info: str):
        print("Warning: TreeRoot: %s" % info)


class Nodule:
    def __init__(
        self,
        file,
        # Kind is one of "File", "Node", "Slab" -- Kind is used in error reports
        kind,
        mapping=None,
    ):
        self.file = file
        self.mapping = mapping
        self.kind = kind
        # Location is a clickable link to the beginning of the nodule
        self.location = ""
        self.flag = focustree.Flag.NONE
        # Name is an indicative name for the nodule
        self.name = ""
        self.children = []
        self.codebox = None
        self.input = {}
        self.matrix = []

    def get_children(self):
        return self.children

    def is_leaf(self) -> bool:
        return len(self.children) == 0

    def get_flag(self) -> focustree.Flag:
        return self.flag

    def warning(info: str) -> focustree:
        print("Warning: %s %s(%s): %s" % (self.kind, self.name, self.location, info))

    def initialize_file(self):
        self.mapping = yaml.compose(self.file.content)
        if not syaml.is_mapping(self.mapping):
            raise RuntimeError("the root of the yaml document is must be a yaml mapping")
        self.initialize()

    def initialize(self):
        # location
        self.location = f"{self.file.path}:{self.mapping.start_mark.line}:{self.mapping.start_mark.column}"

        # flag
        self.flag = read_flag(self.mapping)

        # name
        name_node = syaml.map_try_get_value(self.mapping, "name")
        if name_node is not None:
            self.name = name_node.value

        # content node (for children)
        content_node = syaml.map_try_get_value(self.mapping, "content")
        if content_node is None:
            if syaml.map_try_get_value(self.mapping, "input") is None:
                raise ValueError("the \"content\" entry and the \"input\" entry cannot be both absent from a nodule.")
            return

        # /\ content node processing
        if not syaml.is_sequence(content_node):
            raise TypeError("the \"content\" value must be a yaml sequence")
        # kind
        if syaml.is_sequence(content_node) and len(content_node.value) > 0 and self.kind == "Slab":
            self.kind = "Node"

        # children
        for node in content_node.value:
            child = Nodule(file=self.file, kind="Slab", mapping=node)
            child.initialize()
            if self.flag != focustree.Flag.SKIP:
                self.children.append(child)
        # \/ content node processing


    def populate(self, codebox_set, codebox=None, input_data=None, matrix=None):
        if self.flag == focustree.Flag.SKIP:
            return
        
        # box
        box_node = syaml.map_try_get_value(self.mapping, "box")
        if box_node is not None:
            if box_node.value in codebox_set:
                codebox = codebox_set[box_node.value]
            else:
                raise ValueError(f"no codebox with the name {box_node.value} has been registered")

        # input
        if input_data is not None:
            if self.input:
                raise RuntimeError("internal: self.input is not None")
            self.input = dict(input_data)
        inputNode = syaml.map_try_get_value(self.mapping, "input")
        if inputNode is not None:
            if not syaml.is_mapping(inputNode):
                raise TypeError("the value of \"input\" must be a mapping")
            else:
                localInput = syaml.extract_content(inputNode)
                self.input.update(localInput)

        # matrix
        if matrix is not None:
            if self.matrix:
                raise RuntimeError("internal: both matrix and self.matrix are trueish")
            self.matrix = dict(matrix)
        matrixNode = syaml.map_try_get_value(self.mapping, "matrix")
        if matrixNode != None:
            if not syaml.is_mapping(matrixNode):
                raise TypeError("the value of \"matrix\" must be a mapping")

            self.matrix = {}

            for keyNode, valueNode in matrixNode.value:
                key = syaml.get_string(keyNode)
                syaml.assert_is_sequence(valueNode)
                listData = []
                for node in valueNode.value:
                    listData.append(syaml.extract_content(node))
                self.matrix[key] = listData
        
        # slab case
        if len(self.children) == 0:
            if codebox == None:
                raise ValueError("no box declared down to this slab")
            self.codebox = codebox
            if inputNode is None:
                raise ValueError("the input entry is mendatory on slabs")

            if len(self.matrix) > 0:
                noduleList = [self.clone()]
                noduleList[0].flag = focustree.Flag.NONE
                noduleList[0].matrix = None
                for key, valueList in self.matrix:
                    noduleList = multiplyList(key, valueList, noduleList)
                self.children = noduleList
                for child in self.children:
                    child.input.update(self.input)

            # all good with the current slab
            return

        # node case: populating children
        validChildren = []
        for child in self.children:
            try:
                child.populate(codebox_set, codebox, self.input, self.matrix)
                validChildren.append(child)
            except Exception as e:
                print("Exception", e)

        if len(validChildren) == 0:
            raise ValueError("no valid children")

        self.children = validChildren

        def run_codebox(self, context):
            try:
                self.codebox
            except Exception as e:
                e

def read_flag(node: yaml.Node):
    print("read_flag", node)
