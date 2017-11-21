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
class PlayerChar(pygame.sprite.Sprite):
    BORDER = 5
    WIDTH = 60
    SELECT_COLOR = (0, 192, 32)
    def __init__(self, pos, color):
        super().__init__()
        self.image = pygame.image.load("pig_down.png").convert_alpha()
        self.rect = self.image.get_rect()
        self.rect.x, self.rect.y = pos

        width = self.rect.width
        self.outline = pygame.Rect(self.rect.x - PlayerChar.BORDER,
                                   self.rect.y - PlayerChar.BORDER,
                                   width + (2 * PlayerChar.BORDER),
                                   width + (2 * PlayerChar.BORDER))
        self.color = color
        self.selected = False
        self.speed = 5

    def draw_selected(self, display):
        self.outline.x = self.rect.x - PlayerChar.BORDER
        self.outline.y = self.rect.y - PlayerChar.BORDER
        pygame.draw.rect(display, PlayerChar.SELECT_COLOR, self.outline,
                         PlayerChar.BORDER)


@Drawable.register
class Wall(pygame.sprite.Sprite):
    WIDTH = 10
    COLOR = (255, 255, 255)
    def __init__(self, x, y, width, height):
        super().__init__()
        self.image = pygame.Surface([width, height])
        self.image.fill(Wall.COLOR)

        self.rect = self.image.get_rect()
        self.rect.x = x
        self.rect.y = y


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
        self.player_chars = pygame.sprite.Group()
        self.player = None

    def init_scene(self):
        pygame.init()
        self.display = pygame.display.set_mode((self.width, self.height))

        self.obstacles = pygame.sprite.Group()
        self.obstacles.add(Wall(0, 0, Wall.WIDTH, self.height))
        self.obstacles.add(Wall(0, 0, self.width, Wall.WIDTH))
        self.obstacles.add(Wall(0, self.height - Wall.WIDTH,
                           self.width, Wall.WIDTH))
        self.obstacles.add(Wall(self.width - Wall.WIDTH, 0,
                                Wall.WIDTH, self.height))

        self.player = PlayerChar((30, 30), (0xff, 0xff, 0xff))
        self.player.selected = True
        self.player_chars.add(self.player)

    def redraw(self):
        self.display.fill(Game.BG_COLOR)
        self.obstacles.draw(self.display)

        self.player.draw_selected(self.display)
        self.player_chars.draw(self.display)

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
        if vec is None:
            return

        dx, dy = vec
        # TODO: kinda messy, could probably do it more explicitly?
        for dx, dy in ((vec[0], 0), (0, vec[1])):
            self.player.rect.move_ip(dx, dy)
            collided = pygame.sprite.spritecollide(self.player,
                                                   self.obstacles,
                                                   False)
            for obstacle in collided:
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
