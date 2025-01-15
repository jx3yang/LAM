import sqlite3
import numpy as np
from sentence_transformers import SentenceTransformer

connection = sqlite3.connect('anime_metadata.db')
cursor = connection.cursor()

query = "SELECT id, summary FROM anime_summary"

cursor.execute(query)
rows = cursor.fetchall()

cursor.close()
connection.close()

ids = [(str)(_id) for _id, _ in rows]
summaries = [summary for _, summary in rows]

with open("ids.txt", "w") as f:
    f.write("\n".join(ids))

model = SentenceTransformer("sentence-transformers/all-mpnet-base-v2", device="cuda")
embeddings = model.encode(summaries)
np.savetxt("embeddings.csv", embeddings, delimiter=",")
