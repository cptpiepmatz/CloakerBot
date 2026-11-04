# 🎉 Update v3 🎉

{CloakerBot} Update v3 is live!
It took quite some time, but now we finally have some long awaited features.


## 🏸 Countering

The biggest addition is the new command `/counter`.
It allows users to counter an incoming takedown.
If invoked while the {CloakerBot} is in your channel the target will be 
redirected to the user that called the bot.
If the bot itself decided to strike you via a random takedown the command will 
simply deflect that takedown.

> ⚠️ Takedowns from a Danger Zone cannot get deflected. ⚠️


## 📝 Log Channel

A highly demanded feature was the ability for the bot to log messages.
In v2 and prior, it was often not possible to check who called the bot or if it 
even was a random takedown.
This caused some wild dispute and sometimes to the kick of the bot from a server 
which is quite unfortunate.

Via `/config` it is possible to define such a channel.
Everything that the bot wants to log will be posted there, so make sure that the 
bot is able to send messages in that channel.

Replies to commands are still send as ephemeral messages and won't spam any 
channels.


## 🗳️ Danger Zones Select

Many people asked how they can get rid of danger zones.
In v2 and prior, it was not directly possible, the best option was to create a 
new danger zone channel and then delete that channel.
That is a stupid and hacky workaround and shouldn't be necessary.

Now in v3 it is finally possible to remove danger zones via the 
`/config` command.
It is now also possible to define up to 10 channels as danger zones if you 
prefer a whole category to be unusable for voice chatting or as symbol of power 
how you are immune and the other plebs aren't. 
It is up to you.


## 🛡️ Immunity Roles

Talking about being immune.
Although this bot is quite fun and random with its takedowns, some people really 
cannot stand them.
They get annoyed and demand that the bot gets removed, which is a real bummer.

To give these people an option for their distress with the bot, it is now 
possible to define ~~pussy~~ immunity roles.
Using `/config` it is possible to define four roles:

- general immunity

- random immunity

- general-silent immunity

- random-silent immunity

The general immunity provides immunity against any kind of takedowns, while the 
random immunity only provides immunity against random takedowns but not user 
called takedowns.
And the *-silent variants make it that the bot won't even join the channel and 
try to execute a takedown.

Hopefully this allows all to have fun with this bot.


## 🏅 Updated Ranks

The old ranks system was very brittle and did not work correctly all the time.
The recorded ranks will therefore get removed, and we all start from zero.
Takedown stats are now recorded more precisely allowing to check who is your 
rival, who got taken down the most and who is practically a ghost.


## 🔗 Linked Roles

Discord introduced
[Linked Roles](https://support.discord.com/hc/en-us/articles/10388356626711-Connections-Linked-Roles-Admins)
about a year ago.
This allows servers to define criteria a user must reach in order to get a role.

With the introduction of v3 it is now possible to use the {CloakerBot} as a
source for such criteria.
Users can link their Discord account to the bot to qualify for roles based on their stats.
The metadata that the bot will provide are:

- how many successful takedowns you initiated globally (`times_smacked`)

- how many times you were taken down globally, including returned counters (`times_called`)

- how many successful counters and parries you performed globally (`times_countered`)

Discord connections are user-scoped, so server-local criteria are not available.
Stats update automatically about once a minute while the bot is running.


## 🪛 Nicer Configuration

Configuring the bot is now nicer than before.
While working on v3 Discord added a lot of nice features for bot developers.
Many of these features (e.g. Buttons, Select Dropdowns, rich Markdown) are now 
incorporated in the config commands.

If you have any trouble configuring the bot still, the `/help` command can help
you out.
Or join the CloakerBot Fort.


## ✨ Stability & Availability

The bot is fully rewritten in a new language with many improvements wisely 
planned.
This should cause the bot to be even more stable and available.
If the bot should still get stuck, simply ping me in the CloakerBot Fort.
