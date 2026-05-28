# HNSW concurrent delete/search fixture

Expected regression test:

1. Insert N vectors.
2. Start search.
3. Concurrently delete one candidate.
4. Ensure result filtering uses coherent deleted snapshot or explicitly retry/degrade.
