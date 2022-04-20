import dataclasses
import os

@dataclasses.dataclass
class File:
    path: str
    content: str

def read_local_file(path: str, location: str=None) -> File:
    if location:
        path = os.path.join(os.path.dirname(os.path.abspath(location)), path)
    with open(path) as f:
        return File(path, f.read())

def virtual_file(path: str, content: str) -> File:
    return File(path, content)
