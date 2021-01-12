"use strict";

const discord = require("discord.js");

const embed = new discord.MessageEmbed()
  .setColor("#49ce34")
  .setTitle("CloakerBot Help Page")
  .setDescription('"Not so clever, are we?!"')
  .addField(
    "/cloaker @mention",
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
    "Display if the bot does random takedowns on this server."
  )
  .addField(
    "/cloaker random <true|false>",
    "Allows or denies random takedowns on this server." +
    "\n *Needs Permission*: ADMINISTRATOR"
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