from enum import StrEnum
from typing import Any, List, Optional, Set, Self, Literal
from dataclasses import dataclass
from random import randint

class Direction(StrEnum):
    N = "N"
    S = "S"
    E = "E"
    W = "W"

    def left(self) -> Self:
        match self:
            case "N": 
                return Direction.W
            case "W":
                return Direction.S
            case "S":
                return Direction.E
            case "E":
                return Direction.N

    def right(self) -> Self:
        match self:
            case "N":
                return Direction.E
            case "E":
                return Direction.S
            case "S":
                return Direction.W
            case "W":
                return Direction.N

@dataclass
class Point:
    x: int
    y: int

    def __hash__(self):
        return hash((self.x, self.y))

    def __add__(self, other) -> Self:
        if isinstance(other, Point):
            return Point(self.x + other.x, self.y + other.y)
        else:
            raise TypeError

    def taxi_distance(self, other) -> int:
        return abs(self.x - other.x) + abs(self.y - other.y)

    def forward(self, direction: Direction, distance: int = 1) -> Self:
        match direction:
            case "N":
                return Point(self.x, self.y - distance) # ty: ignore[invalid-return-type]
            case "S":
                return Point(self.x, self.y + distance) # ty: ignore[invalid-return-type]
            case "E":
                return Point(self.x + distance, self.y) # ty: ignore[invalid-return-type]
            case "W":
                return Point(self.x - distance, self.y) # ty: ignore[invalid-return-type]

@dataclass
class Rect:
    top_left: Point
    bot_right: Point

    def height(self) -> int:
        return self.bot_right.y - self.top_left.y

    def width(self) -> int:
        return self.bot_right.x - self.top_left.x

    def contains(self, point: Point) -> bool:
        x_contained = self.top_left.x <= point.x and point.x <= self.bot_right.x
        y_contained = self.top_left.y <= point.y and point.y <= self.bot_right.y
        return x_contained and y_contained

    def random_point_inside(self) -> Point:
        x = randint(self.top_left.x, self.bot_right.x)
        y = randint(self.top_left.y, self.bot_right.y)
        return Point(x, y)



class Shape():
    def __init__(self, text: str):
        self.points: Set[Point] = set()

        lines = text.split("\n")
        for y, line in enumerate(lines):
            for x, char in enumerate(line):
                if char != ".":
                    self.points.add(Point(x, y))
    
    def outline(self) -> Self:
        outline = set()
        for point in self.points:
            for x_adj in [-1, 0, 1]:
                for y_adj in [-1, 0, 1]:
                    point_adj = point + Point(x_adj, y_adj)
                    if point_adj not in self.points:
                        if point_adj.x >= 0 and point_adj.y >= 0:
                           outline.add(point_adj)

        outline_shape = Shape("")
        outline_shape.points = outline
        return outline_shape

@dataclass
class ShapeProperties():
    name: str
    material: Literal["COPPER_PLATE", "IRON_PLATE", "WAFER"]
    shape: Shape 

    @staticmethod
    def from_name(name: str) -> Self:
        match name:
            case "GEAR":
                return GEAR
            case "BAR_WINDING":
                return BAR_WINDING
            case "NUT":
                return NUT
            case x:
                ValueError(f"{x} not a shape")

gear_text = """......OO......
.OO...OO...OO.
..OOOOOOOOOO..
...OO....OO...
OOOO......OOOO
...OO....OO...
..OOOOOOOOOO..
.OO...OO...OO.
......OO......"""

GEAR = ShapeProperties("GEAR", "IRON_PLATE", Shape(gear_text))

bw_text = """......O......
....OOOOO....
..OOO...OOO..
OOO.......OOO
O...........O
O...........O
O...........O
O...........O
O...........O
O...........O
O...........O
O...........O
"""

BAR_WINDING = ShapeProperties("BAR_WINDING", "COPPER_PLATE", Shape(bw_text))

nut_text = """...xxxxxxxx...
..xxxxxxxxxx..
.xxxx....xxxx.
xxxx......xxxx
.xxxx....xxxx.
..xxxxxxxxxx..
...xxxxxxxx...
"""

NUT = ShapeProperties("NUT", "IRON_PLATE", Shape(nut_text))
