from collections import namedtuple
from queue import PriorityQueue

s = open("input","r").read().strip()
grid = [[int(x) for x in l] for l in s.split("\n")]

State = namedtuple("State", ["position", "dir", "moves_so_far"])
MAX_MOVES = 10
MIN_MOVES = 4

def neighbours(grid, state):
    (x, y) = state.position
    result = []

    # Technically should also remove reversing here.
    for dir_ in (1, 0), (-1, 0), (0, 1), (0, -1):
        new_position = (x + dir_[0], y + dir_[1])
        if dir_ == state.dir:
            moves_so_far = state.moves_so_far + 1
        else:
            moves_so_far = 1

        (a, b) = new_position
        if not(a >= 0 and b >= 0 and a < len(grid) and b < len(grid[0])):
            continue
        if moves_so_far > MAX_MOVES:
            continue
        if dir_[0] * -1 == state.dir[0] and dir_[1] * -1 == state.dir[1]:
            continue
        if dir_ != state.dir and state.moves_so_far < MIN_MOVES:
            continue

        result.append(State(position=new_position, dir=dir_, moves_so_far=moves_so_far))
    return result

# Dijkstra from (0, 0)
def starting_point(dir): 
    return State(position=(0, 0), dir=dir, moves_so_far=0)

best_distance = {}
boundary = PriorityQueue()

for dir in [(0, 1), (0, -1), (1, 0), (-1, 0)]:
    boundary.put((0, starting_point(dir)))

while not boundary.empty():
    # Find the boundary spot with the smallest distance
    (value, key) = boundary.get()
    if key in best_distance:
        continue

    # Add it to best_distance
    best_distance[key] = value

    # Add all of the boundary spots.
    for next_ in neighbours(grid, key):
        if next_ not in best_distance:
            path_from_this_node = best_distance[key] + grid[next_.position[0]][next_.position[1]]

            boundary.put((path_from_this_node, next_))

l = {k:v for (k, v) in best_distance.items() if k.position == (len(grid) - 1, len(grid[0]) - 1)}
for k, v in l.items():
    print(k, v)

