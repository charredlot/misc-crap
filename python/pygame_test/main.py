#!/usr/bin/python3

from abc import ABCMeta, abstractmethod
from argparse import ArgumentParser
from math import sqrt

import pygame


# frames per sec
FRAME_RATE = 60


# maybe should figure out sprites later
class Drawable(metaclass=ABCMeta):
    @abstractmethod
    def draw(self, display):
        pass


@Drawable.register
class PlayerChar(object):
    BORDER = 5
    WIDTH = 60
    SELECT_COLOR = (0, 192, 32)
    def __init__(self, pos, color):
        x, y = pos
        self.rect = pygame.Rect(x, y, PlayerChar.WIDTH, PlayerChar.WIDTH)
        self.outline = pygame.Rect(x - PlayerChar.BORDER,
                                   y - PlayerChar.BORDER,
                                   PlayerChar.WIDTH + (2 * PlayerChar.BORDER),
                                   PlayerChar.WIDTH + (2 * PlayerChar.BORDER))
        self.color = color
        self.selected = False
        self.speed = 5

    def draw(self, display):
        if self.selected:
            self.outline.x = self.rect.x - PlayerChar.BORDER
            self.outline.y = self.rect.y - PlayerChar.BORDER
            pygame.draw.rect(display, PlayerChar.SELECT_COLOR, self.outline)
        pygame.draw.rect(display, self.color, self.rect)

    def move(self, vec):
        self.rect.x += vec[0]
        self.rect.y += vec[1]

@Drawable.register
class Wall(object):
    WIDTH = 10
    COLOR = (255, 255, 255)
    def __init__(self, x, y, width, height):
        self.rect = pygame.Rect(x, y, width, height)
        self.left = x
        self.right = x + width
        self.top = y
        self.bottom = y + height

    def draw(self, display):
        pygame.draw.rect(display, Wall.COLOR, self.rect)

class Game(object):
    MOVE_UP = pygame.K_UP
    MOVE_DOWN = pygame.K_DOWN
    MOVE_LEFT = pygame.K_LEFT
    MOVE_RIGHT = pygame.K_RIGHT
    BG_COLOR = (0, 75, 11)
    WALL_COLOR = (139, 69, 19)
    def __init__(self, width, height):
        self.width = width
        self.height = height
        self.display = None
        self.player = None

    def init_scene(self):
        pygame.init()
        self.display = pygame.display.set_mode((self.width, self.height))
        self.obstacles = [
            Wall(0, 0, Wall.WIDTH, self.height),
            Wall(0, 0, self.width, Wall.WIDTH),
            Wall(0, self.height - Wall.WIDTH, self.width, Wall.WIDTH),
            Wall(self.width - Wall.WIDTH, 0, Wall.WIDTH, self.height),
        ]
        self.player = PlayerChar((30, 30), (0xff, 0xff, 0xff))
        self.player.selected = True

    def redraw(self):
        self.display.fill(Game.BG_COLOR)
        for obstacle in self.obstacles:
            obstacle.draw(self.display)
        self.player.draw(self.display)

    def player_movement(self, pressed):
        if self.player is None:
            return None

        x = 0
        y = 0
        if pressed[self.MOVE_UP]:
            y -= 1
        if pressed[self.MOVE_DOWN]:
            y += 1
        if pressed[self.MOVE_LEFT]:
            x -= 1
        if pressed[self.MOVE_RIGHT]:
            x += 1

        if x == 0 and y == 0:
            return None

        # normalize diagonal movement
        norm = sqrt((x * x) + (y * y))
        return ((float(x) / norm) * self.player.speed,
                (float(y) / norm) * self.player.speed)

    def resolve_movement(self, pressed):
        vec = self.player_movement(pressed)
        if vec is not None:
            dx, dy = vec
            # TODO: kinda messy, could probably do it more explicitly?
            for dx, dy in ((vec[0], 0), (0, vec[1])):
                self.player.rect.move_ip(dx, dy)
                for obstacle in self.obstacles:
                    if not self.player.rect.colliderect(obstacle.rect):
                        continue

                    if dx > 0:
                        # collide from the left
                        self.player.rect.right = obstacle.rect.left
                    elif dx < 0:
                        self.player.rect.left = obstacle.rect.right

                    if dy > 0:
                        # collide from above
                        self.player.rect.bottom = obstacle.rect.top
                    elif dy < 0:
                        self.player.rect.top = obstacle.rect.bottom

def main():
    parser = ArgumentParser()
    parser.add_argument("--width", help="screen width",
                        type=int,
                        default=800,
                        required=False)
    parser.add_argument("--height",
                        help="screen height",
                        type=int,
                        default=600,
                        required=False)
    args = parser.parse_args()

    g = Game(args.width, args.height)
    g.init_scene()

    clock = pygame.time.Clock()
    pygame.display.update()

    done = False
    while not done:
        for event in pygame.event.get():
            if event.type == pygame.QUIT:
                done = True
                print("got quit event {}".format(event))
                break
        pressed = pygame.key.get_pressed()

        g.resolve_movement(pressed)
        g.redraw()
        pygame.display.flip()
        clock.tick(FRAME_RATE)


if __name__ == "__main__":
    main()
