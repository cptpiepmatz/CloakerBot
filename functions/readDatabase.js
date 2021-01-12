"use strict";

/**
 * Reads data from the db for a specific guild
 *
 * @param {string} collectionName
 * @param {Guild} guild
 * @returns {Promise<object>}
 */
async function readDatabase(collectionName, guild) {
  const collection = dbo.collection(collectionName);
  return await collection.findOne({GuildID: guild.id});
}

module.exports = readDatabase;