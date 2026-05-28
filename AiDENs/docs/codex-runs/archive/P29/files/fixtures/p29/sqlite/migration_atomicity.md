# SQLite migration atomicity fixture

Expected regression test:

1. Simulate migration body success.
2. Simulate version-record insert failure.
3. Ensure transaction rolls back both.
