import os
import subprocess
from pathlib import Path
import numpy as np
import sys
from matplotlib.patches import Circle, Polygon
from matplotlib.collections import PatchCollection
import matplotlib.pyplot as plt
import itertools

tints = [[1.0, 0.0, 0.0, 1.0], [0.0, 1.0, 0.0, 1.0], [0.0, 0.0, 1.0, 1.0], [1.0, 1.0, 0.0, 1.0], [0.0, 1.0, 1.0, 1.0], [0.3, 0.7, 0.2, 1.0]]
defaultColor = [1.0, 0.0, 0.0, 1.0]
linewidth = 0.10

length = 2.31426e24
onlyBox = False
pointSize = length * 1.0

def getTints():
    return itertools.cycle(tints)


def getCircle(args, color):
    return Circle((args[0], args[1]), args[2], fill=False, edgecolor=color, linewidth=linewidth)


def getPoint(args, color):
    return np.array([args[0], args[1]])
    # return Circle((args[0], args[1]), pointSize, fill=True, facecolor=color)


def getPolygon(args, color):
    numPoints = len(args) // 2
    ps = np.zeros((numPoints, 2))
    for i in range(numPoints):
        ps[i, 0] = args[2 * i]
        ps[i, 1] = args[2 * i + 1]

    return None
    return Polygon(ps, closed=True, linewidth=linewidth, linestyle="-", edgecolor=color, fill=False)


def mix(c1, c2):
    return [0.5 * (c1[0] + c2[0]), 0.5 * (c1[1] + c2[1]), 0.5 * (c1[2] + c2[2]), c1[3]]


def getColor(split, tint):
    if len(split) == 1:
        color = defaultColor
    elif len(split) == 2:
        color = [float(x) for x in split[1].split(" ")]
    else:
        raise ValueError
    if tint is not None:
        return mix(color, tint)
    else:
        return color


def getPatch(line, tint):
    split = line.split(" color ")
    args = split[0].split(" ")
    type_ = args[0]
    args = [float(x) for x in args[1:]]
    color = getColor(split, tint)
    if type_ == "Circle":
        return getCircle(args, color)
    elif type_ == "Polygon":
        return getPolygon(args, color)
    elif type_ == "Point":
        return getPoint(args, color)
    raise NotImplementedError()


def addPatchesForFile(fname, tint=None):
    patches = []
    points = []

    with open(fname, "r") as f:
        for line in f.readlines():
            patch = getPatch(line, tint)
            if type(patch) == np.ndarray:
                points.append(patch)
            else:
                if patch is not None:
                    patches.append(patch)

    return PatchCollection(patches, match_original=True), np.array(points)


def plotFiles(fnames, tints, outFile, show=True):
    fig, ax = plt.subplots()
    if onlyBox:
        ax.set_xlim(length * np.array([0.0, 1.0]))
        ax.set_ylim(length * np.array([0.0, 1.0]))
    else:
        ax.set_xlim(length * np.array([-1.0, 2.0]))
        ax.set_ylim(length * np.array([-1.0, 2.0]))

    for fname, tint in zip(fnames, tints):
        collection, points = addPatchesForFile(fname, tint)
        ax.add_collection(collection)
        ax.scatter(points[:, 0], points[:, 1], color=tint, s=1.0)
    if show:
        plt.show()
    else:
        dirname = Path(fname)
        outFile.parent.mkdir(exist_ok=True)
        out = outFile
        plt.savefig(out, dpi=800)
        showImageInTerminal(outFile)
    fig.clf()


def getFilesInDir(dirname):
    def f():
        for f in os.listdir(dirname):
            yield dirname / f

    return list(f())


def showImageInTerminal(path: Path) -> None:
    width = 800
    height = 600
    tmpfile = Path("/tmp/test.png")
    args = ["convert", str(path), "-scale", f"{width}x{height}", str(tmpfile)]
    subprocess.check_call(args, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
    args = ["kitty", "+kitten", "icat", "--silent", "--transfer-mode", "file", str(tmpfile)]
    subprocess.check_call(args)


if sys.argv[-1] == "show":
    args = sys.argv[1:-1]
    show = True
else:
    args = sys.argv[1:]
    show = False
out = Path(args[0])
dirnames = args[1:]

dirnames = [Path(x) for x in dirnames]

numFiles = len(getFilesInDir(dirnames[0]))
files = getFilesInDir(dirnames[0])
files.sort()
for num, f in enumerate(files):
    fnames = [d / f.name for d in dirnames]
    outFile = (out / f"{num:03}").with_suffix(".png")
    print(fnames)
    print(outFile)
    plotFiles(fnames, getTints(), outFile, show=show)
