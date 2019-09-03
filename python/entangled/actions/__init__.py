from abc import ABC, abstractmethod

from grid import coords_circle


class CombatAction(ABC):
    def __init__(self, key):
        if not key:
            raise Exception("key is required")

        self.key = key

    @abstractmethod
    def targetable(self, origin_coord, combat):
        pass

    @abstractmethod
    def aoe(self, target_coord, combat):
        pass


class RadiusAction(CombatAction):
    def __init__(self, key, radius, aoe_radius=0):
        super().__init__(key)
        self.radius = radius
        self.aoe_radius = aoe_radius

    def targetable(self, origin_coord, combat):
        if self.radius == 0:
            return (origin_coord,)

        return (
            coord
            for coord in coords_circle(origin_coord, self.radius)
            if coord in combat.grid
        )

    def aoe(self, target_coord, combat):
        if self.aoe_radius == 0:
            return (target_coord,)

        return (
            coord
            for coord in coords_circle(target_coord, self.aoe_radius)
            if coord in combat.grid
        )
