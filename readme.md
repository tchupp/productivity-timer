# Productivity timer

This productivity timer takes inspiration from pomodoro, but inverts the method (it's not quite inversion, but it's a pithy description). Rather than starting with a threshold of time and counting down, you count up toward some target. The important part, though, is that you only count that work which is worth counting. The work I care about is `reading, writing, or thinking`, and so I don't count deploys, builds, and so on. That's the kind of work that makes you feel good about your pomodoro session when that session doesn't actually get much done. For a threshold, I shoot for 4 hours of focused work on `reading, writing, or thinking`.

```
pt -h
```

```
Productivity Timer 0.2.0
Aaron Arinder <aaronarinder@protonmail.com>
Productivity Timer is a CLI and Daemon for recording quality time gained on projects. Quality time is time spent
reading, writing, or thinking. Anything absent-minded (builds, deploys, [most] meetings, and so on) doesn't count.
Consistently spending quality time on problems you care about will eventually solve those problems; so, get to it!

USAGE:
    pt [FLAGS] [OPTIONS]

FLAGS:
    -b, --backup       Back up database to Google Drive. Requires a `.env` with API_KEY, GOOGLE_CLIENT_ID, and
                       GOOGLE_CLIENT_SECRET.
    -d, --daemonize    Initializes the daemon, which is used for recording durations and interacting with the host
                       system asynchronously to the CLI.
    -h, --help         Prints help information
    -p, --print        Prints from two places, either `db` for what's been saved or `tmp` for what's in
                       /var/tmp/productivity-timer/time-gained.
    -y, --sync         Syncs local database to what's been backed up in Google Drive. Requires a `.env` with API_KEY,
                       GOOGLE_CLIENT_ID, and GOOGLE_CLIENT_SECRET.
    -V, --version      Prints version information

OPTIONS:
    -a, --add <add>                Add an arbitrary number of minutes to count as one duration for time gained. Example:
                                   pt -a 10, which adds 10 minutes to your time gained.
    -c, --complete <complete>      Completes a session of recording quality time.
    -i, --interface <interface>    Opens a terminal interface.
    -s, --subtract <subtract>      Subtract an arbitrary number of minutes, counting as one duration, from a session.
                                   Example: pt -s 10, which subtracts 10 minutes from your session.
    -g, --tag-time <tag-time>      Get time gained for a tag.
    -t, --trigger <trigger>        Records a moment in time, either the beginning or end of a duration.
```

# Caveats

The daemon doesn't (yet) start automatically. So, use `pt -d` whenever you reboot or start the productivity timer for the first time. You'll need to kill the current daemon process after compilations to have your changes take affect (if they're daemon-related changes, which they most likely are): `kill $(cat ~/.productivity-timer/timer.pid)`

**This is early alpha; it saves an OAuth token in `~/.productivity-timer/token`**. Don't hook it up to anything you care about, and don't care about anything you shouldn't care about.

# Installation

Clone this repo, navigate to it, and then run `cargo install --path .`. The `pt` command should then be available. Test with `pt -h`.

# Examples


### Begin a duration

```
pt -t "prayer"
```

### End a duration

```
pt -t "prayer"
```

### Seeing your current duration

```
pt -p
```

You can plug this into your shell, i3, or wherever. Here's what I have in my .zshrc:

```
# allows for fns in prompt
setopt PROMPT_SUBST
print_time_gained(){ pt -p | tr -d '"'}
PROMPT='%{%f%b%k%}%K{red}$(print_time_gained)%k$(build_prompt)'

# https://www.zsh.org/mla/users/2007/msg00944.html
TMOUT=3
TRAPALRM() {
    zle reset-prompt
}
```

Which spits out:

<img width="130" alt="Screen Shot 2021-12-10 at 8 58 15 AM" src="https://user-images.githubusercontent.com/26738844/145585285-ead429d0-c8c8-45f0-ae65-78c6c232c0b8.png">

### Report on time gained

Replace "work" with whatever session tag you want. Session tags are how you bucket different sessions to a particular 'profile' or class of work (e.g., I use `personal` for my own projects and `work` for work-related stuff).

```
pt -t "work"
```

This will open an interface in your terminal. See the example below.

### Backup/syncing

You can back your database up to Google Drive. You need to set up your own application in GCP's console, giving the barest possible scopes to Google Drive API. Your redirect URL will need to be `http://localhost:8080`. You'll then need a `.env` file with your client ID, secret, and API key:

```
GOOGLE_CLIENT_ID=...
GOOGLE_CLIENT_SECRET=...
API_KEY=...
```

**Backup/syncing will upload your database file to your Google Drive account, replacing what was there before. Syncing _doesn't_ replace your local database file. You'll have to do this manually until I implement a sensible backup flow that doesn't destroy your local database without validation of the new one.**

#### Backup

I haven't implemented refresh flow for tokens, so you'll have to comment out the lines in the oauth file that get the token from the file in `~/.productivity-timer`, replacing them with a call to the function initiating OAuth.

```
pt -b
```

#### Sync

```
pt -y
```

You'll then need to manually `mv` the database to replace `~/.productivity-timer/time_gained`.

###

# Terminal interface

<img width="1473" alt="Screen Shot 2021-12-10 at 8 04 22 AM" src="https://user-images.githubusercontent.com/26738844/145578475-8f2d9e52-e288-4e6f-be3d-642a0f5a0d95.png">
