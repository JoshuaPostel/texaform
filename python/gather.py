from typing import Dict, List, Optional
from time import sleep

from characters import name_char_map, char_name_map
from utils import Point, Rect
from dog import Dog


FAB_LOCATION = Point(125, 125)


TRASH_ZONE = Rect(Point(100, 125), Point(110, 130))
DEPOT_ROOT = Point(125, 130)

class Depot():
    def __init__(self, top_left: Point, entities: List[str]):
        self.root = top_left
        self.resource_roots: Dict[str, Point] = {}
        self.resource_count: Dict[str, int] = {}
        # keep track of how many "chunks"
        self.height = 6
        for idx, entity in enumerate(entities): 
            self.resource_roots[entity] = self.root + Point(2 * idx, 0)
            self.resource_count[entity] = 0
        width = len(self.resource_roots) * 3
        height = 6
        self.area = Rect(self.root, self.root + Point(width, height))

    def address(self, entity: str, n: int) -> Point:
        root = self.resource_roots[entity]
        y = n + ((n - 1) // 5)
        return root + Point(0, y)

    def store(self, dog: Dog, entity: str):
        try:
            self.resource_count[entity] += 1
            count = self.resource_count[entity]
            address = self.address(entity, count)
            if not self.area.contains(address):
                print(f"WARNING address {address} outside of area: {self.area}")
            dog.drop_off(address)
        except KeyError:
            print(f"error, tried to store {entity}")

    def take(self, dog: Dog, entity: str):
        count = self.resource_count[entity]
        if count > 0:
            self.resource_count[entity] -= 1
            address = self.address(entity, count)
            dog.pick_known(entity, address)
            dog.goto(self.root + Point(-1, -1))

    def lowest_resource(self) -> str:
        min_count = min(self.resource_count.values())
        for (entity, count) in self.resource_count.items():
            if count == min_count:
                return entity

    def expand_area(self, dog: Dog):
        dog.charge_if_needed()
        tl = Point(self.root.x, self.area.bot_right.y + 1)
        br = Point(self.area.bot_right.x, self.area.bot_right.y + 6)
        area_to_clear = Rect(tl, br)
        clear_area(dog, area_to_clear, self)
        self.area.bot_right.y += 6


    def gather(self, dog: Dog):
        while True:
            dog.charge_if_needed()
            entity = self.lowest_resource()
            print(f"gathering {entity}")
            # how to prevent dog from picking from depot without going through self.take()?
            #dog.find_char_random_walk()
            # for now, just go far away
            target = dog._generate_random_point(min_dist=20)
            dog.goto(target)
            dog.pick_unknown(entity)
            dog.charge_if_needed()
            dog.goto(self.root)
            self.store(dog, entity)

def clear_column(dog: Dog, top: Point, height: int, depot: Optional[Depot]):
    start = top + Point(0, -1)
    success = dog.goto(start)
    if not success:
        dog.approach(start)
        (char, _, _) = dog.scan()
        dog.send(f"PICK {char}")
        if depot:
            depot.store(dog, char_name_map[char])
        else:
            dog.drop_within(TRASH_ZONE)
        return clear_column(dog, top, height, depot)
    dog.face("S")
    while dog.location != Point(top.x, top.y + height):
        (char, _, _) = dog.scan()
        if char == ".":
            dog.move()
        else:
            dog.pick(char)
            if depot:
                depot.store(dog, char_name_map[char])
            else:
                dog.drop_within(TRASH_ZONE)
            clear_column(dog, top, height, depot)

def clear_area(dog: Dog, area: Rect, depot: Optional[Depot]):
    for x in range(area.width() + 1):
        top = area.top_left + Point(x, 0)
        clear_column(dog, top, area.height(), depot)


#dog0 = Dog(3335)
#dog0.approach(Point(127, 123))
#(char, _, _) = dog0.scan()
#print(char)
#dog0.send(f"PICK {char}")
#assert 2 == 4


#dog1.charge_if_needed()
depot = Depot(Point(125, 130), ["IRON", "COPPER", "SILICATE", "SULFER"])


# for vhs demo recording wait for texaform to boot up
while True:
    try:
        dog1 = Dog(3336)
        break 
    except ConnectionRefusedError:
        print("waiting for texaform")
        sleep(1)

dog1.charge_location = Point(130, 127)
dog1.depot_area = depot.area
#clear_area(dog1, depot.area, depot=None)
while True:
    print("expanding!")
    depot.expand_area(dog1)
    print(depot.area)

#depot.gather(dog1)


#while depot.resource_count[]
#dog1.find_char_random_walk()



    







