from utils import Point, ShapeProperties, GEAR, BAR_WINDING, NUT
from dog import Dog
from smelter import Smelter
from lazer_cutter import LazerCutter
from hud import HUD
from fabricator import Fabricator

from typing import Dict, Optional

from time import sleep


# the only agent kind that is unique
# the only agent without a location
# for vhs demo recording wait for texaform to boot up
while True:
    try:
        hud = HUD(3333, Point(-1, -1))
        break 
    except ConnectionRefusedError:
        print("waiting for texaform")
        sleep(1)

# TODO HUD could return this info?
RESEARCH_REQUIREMENTS = {
    "SMELTER": {"IRON": 2},
    "LAZER_CUTTER": {"COPPER_PLATE": 2, "IRON_PLATE": 2},
    "SOLAR": {"COPPER_PLATE": 2, "WAFER": 2},
    "BATTERY": {"SULFER": 2, "IRON_PLATE": 2},
    "DOG": {"BATTERY": 2, "GEAR": 2, "SOLAR_PANNEL": 2},
    "ACCUMULATOR": {"BATTERY": 2},
    "FABRICATOR": {"NUT": 2},
    "VICTORY": {"DOG": 10},
}

BUILD_REQUIREMENTS = {
    "SMELTER": {"IRON_PLATE": 4, "COPPER_PLATE": 1},
    "FABRICATOR": {"NUT": 4, "IRON_PLATE": 4, "GEAR": 2, "MOTOR": 2},
    "ACCUMULATOR": {"IRON_PLATE": 2, "COPPER_PLATE": 1, "BATTERY": 4},
    "SOLAR_PANNEL": {"IRON_PLATE": 1, "COPPER_PLATE": 2, "WAFER": 2},
    "BATTERY": {"IRON_PLATE": 1, "COPPER_PLATE": 1, "SULFER": 1},
    "MOTOR": {"IRON_PLATE": 1, "GEAR": 1, "BAR_WINDING": 3},
    "LAZER_CUTTER": {"MOTOR": 1, "IRON_PLATE": 4, "GEAR": 2},
    "DOG": {"IRON_PLATE": 6, "MOTOR": 5, "BATTERY": 1, "SOLAR_PANNEL": 1},
}

def smelted_from(smeltable: str) -> str:
    match smeltable:
        case "IRON_PLATE":
            return "IRON"
        case "COPPER_PLATE":
            return "COPPER"
        case "WAFER":
            return "SILICATE"
        case x:
            ValueError(f"{x} not smeltable")

def make_smeltable(smeltable: str, dog: Dog, smelter: Smelter, fab: Fabricator):
    raw_ingredient = smelted_from(smeltable)
    dog.fetch(raw_ingredient, smelter.location)
    smelter.smelt(hud, smeltable)
    dog.transport(smeltable, smelter.location, fab.location)


def make_shape(name: str, dog: Dog, smelter: Smelter, lc: LazerCutter, fab: Fabricator):
    shape = ShapeProperties.from_name(name)
    make(shape.material, dog, smelter, lc, fab)
    dog.transport(shape.material, fab.location, lc.location)
    lc.cut_shape(shape)
    dog.transport(shape.name, lc.location, fab.location)


def make(
    thing: str, dog: Dog, smelter: Smelter, lc: LazerCutter, fab: Fabricator
):
    requirements = BUILD_REQUIREMENTS.get(thing)
    if requirements is not None:
        make_requirements(requirements, dog, smelter, lc, fab)
        fab.send(f"MAKE {thing}")
        dog.transport(thing, fab.location, fab.location)
    else:
        match thing:
            # raw materials
            case "IRON" | "COPPER" | "SILICATE" | "SULFER":
                dog.fetch(thing, fab.location)
            # smeltable 
            case "IRON_PLATE" | "COPPER_PLATE" | "WAFER":
                make_smeltable(thing, dog, smelter, fab)
            # shapes
            case "GEAR" | "BAR_WINDING" | "NUT":
                make_shape(thing, dog, smelter, lc, fab)
            case x:
                ValueError(f"make {x} not implemented")


def make_requirements(
    requirements: Dict[str, int],
    dog: Dog,
    smelter: Smelter,
    lc: LazerCutter,
    fab: Fabricator,
):
    for name, count in requirements.items():
        while fab.resource_count(name) < count:
            make(name, dog, smelter, lc, fab)


def research(
    research_name: str, dog: Dog, smelter: Smelter, lc: LazerCutter, fab: Fabricator
):
    if research_name not in hud.completed_research():
        print(f"researching: {research_name}")
        hud.send(f"RESR {research_name}")
        make_requirements(RESEARCH_REQUIREMENTS[research_name], dog, smelter, lc, fab)
        fab.research()
        fab.research()

def build(
    name: str,
    location: Point,
    dog: Dog,
    smelter: Smelter,
    lc: LazerCutter,
    fab: Fabricator,
) -> Optional[int]:
    make(name, dog, smelter, lc, fab)
    dog.pick_known(name, fab.location)
    return dog.build_at(location)
