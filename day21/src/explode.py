lines_noelf = open("input_elf_stripped","r").read().strip().split("\n")
lines_elf = open("input","r").read().strip().split("\n")

# Explode
SIZE = 10
for i in range(SIZE):
    for j, line in enumerate(lines_noelf):
        if i == SIZE // 2 and j == (len(lines_noelf) // 2):
            print(line * (SIZE//2) + lines_elf[j] + line * (SIZE//2))
        else:
            print(line * SIZE)
