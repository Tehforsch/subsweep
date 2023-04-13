import numpy as np
import sys
from matplotlib.patches import Circle, Polygon
from matplotlib.collections import PatchCollection
import matplotlib.pyplot as plt

fig, ax = plt.subplots()

patches = []
colors = []


def getCircle(args):
    return Circle((args[0], args[1]), args[2])


def getPoint(args):
    return Circle((args[0], args[1]), 0.002)


def getTriangle(args):
    ps = np.array([
        [args[0], args[1]],
        [args[2], args[3]],
        [args[4], args[5]]])
    print(ps)
    return Polygon(ps, closed=True)


def getColor(args):
    return tuple(args)


def getPatch(args):
    type_ = args[0]
    args = [float(x) for x in args[1:]]
    if type_ == "Circle":
        color = getColor(args[3:])
        return getCircle(args[0:]), color
    elif type_ == "Triangle":
        color = getColor(args[6:])
        return getTriangle(args[0:]), color
    elif type_ == "Point":
        color = getColor(args[2:])
        return getPoint(args), color
    raise NotImplementedError()


with open(sys.argv[1], "r") as f:
    for line in f.readlines():
        args = line.split(" ")
        patch, color = getPatch(args)
        patches.append(patch)
        colors.append(color)

p = PatchCollection(patches, alpha=0.4)
# p.set_array(colors)
ax.add_collection(p)
fig.colorbar(p, ax=ax)

plt.show()
