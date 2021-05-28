"use strict";

const writeDatabase = require("../functions/writeDatabase");
const getRandom = require("../functions/getRandom");

const replies = require("../server/replies");

/**
 * Command to set a role for takedowns.
 * @param {Message} message
 * @return {Promise<void>}
 */
async function setRole(message) {

  let args = message.content.trim().toLowerCase().split(/\s+/);
  const collectionName = "roles";

  // Check if the member has the perms needed
  if (!message.member.hasPermission("ADMINISTRATOR")) {
    await message.channel.send(getRandom(replies.noPerms));
    return;
  }

  // Without enough parameters, this will not work
  if (args.length < 3) {
    await message.channel.send(getRandom(replies.incorrectInput));
    return;
  }

  // Check for a valid snowflake or mention
  let matching = args[2].match(/^(?:<@&)?(\d{17,18})>?$/);
  if (!matching) {
    await message.channel.send(getRandom(replies.unknownInput));
    return;
  }

  // Check if the role can be fetched
  let role = await message.guild.roles.fetch(matching[1]);
  if (!role) {
    await message.channel.send(getRandom(replies.noMember));
    return;
  }

  // Write the role to the DB
  await writeDatabase(collectionName, message.guild, {
    takedownRole: role.id
  });

  await message.channel.send(`*You set the role ${role} to takedown people.*`);
  await message.channel.send("Well, you somehow did it.");

}

module.exports = setRole;
