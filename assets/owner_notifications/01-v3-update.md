# 🎉 Update v3 - Resurrection of the CloakerBot 🎉

{CloakerBot} is alive again and in v3!
It took basically forever now, but I finally got myself together and updated
the bot and finished the full rewrite.
We now have at least some cool new features.

<hr size="large" divider="false">

## 🪦 Big Reset

The {CloakerBot} was down for over a year, my Raspberry Pi that hosted the bot
died and with it it's entire database.
The rewrite wasn't done and my motivation to finish v3 wasn't high.
So that's why it took so long.
Also this removed everything we had in the database, so the bot has to be fully
reconfigured again and all stats are reset.
I'm sorry about that, but now I have simple backups in place.
So as long my house down't burn down, we should be able to revive the bot asap. 

<hr size="large" divider="false">

## 🌱 Open Source

I promised that v3 will be open source, and I follow through on that.
The entire source code for the bot is available [here]({repo_url}).
You can clone it, modify it and host your own instance, but I will still host
this instance.
You cannot sell it, check the license.

You can also look through the history if you're interested how the bot changed
over time.

<hr size="large" divider="false">

*Ok, now to the actual new features:*

<hr size="large" divider="false">

## 🏸 Countering

The biggest addition is the new command `/counter`.
It allows users to counter an incoming takedown.
If invoked while the {CloakerBot} is in your channel the target will be 
redirected to the user that called the bot.
If the bot itself decided to strike you via a random takedown the command will 
simply deflect that takedown.

<container color="#ffcc4d">

⚠️ Takedowns from a Danger Zone cannot get deflected.

</container>

<hr size="large" divider="false">

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

<hr size="large" divider="false">

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

<hr size="large" divider="false">

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

<hr size="large" divider="false">

## 🏅 Updated Ranks

The old ranks system was very brittle and did not work correctly all the time.
Since we lost the database we all start from zero.
Takedown stats are now recorded more precisely allowing to check who is your 
rival, who got taken down the most and who is practically a ghost.

<hr size="large" divider="false">

## 🔗 Linked Roles

Discord introduced
[Linked Roles](https://support.discord.com/hc/en-us/articles/10388356626711-Connections-Linked-Roles-Admins)
some time ago, I don't know.
This allows servers to define criteria a user must reach in order to get a role.

With the introduction of v3 it is now possible to use the {CloakerBot} as a
source for such criteria.
Users can link their Discord account to the bot to qualify for roles based on
their stats.
The metadata that the bot will provide are:

- how many successful takedowns you initiated globally (`times_smacked`)
- how many times you were taken down globally, including returned counters (`times_called`)
- how many successful counters and parries you performed globally (`times_countered`)

Discord connections are user-scoped, so server-local criteria are not available.
Stats update automatically about once a minute while the bot is running.

<hr size="large" divider="false">

## 🪛 Nicer Configuration

Configuring the bot is now nicer than before.
While working on v3 Discord added a lot of nice features for bot developers.
Many of these features (e.g. Buttons, Select Dropdowns, rich Markdown) are now 
incorporated in the config commands.

If you have any trouble configuring the bot still, the `/help` command can help
you out.
[Or join the CloakerBot Fort]({invite_url}).

<hr size="large" divider="false">

## ✨ Stability & Availability

The bot is fully rewritten in a new language with many improvements wisely 
planned.
This should cause the bot to be even more stable and available.
If the bot should still get stuck, simply ping me in the
[CloakerBot Fort]({invite_url}).

<hr size="large" divider="false">

<action-row>
  <button style="secondary" custom-id="{disable_notifications}">
    Don't send me owner notifications
  </button>
</action-row>
