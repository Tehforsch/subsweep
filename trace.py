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
    return (name, [pq.Quantity(x["solver"][name]).value for x in cont])

def getDf(f):
    f = Path(f)
    cont = yaml.load(open(f, "r"), Loader=yaml.SafeLoader)
    cont = [{"time": e["time"], "solver": yaml.load(e["solver"], Loader=yaml.SafeLoader)} for e in cont]
    entries = ["rate", "temperature", "density", "ionized_hydrogen_fraction", "scale_factor", "time"]
    i = int(f.name.replace("trace_", "").replace(".yml", ""))
    df = pl.DataFrame(dict(make_row(cont, x) for x in entries))
    df = df.with_columns(pl.lit(i).alias("id"))
    df = df.with_columns(pl.col("rate").log10().alias("rate_log10"))
    return df

df = pl.concat([getDf(f) for f in sys.argv[1:]])
    
print(df)

g = sns.relplot(
    x=df["time"],
    y=df["temperature"],
    style=df["id"],
    hue=df["ionized_hydrogen_fraction"],
    # palette=sns.color_palette("tab10")
)
ax = g.ax
ax.set_yscale('log')

plt.savefig("temp_over_time.png")
plt.show()

