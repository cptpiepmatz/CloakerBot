# 🎉 Update v3 - Resurrection of the CloakerBot 🎉

{CloakerBot} is alive again, now with v3!
It took basically forever, but I finally got myself together and finished
the full rewrite.
At least we got some cool new features out of it.

<hr size="large" divider="false">

## 🪦 Big Reset

{CloakerBot} was down for over a year. The Raspberry Pi hosting it died,
taking the entire database with it.
The rewrite wasn't done, and my motivation to finish v3 wasn't exactly high.
So yeah, that's why it took so long.
With the database gone, the bot needs to be fully reconfigured, and all stats
are back to zero.
I'm sorry about that, but I now have some basic backups in place.
So as long as my house doesn't burn down, we should be able to revive the bot
ASAP.

<hr size="large" divider="false">

## 🌱 Open Source

I promised that v3 would be open source, and I'm keeping that promise.
The entire source code for the bot is available [here]({repo_url}).
You can clone it, modify it and host your own instance. I'll still keep hosting
this one.
You can't sell it, though. Check the license.

You can also dig through the history if you're interested in how the bot changed
over time.

<hr size="large" divider="false">

*Ok, now to the actual new features:*

<hr size="large" divider="false">

## 🏸 Countering

The biggest addition is the new command `/counter`.
It allows users to counter an incoming takedown.
Use it while {CloakerBot} is in your channel to redirect the takedown
back at whoever called the bot.
If the bot decided to strike on its own with a random takedown, the command
will simply deflect it.

<container color="#ffcc4d">

⚠️ Takedowns from a Danger Zone cannot be deflected.

</container>

<hr size="large" divider="false">

## 📝 Log Channel

A much-requested feature was a log channel for the bot.
In v2 and earlier, it was often impossible to tell who called the bot or whether
it was just a random takedown.
This caused some wild arguments and sometimes even got the bot kicked from a
server, which is quite unfortunate.

You can now set up a log channel via `/config`.
Everything the bot wants to log will be posted there, so make sure it can
send messages in that channel.

Command replies are still ephemeral (only visible to you), so they won't spam
any channels.

<hr size="large" divider="false">

## 🗳️ Danger Zone Selection

Many people asked how to get rid of danger zones.
In v2 and earlier, there was no direct way to do that. The best option was to
create a new danger zone channel and then delete that channel.
That's a stupid, hacky workaround and shouldn't be necessary.

In v3, you can finally remove danger zones via `/config`.
You can also set up to 10 channels as danger zones, in case you'd prefer a whole
category to be unusable for voice chatting. Or just to show off that you're
immune and the other plebs aren't.
It's up to you.

<hr size="large" divider="false">

## 🛡️ Immunity Roles

Speaking of being immune.
As fun as the bot and its random takedowns can be, some people really
cannot stand them.
They get annoyed and demand that the bot be removed, which is a real bummer.

To give those people a way out without ruining the fun for everyone else,
you can now set up ~~pussy~~ immunity roles.
Using `/config`, you can define four roles:

- general immunity
- random immunity
- general-silent immunity
- random-silent immunity

General immunity protects against all takedowns. Random immunity only protects
against random takedowns, not those called by other users.
With the `-silent` variants, the bot won't even join the channel to attempt
a takedown covered by that immunity.

Hopefully this lets everyone have fun with the bot.

<hr size="large" divider="false">

## 🏅 Updated Ranks

The old ranking system was pretty fragile and didn't always work correctly.
Since we lost the database, we're all starting from zero.
Takedown stats are now recorded more precisely, so you can see who your rival
is, who gets taken down the most and who is practically a ghost.

<hr size="large" divider="false">

## 🔗 Linked Roles

Discord introduced
[Linked Roles](https://support.discord.com/hc/en-us/articles/10388356626711)
some time ago. Don't ask me when.
These let servers set requirements users must meet to get a role.

With v3, you can now use {CloakerBot} stats for those requirements.
Users can link their Discord account to the bot to qualify for roles based on
their stats.
Here's what the bot provides:

- how many successful takedowns you initiated globally (`times_smacked`)
- how many times you were taken down globally, including returned counters
  (`times_called`)
- how many successful counters and parries you performed globally
  (`times_countered`)

Discord connections are tied to users, not servers, so you can't use
server-specific stats.
Stats update automatically about once a minute while the bot is running.

<hr size="large" divider="false">

## 🪛 Nicer Configuration

Configuring the bot is now a bit less painful.
While I was working on v3, Discord added a bunch of nice features for bot
developers.
The config commands now use many of them, including buttons, dropdowns and
rich Markdown.

If you're still having trouble configuring the bot, try `/help`.
[Or join the CloakerBot Fort]({invite_url}).

<hr size="large" divider="false">

## ✨ Stability & Availability

The bot has been fully rewritten in a new language, with plenty of improvements
along the way.
It should be more stable and spend less time being dead.
If it still manages to get stuck, just ping me in the
[CloakerBot Fort]({invite_url}).

<hr size="large" divider="false">

<action-row>
  <button style="secondary" custom-id="{disable_notifications}">
    Don't send me owner notifications
  </button>
</action-row>
