"use strict";

/**
 * Clears data from the db for a specific guild.
 * Only the specified collection will be cleared
 *
 * @param {string} collectionName
 * @param {Guild} guild
 * @returns {Promise<object>}
 */
async function clearDatabase(collectionName, guild) {
  const collection = dbo.collection(collectionName);
  await collection.deleteOne({GuildID: guild.id});
}

module.exports = clearDatabase;