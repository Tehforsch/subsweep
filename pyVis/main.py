import numpy as np
import sys
from matplotlib.patches import Circle, Polygon, Rectangle
from matplotlib.collections import PatchCollection
import matplotlib.pyplot as plt

fig, ax = plt.subplots()

patches = []
colors = []

ax.set_xlim([-1.0, 2.0])
ax.set_ylim([-1.0, 2.0])

def getCircle(args):
    return Circle((args[0], args[1]), args[2])


def getPoint(args):
    return Circle((args[0], args[1]), 0.002)


def getTriangle(args):
    ps = np.array([
        [args[0], args[1]],
        [args[2], args[3]],
        [args[4], args[5]]])
    return Polygon(ps, closed=True, linewidth=0.1, linestyle="-", facecolor="red", fill=False, edgecolor="r")


def getColor(args):
    if len(args) == 4:
        return list(args)
    else:
        return [0.0, 0.0, 0.0, 1.0]


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

p = PatchCollection(patches, match_original=True)
p.set_array(None)
# print(np.array(colors).shape)
# p.set_array(np.array(colors))
ax.add_collection(p)
fig.colorbar(p, ax=ax)

plt.show()
