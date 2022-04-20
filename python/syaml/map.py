import yaml
from .assertit import *

def map_try_get_value(node: yaml.Node, key: str) -> yaml.Node:
    for key_node, value_node in node.value:
        if key_node.value == key:
            return value_node
    return None

def map_get_value(node: yaml.Node, key: str) -> yaml.Node:
    value = map_try_get_value(node, key)
    if value is None:
        raise TypeError(f"could not find key {key}")

def get_string(node) -> str:
    assert_is_string(node)
    return node.value

def get_int(node) -> int:
    assert_is_int(node)
    return int(node.value)

def map_assert_string_keys_among(node: yaml.Node, accept_slice: list[str]):
    print("map_assert_string_keys_among", node)

def map_get_any(node: yaml.Node, key: str):
    return extract_content(map_get_value(node, key))

def extract_content(node: yaml.Node):
    if node.id == "scalar":
        if node.tag == "tag:yaml.org,2002:str":
            return node.value
        if node.tag == "tag:yaml.org,2002:int":
            return int(node.value)
        if node.tag == "tag:yaml.org,2002:float":
            return float(node.value)
        if node.tag == "tag:yaml.org,2002:bool":
            return node.value.lower() in ["true", "on", "y", "yes"]
        raise TypeError("internal: unknown scalar tag: " + node.tag)
    if node.id == "sequence":
        return [extract_content(child) for child in node.value]
    if node.id == "mapping":
        return { 
            key.value: extract_content(value)
            for key, value in node.value
        }
    raise TypeError("internal: unknown node kind: " + node.id)