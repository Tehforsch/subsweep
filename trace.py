import matplotlib.pyplot as plt
import sys
import seaborn as sns
import polars as pl
from pathlib import Path
import yaml
import astropy.units as pq

def make_row(cont, name):
    if name == "time":
        return (name, [pq.Quantity(x["time"]).to_value(pq.Myr) for x in cont])
    if name == "recomb":
        return (name, [pq.Quantity(x["recomb"]).to_value(pq.m**3 / pq.s) for x in cont])
    if name == "coll_ion":
        return (name, [pq.Quantity(x["coll_ion"]).to_value(pq.m**3 / pq.s) for x in cont])
    return (name, [pq.Quantity(x["solver"][name]).value for x in cont])

def getDf(f):
    print(f)
    try:
        f = Path(f)
        cont = yaml.load(open(f, "r"), Loader=yaml.SafeLoader)
        cont = [{"time": e["time"], "solver": yaml.load(e["solver"], Loader=yaml.SafeLoader)} for e in cont]
        entries = ["rate", "temperature", "density", "ionized_hydrogen_fraction", "scale_factor", "time", "recomb", "coll_ion"]
        i = int(f.name.replace("trace_", "").replace(".yml", ""))
        df = pl.DataFrame(dict(make_row(cont, x) for x in entries))
        df = df.with_columns(pl.lit(i).alias("id"))
        df = df.with_columns(pl.col("rate").log10().alias("rate_log10"))
        return df
    except:
        raise
        return None

# df = pl.read_csv("out.df")
# df = df.filter(pl.col("rate_log10") > 10)
dfs = [getDf(f) for f in sys.argv[1:]]
df = pl.concat([df for df in dfs if df is not None])
df.write_csv("out.df")
    
# n = int(df.shape[0] / 2)
print(df.top_k(by="density", k=1)["density"])
print(df.bottom_k(by="density", k=1)["density"])
# df = df.top_k(by="density", k=n)
df = df.top_k(by="density", k=1000000)
print(df)
print(df["time"])

g = sns.relplot(
    x=df["time"],
    y=df["recomb"],
    hue=df["id"],
    # hue=df["temperature"],
    # palette=sns.color_palette("tab10")
)
ax = g.ax
ax.set_yscale('log')
ax.set_ylim([1e-4, 1])

plt.savefig("temp_over_time.png")
plt.show()

