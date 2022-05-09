import enum
import sys
import traceback
import unittest
import yaml

import python.syaml as syaml
import python.focustree as focustree


class FailStatus(enum.Enum):
    PRISTINE = 0
    FAILED = 1
    ABORTED = 2
    RAISED = 3


class SpecimenContext:
    def __init__(self, test_case: unittest.TestCase):
        self.test_case = test_case
        self.slab_count = 0
        self.slab_passed = 0
        self.slab_failed = 0
        self.slab_aborted = 0
        self.slab_raised = 0
        self.failure_report = []

        self.status = FailStatus.PRISTINE
        self.fail_info = []

    def fail(self, info: str):
        self.status = Failed
        if len(info) > 0:
            self.fail_info.append(info)

    def abort(self, info: str):
        self.status = Aborted
        if len(info) > 0:
            self.fail_info.append(info)
        raise Exception()

    def expect_equal(self, value, wanted, context: str = ""):
        if value != wanted:
            if len(context) > 0:
                context = "(" + context + "): "
            self.fail(context + str(value))


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

    def warning(self, info: str):
        print("Warning: TreeRoot: %s" % info)


class Nodule:
    def __init__(
        self,
        file,
        kind,
        mapping=None,
    ):
        self.file = file
        self.mapping = mapping
        # Kind is one of "File", "Node", "Slab" -- Kind is used in error reports
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

    def warning(self, info: str) -> focustree:
        print("Warning: %s %s(%s): %s" % (self.kind, self.name, self.location, info))

    def initialize_file(self):
        self.mapping = yaml.compose(self.file.content)
        if not syaml.is_mapping(self.mapping):
            raise RuntimeError("the root of the yaml document must be a yaml mapping")
        self.initialize()

    def initialize(self):
        # location
        self.location = f"{self.file.path}:{self.mapping.start_mark.line+1}:{self.mapping.start_mark.column+1}"

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
        if len(content_node.value) > 0 and self.kind == "Slab":
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
                nodule_list = [self.clone()]
                nodule_list[0].flag = focustree.Flag.NONE
                nodule_list[0].matrix = None
                for key, value_list in self.matrix.items():
                    nodule_list = multiply_list(key, value_list, nodule_list)
                self.children = nodule_list
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
            except ValueError as e:
                raise
                print("Error", e)

        if len(validChildren) == 0:
            raise ValueError("no valid children")

        self.children = validChildren

    def run_codebox(self, context: SpecimenContext):
        try:
            self.codebox.box_function(**self.input)
        except Exception as e:
            if context.status == FailStatus.ABORTED:
                return
            if isinstance(e, AssertionError):
                context.status = FailStatus.FAILED
                context.fail_info.append(str(e))
                return

            exc_type, exc_value, exc_traceback = sys.exc_info()
            report = "".join(map(str,traceback.format_exception(e)))

            context.status = FailStatus.RAISED
            context.fail_info.append(report)

    def clone(self):
        other = Nodule(self.file, self.kind, self.mapping)
        other.location = self.location
        other.flag = self.flag
        other.name = self.name
        other.children = self.children
        other.codebox = self.codebox
        other.input = dict(self.input)
        other.matrix = self.matrix
        return other

def read_flag(node: yaml.Node):
    syaml.assert_is_mapping(node)
    flag_node = syaml.map_try_get_value(node, "flag")
    flag = focustree.Flag.NONE

    if flag_node is None:
        return flag

    name = ""
    both = False
    sentence = syaml.get_string(flag_node).split()
    for word in sentence:
        if word == "FOCUS":
            if flag == focustree.Flag.SKIP:
                both = True
            flag = focustree.Flag.FOCUS
            name = word
        elif word == "PENDING":
            if flag == focustree.Flag.FOCUS:
                both = True
            flag = focustree.Flag.SKIP
            name = word
        elif word.isupper():
            print(f"Warning: Unrecognised all uppercase flag \"{word}\". It has been ignored.")
    if both:
        print(f"Warning: both FOCUS and PENDING have been found among the flags of a node. {name} has been kept.")

    return flag

def multiply_list(key, value_list, nodule_list):
    result_list = []
    for nodule in nodule_list:
        for k, value in enumerate(value_list):
            other = nodule.clone()
            other.input[key] = value
            other.location = "%s(%s[%d])" % (other.location, key, k)
            result_list.append(other)
    return result_list


class NoduleError(ValueError):
    def __init__(self, nodule: Nodule, message: str):
        self.nodule = nodule
        super().__init__(message)
    def __repr__(self):
        n = self.nodule
        return f"{n.kind} {n.name}({n.location}): {super().__repr__()}"
