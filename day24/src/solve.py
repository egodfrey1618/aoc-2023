import math

def read_hailstone(s):
    (a, b) = s.strip().split(" @ ")
    (a1, a2, a3) = [int(c.strip()) for c in a.split(",")]
    (b1, b2, b3) = [int(c.strip()) for c in b.split(",")]
    return ((a1, a2, a3), (b1, b2, b3))

hailstones = []
with open("input") as f:
    while (line := f.readline()):
        if line:
            hailstones.append(read_hailstone(f.readline()))

def vector_minus(a, b):
    return [x-y for (x, y) in zip(a, b)]

def consistent(a1, m1, a2, m2):
    # Are the equations a1 mod m1, a2 mod m2 consistent with each other?
    g = math.gcd(m1, m2)

    if a1 % g == a2 % g:
        return True
    return False

def egcd(a, b):
    if a == 0:
        return (b, 0, 1)
    else:
        g, y, x = egcd(b % a, a)
        return (g, x - (b // a) * y, y)

def crt_coprime(a1, m1, a2, m2):
    (g, a, b) = egcd(m1, m2)
    assert g == 1
    assert a*m1 + b*m2 == 1

    X = (a2*m1*a + a1*m2*b) % (m1 * m2)

    assert X % m1 == a1 % m1 
    assert X % m2 == a2 % m2
    return (X, m1 * m2)

def test_velocity_guess(S):
    # Try and solve given a guess for the velocity.
    # If the hailstones have positions p_i and velocity v_i, we're trying to solve
    # p_i + t_i(v_i - S) = R
    # 
    # Where (R, S) is the position/velocity of our rock (integers), and t_i should be positive reals.
    # 
    # If we let w_i be (v_i - S) but with any gcd divided out, then we can assume t_i are integers.
    # This lets us then take moduli to get R mod w_i.

    moduli = []

    for (p, v) in hailstones:
        w = vector_minus(v, S)
        g = math.gcd(*w)
        w = [(x//g) for x in w]
        w = [a*-1 if a < 0 else a for a in w]

        # if an index of the velocity's zero, this gives us something stronger! but I'll ignore for now.
        w = [1 if a == 0 else a for a in w]

        m = [(a % b, b) for (a, b) in zip(p, w)]
        moduli.append(m)

    for index in [0, 1, 2]:
        for m in moduli:
            for m2 in moduli:
                X = m[index]
                Y = m2[index]
                if not consistent(X[0], X[1], Y[0], Y[1]):
                    return False
    return True

def solve_given_velocity_guess(S):
    moduli = []

    for (p, v) in hailstones:
        w = vector_minus(v, S)
        g = math.gcd(*w)
        w = [(x//g) for x in w]
        w = [a*-1 if a < 0 else a for a in w]
        w = [1 if a == 0 else a for a in w]
        m = [(a % b, b) for (a, b) in zip(p, w)]
        moduli.append(m)

    l = []
    for index in [0, 1, 2]:
        result = (0, 1)
        for (value, modulus) in [m[index] for m in moduli]:
            # I couldn't be bothered to work out how to CRT when things aren't coprime.
            # Probably I could do this with factoring integers? But this works well!
            if math.gcd(result[1], modulus) == 1:
                result = crt_coprime(result[0], result[1], value, modulus)
                # print(result)
        print(index, result)
        l.append((index, result))
    print(sum(r[1][0] for r in l))

print("Searching for valid velocities...")
valid_velocities = []

# These ranges came from some Jupyter investigation - I committed the notebook, contains a comment there.
for Z in range(32, 41):
    for Y in range(25, 35):
        for X in range(-400, -200):
            if test_velocity_guess((X, Y, Z)):
                print("Found one", X, Y, Z)
                valid_velocities.append((X, Y, Z))

velocity = valid_velocities[0]
solve_given_velocity_guess(velocity)


