"use strict";

const Discord = require("discord.js");

/**
 * Writes data to the database
 *
 * @param {string} collectionName
 * @param {Guild|Snowflake} guild
 * @param {object} data to insert into the database
 * @returns {Promise<void>}
 */
async function writeDatabase(collectionName, guild, data) {
  const collection = dbo.collection(collectionName);

  let insertData = {
    $set: {}
  };

  if (guild instanceof Discord.Guild) {
    insertData.$set.GuildName = guild.name;
    guild = guild.id;
  }

  let filter = {
    GuildID: guild
  }

  for (let dataKey in data) {
    if (data.hasOwnProperty(dataKey)) {
      insertData.$set[dataKey] = data[dataKey];
    }
  }

  // uses the updateOne to update if the data is already in the db
  // adds the data if not existing
  await collection.updateOne(filter, insertData, {upsert: true});
}

module.exports = writeDatabase;