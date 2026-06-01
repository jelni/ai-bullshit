import json

# Ensure stats.json has a valid empty state or does not leak into tests
import os
if os.path.exists("stats.json"):
    os.remove("stats.json")
