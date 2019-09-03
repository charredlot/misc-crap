from actions import RadiusAction
from combat import Combat
from grid import AxialCoord, HexGrid
from unit import ActionList, CombatUnit


def get_combat():
    grid = HexGrid(
        (
            AxialCoord(-3, -2),
            AxialCoord(-3, 0),
            AxialCoord(-2, -4),
            AxialCoord(-2, -3),
            AxialCoord(-2, 0),
            AxialCoord(-2, 1),
            AxialCoord(-2, 2),
            AxialCoord(-1, -4),
            AxialCoord(-1, -3),
            AxialCoord(-1, 0),
            AxialCoord(-1, 1),
            AxialCoord(-1, 2),
            AxialCoord(0, -4),
            AxialCoord(0, -3),
            AxialCoord(0, -2),
            AxialCoord(0, -1),
            AxialCoord(0, 0),
            AxialCoord(0, 1),
            AxialCoord(0, 2),
            AxialCoord(0, 3),
            AxialCoord(1, -2),
            AxialCoord(1, -1),
            AxialCoord(1, 0),
            AxialCoord(1, 1),
            AxialCoord(1, 2),
            AxialCoord(1, 3),
            AxialCoord(1, 4),
            AxialCoord(1, 5),
            AxialCoord(1, 6),
            AxialCoord(1, 7),
            AxialCoord(2, -4),
            AxialCoord(2, -3),
            AxialCoord(2, -2),
            AxialCoord(2, 0),
            AxialCoord(2, 1),
            AxialCoord(2, 2),
            AxialCoord(2, 3),
            AxialCoord(2, 4),
            AxialCoord(2, 5),
            AxialCoord(3, -5),
            AxialCoord(3, -4),
            AxialCoord(3, -3),
            AxialCoord(3, 4),
            AxialCoord(3, 5),
            AxialCoord(3, 6),
            AxialCoord(3, 7),
        )
    )
    combat = Combat(grid)
    combat.debug.print_events = True

    combat.place_unit(
        CombatUnit(
            "P1",
            friendly=True,
            control=CombatUnit.PLAYER_CONTROL,
            actions=ActionList(
                top_level=[RadiusAction(key="Melee", radius=1)],
                folders={
                    "Elemental": [
                        RadiusAction(key="Fireball", radius=3, aoe_radius=1)
                    ]
                },
            ),
        ),
        AxialCoord(2, 5),
    )
    combat.place_unit(
        CombatUnit("P2", friendly=True, control=CombatUnit.PLAYER_CONTROL),
        AxialCoord(1, 6),
    )

    for i, coord in enumerate(
        (
            AxialCoord(-1, -3),
            AxialCoord(-2, -4),
            AxialCoord(-2, -3),
            AxialCoord(3, -4),
        )
    ):
        combat.place_unit(CombatUnit("Mook {}".format(i)), coord)

    return combat
