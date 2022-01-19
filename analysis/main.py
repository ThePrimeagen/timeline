import pandas as pd
import os
import numpy as np
import pandas as pd
import matplotlib.pyplot as plt

pd.set_option('display.max_colwidth', None)

def create_histograms(pf):
    names = pf.name.unique()
    print(names)
    for name in names:
        print(f'creating {name}')

        # global coms.. scares
        dist = pf[(pf.name == name)]
        if os.environ.get("REDUCE_DATA_BY_STD"):
            dist_mean = dist.duration.mean()
            dist_std = dist.duration.std()
            dist = dist[dist.duration <= dist_mean + dist_std * int(os.environ.get("REDUCE_DATA_BY_STD"))]

        dist.hist(bins=200, column="duration");
        plt.title(f'{name}')
        plt.savefig(f'./images/{name}.png')


pf = pd.read_csv(os.environ.get("FILE"))
pf.columns = [
    "measurement_type",
    "name",
    "duration",
    "additional_data",
]

#create_histograms(pf)
grouped = pf.groupby(['measurement_type', "name"])
print(grouped.describe())
