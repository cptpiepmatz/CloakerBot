"use strict";

const readDatabase = require("../functions/readDatabase");
const writeDatabase = require("../functions/writeDatabase");

/**
 * Updates all guild names in the db
 *
 * @param {Message} message
 * @returns {Promise<void>}
 */
async function fetchGuildNames(message) {
  if (message.author.id !== "137280289708834816") {
    await message.channel.send(
      "I'm proud you found that command, but you have perms for it.");
    return;
  }
  let client = message.client;
  let collections = await dbo.collections();
  let guilds = client.guilds.cache;
  for (let collection of collections) {
    let guildData = await readDatabase(collection.collectionName, "all", {});
    for (let guildDate of guildData) {
      let guildID = guildDate.GuildID;
      let guildName = guilds.get(guildID).name;
      await writeDatabase(collection.collectionName, guildID, {GuildName: guildName});
    }
  }
  await message.channel.send("Updated all names!");
}

module.exports = fetchGuildNames;