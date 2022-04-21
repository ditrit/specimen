import dataclasses

@dataclasses.dataclass
class Book:
    title:    str
    left_page: int
    size:     int

    def turn_page(self, count: int):
        self.left_page += 2 * count

        if self.left_page < 0:
            self.left_page = 0
        elif self.left_page >= self.size:
            self.left_page = self.size - 1

    def get_page(self) -> int:
        return self.left_page
