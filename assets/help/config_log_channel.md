Define what channel to use for logging.

The command opens a form with the current channel selected. Submit to save,
or clear the selection and submit to restore automatic fallback. Cancelling changes nothing.


The CloakerBot will post updates and takedown logs into the log channel.

If no log channel is defined a fallback will be searched for it.
The first possible fallback is the system channel, the one where Discord 
automatically posts a message when someone joins or leaves the server.

The second is fallback is the public updates channel, it's the channel that 
Discord uses to push updates and notifications to server admins and moderators.

The third fallback channel is the #general channel.
Usually such a channel exists upon creating a Discord server.


The bot will first verify if the channel is visible to the bot and if it has 
permission to send messages in that channel. 
If the bot is not able to send messages in any of these channels, no logging 
will take place. 
To turn off logging, simply revoke the bot's permission to send messages. 
All user notifications will be handled through interactions and the bot will 
respond with an ephemeral message only visible to the user interacting with the 
bot.
