# Orange Whale
Orange whale is a backup software that uses telegram as free storge. A docker image is available, just mount
the folders you want to backup and pass the folder names to `LOCATIONS` to tell Orange Whale what to upload.

It requires you to also mount `/app/pub.txt`, a file containing your public PGP key. Yes it is required, and yes
it is useful. DO NOT upload your personal data on external services without encrypting them.
Now the following envvar:
- `TELOXIDE_TOKEN`: is the token of the telegram bot you'll be using
- `CHAT_ID`: the group/channel your bot will upload to
- `LOCATIONS`: the folders to upload
- `INTERVAL`: an integer indicating how often to backup specified in hours
- `RUST_LOG`: logging verbosity. If you're using docker I advise you to set it to `info`

## Backups
Once the bot starts uploading you'll realize that the data is split in multiple parts. That's because bots on
telegram have an upload limit of 50MB on files. Just download all the parts and `cat` them together. After that
you can decrypt using your PGP key and unpack using tar.
```sh
cat part_* | gpg -d | tar x
```
