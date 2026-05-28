# semantic-memory v21/v24 type notes

Minimum storage expectations for the final pass:

- preserve the original canonical JSON payload for every v21–v24 family,
- index by artifact family and recorded time,
- retain valid-time where the artifact meaning depends on windows or expiry,
- keep backpointer refs queryable without joining through operator folklore,
- never collapse effect, delegation, assurance, or continuity receipts into a generic event blob.
