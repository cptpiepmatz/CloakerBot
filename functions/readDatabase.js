"use strict";

const Discord = require("discord.js");

/**
 * Reads data from the db for a specific guild
 *
 * @param {string} collectionName
 * @param {Guild|Snowflake|"all"} guild
 * @param {object} [query]
 * @returns {Promise<object>|Promise<object[]>}
 */
async function readDatabase(collectionName, guild, query) {
  const collection = dbo.collection(collectionName);

  if (guild !== "all") {
    if (guild instanceof Discord.Guild) {
      guild = guild.id;
    }
    return await collection.findOne({GuildID: guild});
  }

  let cursor = collection.find(query);
  let returnData = await cursor.toArray();
  cursor.close();
  return returnData;
}

module.exports = readDatabase;