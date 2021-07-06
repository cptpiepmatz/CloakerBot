"use strict";

const discord = require("discord.js");

const embed = new discord.MessageEmbed()
  .setColor("#49ce34")
  .setTitle("CloakerBot Help Page")
  .setDescription('"Not so clever, are we?!"')
  .addField(
    "/cloaker <@mention|userID>",
    "Takes down the user mentioned. " +
    "The bot does need permissions to read the message, " +
    "join the channel and kick the user from the voice channel." +
    "\n *Needs Permission*: MOVE_MEMBERS"
    )
  .addField(
    "/cloaker help",
    "Opens this help page."
  )
  .addField(
    "/cloaker random",
    "Display if the bot does random takedowns on this server. " +
    "This also show the timings for random takedown checks."
  )
  .addField(
    "/cloaker random <true|false|min|max|default> <number>",
    "Allows or denies random takedowns on this server. " +
    "'default' sets to the current default of this bot and removes the data " +
    "for this from the database. Current default is no random kicking. " +
    "\nYou can set how frequently the bot appears by setting the min and max values." +
    "\n *Needs Permission*: ADMINISTRATOR"
  )
  .addField(
    "/cloaker setRole <role>",
    "Defines a role that is allowed to take down people." +
    "\nThis will overwrite the other roles. Only one role is can fulfill this " +
    "feature." +
    "\n *Needs Permission*: ADMINISTRATOR"
  )
  .addField(
    "/cloaker ranks <user>",
    "Generates a rank card based on tracked takedown stats." +
    "\n *Needs Permission*: ATTACH_FILES"
  )
  .addField("​",
    "If you need any more help, join the " +
    "[discord server](https://discord.gg/XMsdwM5NXf). " +
    "Also updates about the bot will be posted there."
  );

/**
 * Posts a help message for the author of the message
 *
 * @param {Message} message the message that called the command
 * @returns {Promise<void>}
 */
async function helpCommand(message) {
  await message.channel.send({embed: embed});
}

module.exports = helpCommand;
