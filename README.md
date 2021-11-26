# TimeTrack
A dead simple CLI time tracking tool.

## How does it work
- This program runs on the command line.
- There are two types of entires in the database: projects and checkpoints.
- A project has an automatically assigned ID that never changes, a long name and a short name.
- A checkpoint has a timestamp, a message, and optionally an associated project ID.
- All the program does is manipulate this simple .json database.

## Getting started
1. Place the program somewhere you can run it from.
   1. I recommend renaming it to something simple or creating an alias, I personally use `tt`.
1. Add a project with `tt add-project -l 'My long name' -s 'mln'`
   1. The short name is what you're going to write to associate an checkpoint with that project.
   1. The long name is only used for printing.
   1. The long and short names can be changed later by manually editing the .json database file.
1. When you start work, write `tt add`, this creates an empty checkpoint.
   1. An empty checkpoint is interpreted as "no work was done between the previous checkpoint and this checkpoint".
1. When you've finished a chunk of work that you want to track, write `tt add 'Message' 'shortname'`.
   1. This will create a checkpoint at the current time with the given message and projects.
   1. Use `-t HH:MM` to specify another time.
1. To see your tracked time today, write `tt log`.
   1. I use this command when I write my invoices to check how much time I've spent on different projects for a specific client. Write `tt log --help` for usage information.
1. Edit an existing checkpoint with `tt edit`, it edits the most recent checkpoint by default.
   1. Write `tt edit --help` to check different ways to edit checkpoints.
   1. To edit further back in history, use `tt log` to list the checkpoint and use the number in the leftmost column to refer to the checkpoint. For instance `tt log 2 -m 'My new message'` will change the checkpoint before the previous one.
1. Use `tt help` to for for more help.
