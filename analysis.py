import csv
import os
import pandas as pd

def get_csv(filename):
    with open(filename, newline='') as csvfile:
        csv_reader = csv.reader(csvfile, delimiter=',')
        csv_arr = []
        track_id = 0
        for row in csv_reader:
            if len(row) > 2:
                row[2] = int(row[2])
                csv_arr.append(row)

        return csv_arr

pf = pd.DataFrame(get_csv(os.environ.get("FILE")))
pf.columns = [
    "measurement_type",
    "name",
    "duration",
]
grouped = pf.groupby(["measurement_type", "name"])

print(grouped.describe())
