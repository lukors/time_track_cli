# TimeTrack
A dead simple CLI time tracking tool.

## How does it work
- This program runs on the command line.
- There are two types of entires in the database: tags and events.
- A tag has an automatically assigned ID that never changes, a long name and a short name.
- An event has a timestamp, a comment, and a list of associated tag IDs.
- All the program does is manipulate this simple .json database.

## Getting started
1. Place the program somewhere you can run it from.
   1. I recommend renaming it to something simple or creating an alias, I personally use `tt`.
1. Add tags with `tt add-tag -l 'My long name' -s 'mln'`
   1. Tags are used to associate an event with something, like a project or a type of task or a client etc. An event can have any number of associated tags. Personally I use one tag for each client, one for each project, and then have a few for different types of tasks like development, meetings, etc.
   1. The short name is what you're going to write to associate an event with that tag.
   1. The long name is useful to remember what the short name stands for, you never have to write this.
   1. The long and short names can be changed later by manually editing the .json database file.
   1. Once you've added the tags you want you're ready to use the program.
1. When you start work, write `tt add`, this creates an empty event.
   1. An empty event is interpreted as "no work was done between the previous event and this event".
   1. Events are always created at the current time if nothing else is specified with the `-t` flag.
1. When you've finished a chunk of work that you want to track, write `tt add 'Message' 'tag1 tag2 tag3'`.
   1. This will create an event at the current time with the given message and tags.
1. To see your tracked time today, write `tt log`.
   1. `tt log` has some useful flags that you can use to list events between certain times and with specific flags, write `tt log --help` for more information. I use this when I write my invoices to check how much time I've spent on different projects for a specific client.
1. Edit an existing event with `tt edit`, it edits the most recent event by default.
   1. Write `tt edit --help` to check different ways to edit events.
   1. To edit further back in history, use `tt log` to list the event and use the number in the leftmost column to refer to the event. For instance `tt log 2 -m 'My new message'` will change the event before the previous one.
1. Use `tt help` to for fast help.
