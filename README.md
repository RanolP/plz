# plz

Suggest the command you may want for ergonomics

## The Idea

Basically it's the clone of [nvbn/thefuck](https://github.com/nvbn/thefuck) with some weird idea: govern the shell.

Instead of running `/usr/bin/your-sh`, start `/usr/bin/plz enter /usr/bin/your-sh`.
It will not only pipe stdio into the shell spawned inside but watch for correction on the fly.
The output buffer for providing data to correction matcher will be cleared every time you press enter key.
Run `plz` command will output one-line json with clear this line ansi escape code for exchanging data between the processes.

## Goals

- [ ] PoC
- [ ] Port thefuck matchers as many as we can
- [ ] GPT based shell command synth?
